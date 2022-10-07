use crate::ast;
use std::collections::HashMap;

/// Evaluates specified `prog`, given `input`.
///
/// See also: [`crate::vm::compile()`].
///
/// # Example
///
/// ```
/// # use rast_jit_vm::{examples, prelude::*};
/// #
/// assert_eq!(55, vm::eval(&examples::fibonacci(), 10));
/// ```
pub fn eval<'s, Input, Output>(prog: &ast::Program<'s>, input: Input) -> Output
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

    let mut ctxt = RuntimeContext {
        vars: HashMap::from_iter(vec![("input", input.into_value())]),
    };

    prog.body.eval(&mut ctxt).unbox()
}

impl<'s> ast::Node<'s> {
    fn eval(&self, ctxt: &mut RuntimeContext<'s>) -> ast::Value<'s> {
        match self {
            ast::Node::Const(value) => Self::eval_const(value),
            ast::Node::Var(name) => Self::eval_var(ctxt, name),
            ast::Node::ExtractTuple { expr, idx } => Self::eval_extract_tuple(ctxt, expr, *idx),
            ast::Node::ExtractArray { expr, idx } => Self::eval_extract_array(ctxt, expr, idx),
            ast::Node::Cast { expr, ty } => Self::eval_cast(ctxt, expr, ty),

            ast::Node::Add { lhs, rhs }
            | ast::Node::Sub { lhs, rhs }
            | ast::Node::Mul { lhs, rhs }
            | ast::Node::Div { lhs, rhs }
            | ast::Node::Mod { lhs, rhs }
            | ast::Node::Eq { lhs, rhs }
            | ast::Node::Neq { lhs, rhs }
            | ast::Node::Gt { lhs, rhs }
            | ast::Node::GtEq { lhs, rhs }
            | ast::Node::Lt { lhs, rhs }
            | ast::Node::LtEq { lhs, rhs }
            | ast::Node::And { lhs, rhs }
            | ast::Node::Or { lhs, rhs } => self.eval_binop(ctxt, lhs, rhs),

            ast::Node::Declare { name, value } => Self::eval_declare(ctxt, name, value),
            ast::Node::Assign { name, value } => Self::eval_assign(ctxt, name, value),
            ast::Node::While { cond, body } => Self::eval_while(ctxt, cond, body),
            ast::Node::Print(nodes) => Self::eval_print(ctxt, nodes),
            ast::Node::Block(nodes) => Self::eval_block(ctxt, nodes),
        }
    }

    fn eval_const(value: &ast::Value<'s>) -> ast::Value<'s> {
        value.to_owned()
    }

    fn eval_var(ctxt: &mut RuntimeContext<'s>, name: ast::Ident<'s>) -> ast::Value<'s> {
        ctxt.vars
            .get(name)
            .unwrap_or_else(|| panic!("Cannot find variable `{}`", name))
            .to_owned()
    }

    fn eval_extract_tuple(
        ctxt: &mut RuntimeContext<'s>,
        expr: &Self,
        idx: usize,
    ) -> ast::Value<'s> {
        expr.eval(ctxt).unbox_tuple()[idx].to_owned()
    }

    fn eval_extract_array(
        ctxt: &mut RuntimeContext<'s>,
        expr: &Self,
        idx: &Self,
    ) -> ast::Value<'s> {
        let expr = expr.eval(ctxt).unbox::<&str>();
        let idx = idx.eval(ctxt).unbox::<i64>() as _;
        let value = expr.chars().nth(idx).expect("Index out of bounds");

