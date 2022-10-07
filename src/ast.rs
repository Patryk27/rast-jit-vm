mod from_value;
mod into_value;

pub use self::{from_value::*, into_value::*};

#[derive(Clone, Debug)]
pub struct Program<'s> {
    pub input: Type,
    pub output: Type,
    pub body: Node<'s>,
}

pub type Ident<'s> = &'s str;

#[derive(Clone, Debug)]
pub enum Node<'s> {
    /// E.g. `123`
    Const(Value<'s>),

    /// `name`
    Var(Ident<'s>),

    /// `expr.idx`
    ExtractTuple { expr: Box<Self>, idx: usize },

    /// `expr[idx]`
    ExtractArray { expr: Box<Self>, idx: Box<Self> },

    /// `expr as ty`
    Cast { expr: Box<Self>, ty: Type },

    /// `lhs + rhs`
    Add { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs - rhs`
    Sub { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs * rhs`
    Mul { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs / rhs`
    Div { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs % rhs`
    Mod { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs == rhs`
    Eq { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs != rhs`
    Neq { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs > rhs`
    Gt { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs >= rhs`
    GtEq { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs < rhs`
    Lt { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs <= rhs`
    LtEq { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs && rhs`
    And { lhs: Box<Self>, rhs: Box<Self> },

    /// `lhs || rhs`
    Or { lhs: Box<Self>, rhs: Box<Self> },

    /// `var name = value;`
    Declare { name: Ident<'s>, value: Box<Self> },

    /// `name = value`
    Assign { name: Ident<'s>, value: Box<Self> },

    /// `while cond { body }`
    While { cond: Box<Self>, body: Box<Self> },

    /// `print(node, node, ...)`
    Print(Vec<Self>),

    /// `{ node node ... }`
    Block(Vec<Self>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
    Char,
    Int,
    Float,
    Str,
    Tuple(Vec<Self>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'s> {
    Unit,
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f32),
    Str(&'s str),
    Tuple(Vec<Self>),
}

impl<'s> Value<'s> {
    pub fn print(&self) {
        match self {
            Value::Unit => print!("()"),
            Value::Bool(value) => print!("{}", value),
            Value::Char(value) => print!("{}", value),
            Value::Int(value) => print!("{}", value),
            Value::Float(value) => print!("{}", value),
            Value::Str(value) => print!("{}", value),

            Value::Tuple(values) => {
                print!("(");

                for value in values {
                    value.print();
                }

                print!(")");
            }
        }
    }

    pub fn ty(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::Char(_) => Type::Char,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Str(_) => Type::Str,
            Value::Tuple(values) => Type::Tuple(values.iter().map(|value| value.ty()).collect()),
        }
    }

    /// Converts value into given Rust type.
    ///
    /// If `self` is of different type than `T`, panics.
    pub fn unbox<T>(self) -> T
    where
        T: FromValue<'s>,
    {
        FromValue::from_value(self)
    }

    pub(crate) fn unbox_tuple(self) -> Vec<Self> {
        match self {
            Value::Tuple(values) => values,
            this => panic!("Got `{:?}`, expected a tuple", this),
        }
    }
}
