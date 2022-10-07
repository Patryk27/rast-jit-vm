use crate::ast;
use std::collections::BTreeMap;

/// Type-checks given `prog` and compiles it into a Rust function.
///
/// See also: [`crate::vm::eval()`].
///
/// # Example
///
/// ```
/// # use rast_jit_vm::{examples, prelude::*};
/// #
/// let fib = vm::compile(examples::fibonacci());
///
/// assert_eq!(55, fib(10));
/// ```
pub fn compile<'s, Input, Output>(prog: ast::Program<'s>) -> impl Fn(Input) -> Output + 's
where
    Input: ast::IntoValue<'s>,
    Output: ast::FromValue<'s>,
{
    if Input::ty().map_or(false, |ty| ty != prog.input) {
        panic!(
            "Invalid invocation: Generic parameter `Input` must be `{:?}`",
            prog.input,
        );
    }

    if Output::ty().map_or(false, |ty| ty != prog.output) {
        panic!(
            "Invalid invocation: Generic parameter `Output` must be `{:?}`",
            prog.output
        );
    }

    let mut ctxt = CompilationContext::new(prog.input);
    let (ty, thunk) = prog.body.compile(&mut ctxt);

    if ty != prog.output {
        panic!(
            "Output type mismatch: Program declared to return `{:?}`, but in \
             reality it returns `{:?}`",
            prog.output, ty
        );
    }

    let stack_size = ctxt.stack.len();

    move |arg| {
        let mut ctxt = RuntimeContext::new(stack_size, arg.into_value());

        (thunk)(&mut ctxt).unbox()
    }
}

impl<'s> ast::Node<'s> {
    fn compile(self, ctxt: &mut CompilationContext<'s>) -> (ast::Type, Thunk<'s>) {
        match self {
            ast::Node::Const(value) => Self::compile_const(value),
            ast::Node::Var(name) => Self::compile_var(ctxt, name),
            ast::Node::ExtractTuple { expr, idx } => Self::compile_extract_tuple(ctxt, *expr, idx),
            ast::Node::ExtractArray { expr, idx } => Self::compile_extract_array(ctxt, *expr, *idx),
            ast::Node::Cast { expr, ty } => Self::compile_cast(ctxt, *expr, ty),

            this @ (ast::Node::Add { .. }
            | ast::Node::Sub { .. }
            | ast::Node::Mul { .. }
            | ast::Node::Div { .. }
            | ast::Node::Mod { .. }
            | ast::Node::Eq { .. }
            | ast::Node::Neq { .. }
            | ast::Node::Gt { .. }
            | ast::Node::GtEq { .. }
            | ast::Node::Lt { .. }
            | ast::Node::LtEq { .. }
            | ast::Node::And { .. }
            | ast::Node::Or { .. }) => this.compile_binop(ctxt),

            ast::Node::Declare { name, value } => Self::compile_declare(ctxt, name, *value),
            ast::Node::Assign { name, value } => Self::compile_assign(ctxt, name, *value),
            ast::Node::While { cond, body } => Self::compile_while(ctxt, *cond, *body),
            ast::Node::Print(nodes) => Self::compile_print(ctxt, nodes),
            ast::Node::Block(nodes) => Self::compile_block(ctxt, nodes),
        }
    }

    fn compile_const(value: ast::Value<'s>) -> (ast::Type, Thunk<'s>) {
        let ty = value.ty();
        let thunk = thunk(move |_| value.clone());

        (ty, thunk)
    }

    fn compile_var(
        ctxt: &mut CompilationContext<'s>,
        name: ast::Ident<'s>,
    ) -> (ast::Type, Thunk<'s>) {
        let id = ctxt
            .vars
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Unknown variable: {}", name));

        let ty = ctxt.stack[id].clone();
        let thunk = thunk(move |ctxt| ctxt.stack[id].clone());

        (ty, thunk)
    }

    fn compile_extract_tuple(
        ctxt: &mut CompilationContext<'s>,
        expr: Self,
        idx: usize,
    ) -> (ast::Type, Thunk<'s>) {
        let (ty, expr) = expr.compile(ctxt);

        let tys = match ty {
            ast::Type::Tuple(tys) => tys,
            ty => panic!("Invalid type: `{:?}` is not a tuple", ty),
        };

        let ty = tys.get(idx).cloned().unwrap_or_else(|| {
            panic!(
                "Invalid subscript: Tuple `{:?}` doesn't have element .{}",
                tys, idx
            )
        });

        let thunk = thunk(move |ctxt| (expr)(ctxt).unbox_tuple()[idx].clone());

        (ty, thunk)
    }

