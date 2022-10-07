use crate::ast::*;

pub trait FromValue<'s> {
    fn from_value(value: Value<'s>) -> Self;
    fn ty() -> Option<Type>;
}

impl<'s> FromValue<'s> for Value<'s> {
    fn from_value(value: Value<'s>) -> Self {
        value
    }

    fn ty() -> Option<Type> {
        None
    }
}

impl<'s> FromValue<'s> for () {
    fn from_value(value: Value<'s>) -> Self {
        if let Value::Unit = value {
            //
        } else {
            panic!(
                "Got `{:?}`, but expected value of type `{:?}`",
                value,
                Type::Unit,
            );
        }
    }

    fn ty() -> Option<Type> {
        Some(Type::Unit)
    }
}

macro_rules! from_value {
    ([ $( $variant:ident($param:ident) = $ty:ty => $expr:expr ),+ $(,)? ]) => {
        $(
            impl<'s> FromValue<'s> for $ty {
                fn from_value(value: Value<'s>) -> Self {
                    if let Value::$variant($param) = value {
                        $expr
                    } else {
                        panic!(
                            "Got `{:?}`, but expected value of type `{:?}`",
                            value,
                            Type::$variant,
                        );
                    }
                }

                fn ty() -> Option<Type> {
                    Some(Type::$variant)
                }
            }
        )+
    };
}

from_value!([
    Bool(value) = bool => value,
    Char(value) = char => value,
    Int(value) = i64 => value,
    Int(value) = i32 => value as _,
    Float(value) = f32 => value,
    Float(value) = f64 => value as _,
    Str(value) = &'s str => value,
]);