        ast::Value::Char(value)
    }

    fn eval_cast(ctxt: &mut RuntimeContext<'s>, expr: &Self, ty: &ast::Type) -> ast::Value<'s> {
        let expr = expr.eval(ctxt);

        match (expr.ty(), ty) {
            (ast::Type::Int, ast::Type::Float) => ast::Value::Float(expr.unbox::<i64>() as _),
            (ast::Type::Float, ast::Type::Int) => ast::Value::Int(expr.unbox::<f32>() as _),
            (source_ty, target_ty) => panic!("Cannot cast `{:?}` to `{:?}`", source_ty, target_ty),
        }
    }

    fn eval_binop(&self, ctxt: &mut RuntimeContext<'s>, lhs: &Self, rhs: &Self) -> ast::Value<'s> {
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

        let op = match self {
            ast::Node::Add { .. } => Op::Add,
            ast::Node::Sub { .. } => Op::Sub,
            ast::Node::Mul { .. } => Op::Mul,
            ast::Node::Div { .. } => Op::Div,
            ast::Node::Mod { .. } => Op::Mod,

            ast::Node::Eq { .. } => Op::Eq,
            ast::Node::Neq { .. } => Op::Neq,
            ast::Node::Gt { .. } => Op::Gt,
            ast::Node::GtEq { .. } => Op::GtEq,
            ast::Node::Lt { .. } => Op::Lt,
            ast::Node::LtEq { .. } => Op::LtEq,

            ast::Node::And { .. } => Op::And,
            ast::Node::Or { .. } => Op::Or,

            _ => unreachable!(),
        };

        let lhs = lhs.eval(ctxt);
        let rhs = rhs.eval(ctxt);

        match (lhs.ty(), op, rhs.ty()) {
            (ast::Type::Bool, op @ (Op::Eq | Op::Neq | Op::And | Op::Or), ast::Type::Bool) => {
                let lhs = lhs.unbox::<bool>();
                let rhs = rhs.unbox::<bool>();

                ast::Value::Bool(match op {
                    Op::Eq => lhs == rhs,
                    Op::Neq => lhs != rhs,
                    Op::And => lhs && rhs,
                    Op::Or => lhs || rhs,
                    _ => unreachable!(),
                })
            }

            (
                ast::Type::Int,
                op @ (Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod),
                ast::Type::Int,
            ) => {
                let lhs = lhs.unbox::<i64>();
                let rhs = rhs.unbox::<i64>();

                ast::Value::Int(match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => lhs / rhs,
                    Op::Mod => lhs % rhs,
                    _ => unreachable!(),
                })
            }

            (
                ast::Type::Int,
                op @ (Op::Eq | Op::Neq | Op::Gt | Op::GtEq | Op::Lt | Op::LtEq),
                ast::Type::Int,
            ) => {
                let lhs = lhs.unbox::<i64>();
                let rhs = rhs.unbox::<i64>();

                ast::Value::Bool(match op {
                    Op::Eq => lhs == rhs,
                    Op::Neq => lhs != rhs,
                    Op::Gt => lhs > rhs,
                    Op::GtEq => lhs >= rhs,
                    Op::Lt => lhs < rhs,
                    Op::LtEq => lhs <= rhs,
                    _ => unreachable!(),
                })
            }

            (
                ast::Type::Float,
                op @ (Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod),
                ast::Type::Float,
            ) => {
                let lhs = lhs.unbox::<f32>();
                let rhs = rhs.unbox::<f32>();

                ast::Value::Float(match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => lhs / rhs,
                    Op::Mod => lhs % rhs,
                    _ => unreachable!(),
                })
            }

            (
                ast::Type::Float,
                op @ (Op::Eq | Op::Neq | Op::Gt | Op::GtEq | Op::Lt | Op::LtEq),
                ast::Type::Float,
            ) => {
                let lhs = lhs.unbox::<f32>();
                let rhs = rhs.unbox::<f32>();

                ast::Value::Bool(match op {
                    Op::Eq => lhs == rhs,
                    Op::Neq => lhs != rhs,
                    Op::Gt => lhs > rhs,
                    Op::GtEq => lhs >= rhs,
                    Op::Lt => lhs < rhs,
                    Op::LtEq => lhs <= rhs,
                    _ => unreachable!(),
                })
            }

            (lhs_ty, op, rhs_ty) => {
                panic!("Unknown operation `{:?}` `{:?}` `{:?}`", lhs_ty, op, rhs_ty)
            }
        }
    }

    fn eval_declare(
        ctxt: &mut RuntimeContext<'s>,
        name: ast::Ident<'s>,
        value: &Self,
    ) -> ast::Value<'s> {
        let value = value.eval(ctxt);

        ctxt.vars.insert(name, value);

        ast::Value::Unit
    }

    fn eval_assign(
        ctxt: &mut RuntimeContext<'s>,
        name: ast::Ident<'s>,
        value: &Self,
    ) -> ast::Value<'s> {
        if name == "input" {
            panic!("`input` variable is read-only");
        }

        let new_value = value.eval(ctxt);

        let curr_value = ctxt
            .vars
            .get(name)
            .unwrap_or_else(|| panic!("Cannot find variable `{}`", name));

        if new_value.ty() != curr_value.ty() {
            panic!(
                "Cannot assign `{:?}` to a variable of type `{:?}`",
                new_value.ty(),
                curr_value.ty()
            );
        }

        ctxt.vars.insert(name, new_value);

        ast::Value::Unit
    }

    fn eval_while(ctxt: &mut RuntimeContext<'s>, cond: &Self, body: &Self) -> ast::Value<'s> {
        while cond.eval(ctxt).unbox() {
            body.eval(ctxt);
        }

        ast::Value::Unit
    }

    fn eval_print(ctxt: &mut RuntimeContext<'s>, nodes: &[Self]) -> ast::Value<'s> {
        for node in nodes {
            node.eval(ctxt).print();
        }

        ast::Value::Unit
    }

    fn eval_block(ctxt: &mut RuntimeContext<'s>, nodes: &[Self]) -> ast::Value<'s> {
        let mut value = ast::Value::Unit;

        for node in nodes {
            value = node.eval(ctxt);
        }

        value
    }
}

struct RuntimeContext<'s> {
    vars: HashMap<ast::Ident<'s>, ast::Value<'s>>,
}