    fn compile_extract_array(
        ctxt: &mut CompilationContext<'s>,
        expr: Self,
        idx: Self,
    ) -> (ast::Type, Thunk<'s>) {
        let (expr_ty, expr) = expr.compile(ctxt);
        let (idx_ty, idx) = idx.compile(ctxt);

        match expr_ty {
            ast::Type::Str => (),
            ty => panic!("Invalid type: Expected `str`, got `{:?}`", ty),
        }

        match idx_ty {
            ast::Type::Int => (),
            ty => panic!("Invalid type: Expected `int`, got `{:?}`", ty),
        }

        let thunk = thunk(move |ctxt| {
            let expr = (expr)(ctxt).unbox::<&str>();
            let idx = (idx)(ctxt).unbox::<i64>() as _;
            let char = expr.chars().nth(idx).expect("Index out of bounds");

            ast::Value::Char(char)
        });

        (ast::Type::Char, thunk)
    }

    fn compile_cast(
        ctxt: &mut CompilationContext<'s>,
        expr: Self,
        target_ty: ast::Type,
    ) -> (ast::Type, Thunk<'s>) {
        enum Cast {
            IntToFloat,
            FloatToInt,
        }

        let (source_ty, expr) = expr.compile(ctxt);

        let cast = match (source_ty, &target_ty) {
            (ast::Type::Int, ast::Type::Float) => Cast::IntToFloat,
            (ast::Type::Float, ast::Type::Int) => Cast::FloatToInt,
            (source_ty, target_ty) => panic!("Cannot cast `{:?}` to `{:?}`", source_ty, target_ty),
        };

        let thunk = thunk(move |ctxt| {
            let expr = (expr)(ctxt);

            match cast {
                Cast::IntToFloat => ast::Value::Float(expr.unbox::<i64>() as _),
                Cast::FloatToInt => ast::Value::Int(expr.unbox::<f32>() as _),
            }
        });

        (target_ty, thunk)
    }

    fn compile_binop(self, ctxt: &mut CompilationContext<'s>) -> (ast::Type, Thunk<'s>) {
        #[derive(Debug)]
        enum Op {
            Add,
            Sub,
            Mul,
            Div,
            Mod,

            Eq,
            Neq,
            Gt,
            GtEq,
            Lt,
            LtEq,

            And,
            Or,
        }

        let (op, lhs, rhs) = match self {
            ast::Node::Add { lhs, rhs } => (Op::Add, lhs, rhs),
            ast::Node::Sub { lhs, rhs } => (Op::Sub, lhs, rhs),
            ast::Node::Mul { lhs, rhs } => (Op::Mul, lhs, rhs),
            ast::Node::Div { lhs, rhs } => (Op::Div, lhs, rhs),
            ast::Node::Mod { lhs, rhs } => (Op::Mod, lhs, rhs),

            ast::Node::Eq { lhs, rhs } => (Op::Eq, lhs, rhs),
            ast::Node::Neq { lhs, rhs } => (Op::Neq, lhs, rhs),
            ast::Node::Gt { lhs, rhs } => (Op::Gt, lhs, rhs),
            ast::Node::GtEq { lhs, rhs } => (Op::GtEq, lhs, rhs),
            ast::Node::Lt { lhs, rhs } => (Op::Lt, lhs, rhs),
            ast::Node::LtEq { lhs, rhs } => (Op::LtEq, lhs, rhs),

            ast::Node::And { lhs, rhs } => (Op::And, lhs, rhs),
            ast::Node::Or { lhs, rhs } => (Op::Or, lhs, rhs),

            _ => unreachable!(),
        };

        let (lhs_ty, lhs) = lhs.compile(ctxt);
        let (rhs_ty, rhs) = rhs.compile(ctxt);

        #[allow(clippy::single_match)]
        match (&lhs_ty, &op, &rhs_ty) {
            (ast::Type::Bool, Op::Eq, ast::Type::Bool) => {
                let thunk = thunk(move |ctxt| {
                    let lhs = lhs(ctxt).unbox::<bool>();
                    let rhs = rhs(ctxt).unbox::<bool>();

                    ast::Value::Bool(lhs == rhs)
                });

                return (ast::Type::Bool, thunk);
            }

            _ => {
                // Because there's so many types (all three of them!) and so
                // many operations, the rest of those have been implemented
                // through the macro below.
                //
                // The code above serves as an example of what the macro
                // outputs.
            }
        }

        macro_rules! ops {
            ($lhs:expr, $rhs:expr, $lhs_ty:expr, $rhs_ty:expr, $op:expr, {
                $( $match_lhs_ty:tt $match_op:tt $match_rhs_ty:tt = $out_ty:tt: $eval:expr; )*
            }) => {
                match ($lhs_ty, $op, $rhs_ty) {
                    $(
                        (ops!(@ty $match_lhs_ty), ops!(@op $match_op), ops!(@ty $match_rhs_ty)) => {
                            let ty = ops!(@ty $out_ty);

                            let thunk = thunk(move |ctxt| {
                                let lhs = ops!(@unbox $lhs, ctxt, $match_lhs_ty);
                                let rhs = ops!(@unbox $rhs, ctxt, $match_rhs_ty);
                                let out = $eval(lhs, rhs);

                                ops!(@box out, $out_ty)
                            });

                            (ty, thunk)
                        }
                    )*

                    (lhs_ty, op, rhs_ty) => {
                        panic!("Unknown operation `{:?}` `{:?}` `{:?}`", lhs_ty, op, rhs_ty);
                    }
                }
            };

            (@ty Bool) => { ast::Type::Bool };
            (@ty Int) => { ast::Type::Int };
            (@ty Float) => { ast::Type::Float };

            (@op Add) => { Op::Add };
            (@op Sub) => { Op::Sub };
            (@op Mul) => { Op::Mul };
            (@op Div) => { Op::Div };
            (@op Mod) => { Op::Mod };

            (@op Eq) => { Op::Eq };
            (@op Neq) => { Op::Neq };
            (@op Gt) => { Op::Gt };
            (@op GtEq) => { Op::GtEq };
            (@op Lt) => { Op::Lt };
            (@op LtEq) => { Op::LtEq };

            (@op And) => { Op::And };
            (@op Or) => { Op::Or };

            (@unbox $expr:expr, $ctxt:expr, Bool) => { ($expr)($ctxt).unbox::<bool>() };
            (@unbox $expr:expr, $ctxt:expr, Int) => { ($expr)($ctxt).unbox::<i64>() };
            (@unbox $expr:expr, $ctxt:expr, Float) => { ($expr)($ctxt).unbox::<f32>() };

            (@box $expr:expr, Bool) => { ast::Value::Bool($expr) };
            (@box $expr:expr, Int) => { ast::Value::Int($expr) };
            (@box $expr:expr, Float) => { ast::Value::Float($expr) };
        }

        ops!(lhs, rhs, lhs_ty, rhs_ty, op, {
            // Bool Eq Bool = Bool: |lhs, rhs| lhs == rhs; // Already implemented above
            Bool Neq Bool = Bool: |lhs, rhs| lhs != rhs;
            Bool And Bool = Bool: |lhs, rhs| lhs && rhs;
            Bool Or Bool = Bool: |lhs, rhs| lhs || rhs;

            Int Add Int = Int: |lhs, rhs| lhs + rhs;
            Int Sub Int = Int: |lhs, rhs| lhs - rhs;
            Int Mul Int = Int: |lhs, rhs| lhs * rhs;
            Int Div Int = Int: |lhs, rhs| lhs / rhs;
            Int Mod Int = Int: |lhs, rhs| lhs % rhs;

            Int Eq Int = Bool: |lhs, rhs| lhs == rhs;
            Int Neq Int = Bool: |lhs, rhs| lhs != rhs;
            Int Gt Int = Bool: |lhs, rhs| lhs > rhs;
            Int GtEq Int = Bool: |lhs, rhs| lhs >= rhs;
            Int Lt Int = Bool: |lhs, rhs| lhs < rhs;
            Int LtEq Int = Bool: |lhs, rhs| lhs <= rhs;

            Float Add Float = Float: |lhs, rhs| lhs + rhs;
            Float Sub Float = Float: |lhs, rhs| lhs - rhs;
            Float Mul Float = Float: |lhs, rhs| lhs * rhs;
            Float Div Float = Float: |lhs, rhs| lhs / rhs;
            Float Mod Float = Float: |lhs, rhs| lhs % rhs;

            Float Eq Float = Bool: |lhs, rhs| lhs == rhs;
            Float Neq Float = Bool: |lhs, rhs| lhs != rhs;
            Float Gt Float = Bool: |lhs, rhs| lhs > rhs;
            Float GtEq Float = Bool: |lhs, rhs| lhs >= rhs;
            Float Lt Float = Bool: |lhs, rhs| lhs < rhs;
            Float LtEq Float = Bool: |lhs, rhs| lhs <= rhs;
        })
    }

    fn compile_declare(
        ctxt: &mut CompilationContext<'s>,
        name: ast::Ident<'s>,
        value: Self,
    ) -> (ast::Type, Thunk<'s>) {
        let (ty, value) = value.compile(ctxt);
        let id = ctxt.allocate_var(name, ty);

        let node = thunk(move |ctxt| {
            ctxt.stack[id] = (value)(ctxt);
            ast::Value::Unit
        });

        (ast::Type::Unit, node)
    }

    fn compile_assign(
        ctxt: &mut CompilationContext<'s>,
        name: ast::Ident<'s>,
        value: Self,
    ) -> (ast::Type, Thunk<'s>) {
        if name == "input" {
            panic!("`input` variable is read-only");
        }

        let id = ctxt
            .vars
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Unknown variable: {}", name));

        let ty = ctxt.stack[id].clone();
        let (value_ty, value) = value.compile(ctxt);

        if value_ty != ty {
            panic!(
                "Type mismatch: Cannot assign `{:?}` to `{:?}`",
                value_ty, ty
            );
        }

        let thunk = thunk(move |ctxt| {
            ctxt.stack[id] = (value)(ctxt);
            ast::Value::Unit
        });

        (ast::Type::Unit, thunk)
    }

    fn compile_while(
        ctxt: &mut CompilationContext<'s>,
        cond: Self,
        body: Self,
    ) -> (ast::Type, Thunk<'s>) {
        let (cond_ty, cond) = cond.compile(ctxt);
        let (_, body) = body.compile(ctxt);

        if cond_ty != ast::Type::Bool {
            panic!("Type mismatch: Loop's condition was expected to be `bool`");
        }

        let thunk = thunk(move |ctxt| {
            while (cond)(ctxt).unbox() {
                (body)(ctxt);
            }

            ast::Value::Unit
        });

        (ast::Type::Unit, thunk)
    }

    fn compile_print(
        ctxt: &mut CompilationContext<'s>,
        nodes: Vec<Self>,
    ) -> (ast::Type, Thunk<'s>) {
        let nodes: Vec<_> = nodes
            .into_iter()
            .map(|node| node.compile(ctxt))
            .map(|(_, node)| node)
            .collect();

        let node = thunk(move |ctxt| {
            for node in &nodes {
                (node)(ctxt).print();
            }

            ast::Value::Unit
        });

        (ast::Type::Unit, node)
    }

    fn compile_block(
        ctxt: &mut CompilationContext<'s>,
        nodes: Vec<Self>,
    ) -> (ast::Type, Thunk<'s>) {
        let (types, nodes): (Vec<_>, Vec<_>) =
            nodes.into_iter().map(|node| node.compile(ctxt)).unzip();

        let ty = types.into_iter().last().unwrap_or(ast::Type::Unit);

        let thunk = thunk(move |ctxt| {
            let mut value = ast::Value::Unit;

            for node in &nodes {
                value = (node)(ctxt);
            }

            value
        });

        (ty, thunk)
    }
}

struct CompilationContext<'s> {
    /// Variables allocated so far
    stack: Vec<ast::Type>,

    /// Maps variable name into its stack slot
    vars: BTreeMap<ast::Ident<'s>, usize>,
}

impl<'s> CompilationContext<'s> {
    fn new(input: ast::Type) -> Self {
        let mut this = Self {
            stack: Default::default(),
            vars: Default::default(),
        };

        this.allocate_var("input", input);
        this
    }

    fn allocate_var(&mut self, name: ast::Ident<'s>, ty: ast::Type) -> usize {
        let id = self.stack.len();

        self.stack.push(ty);

        if self.vars.insert(name, id).is_some() {
            panic!("Variable `{}` has been already defined", name);
        }

        id
    }
}

struct RuntimeContext<'s> {
    stack: Vec<ast::Value<'s>>,
}

impl<'s> RuntimeContext<'s> {
    fn new(stack_size: usize, input: ast::Value<'s>) -> Self {
        let mut stack = vec![ast::Value::Unit; stack_size];

        stack[0] = input;

        Self { stack }
    }
}

type Thunk<'s> = Box<dyn Fn(&mut RuntimeContext<'s>) -> ast::Value<'s> + 's>;

fn thunk<'s>(f: impl Fn(&mut RuntimeContext<'s>) -> ast::Value<'s> + 's) -> Thunk<'s> {
    Box::new(f)
}
