use crate::ast::*;

pub trait IntoValue<'s> {
    fn into_value(self) -> Value<'s>;
    fn ty() -> Option<Type>;
}

impl<'s> IntoValue<'s> for Value<'s> {
    fn into_value(self) -> Value<'s> {
        self
    }

    fn ty() -> Option<Type> {
        None
    }
}

impl<'s> IntoValue<'s> for () {
    fn into_value(self) -> Value<'s> {
        Value::Unit
    }

    fn ty() -> Option<Type> {
        Some(Type::Unit)
    }
}

impl<'s> IntoValue<'s> for bool {
    fn into_value(self) -> Value<'s> {
        Value::Bool(self)
    }

    fn ty() -> Option<Type> {
        Some(Type::Bool)
    }
}

impl<'s> IntoValue<'s> for char {
    fn into_value(self) -> Value<'s> {
        Value::Char(self)
    }

    fn ty() -> Option<Type> {
        Some(Type::Char)
    }
}

impl<'s> IntoValue<'s> for i64 {
    fn into_value(self) -> Value<'s> {
        Value::Int(self)
    }

    fn ty() -> Option<Type> {
        Some(Type::Int)
    }
}

impl<'s> IntoValue<'s> for f32 {
    fn into_value(self) -> Value<'s> {
        Value::Float(self)
    }

    fn ty() -> Option<Type> {
        Some(Type::Float)
    }
}

impl<'s> IntoValue<'s> for &'s str {
    fn into_value(self) -> Value<'s> {
        Value::Str(self)
    }

    fn ty() -> Option<Type> {
        Some(Type::Str)
    }
}

macro_rules! into_value_tuple {
    ([ $( $tys:tt ),+ $(,)? ]) => {
        $( into_value_tuple!(@expand $tys); )*
    };

    (@expand ( $( $ty:ident ),+ )) => {
        impl<'s, $($ty,)+> IntoValue<'s> for ($($ty,)+)
        where
            $( $ty: IntoValue<'s>, )+
        {
            fn into_value(self) -> Value<'s> {
                Value::Tuple(vec![
                    $(
                        (into_value_tuple!(@get self[$ty])).into_value()
                    ),+
                ])
            }

            fn ty() -> Option<Type> {
                Some(Type::Tuple(vec![
                    $(
                        $ty::ty().expect(
                            "Tuples containing `Value`s are not supported - \
                             please specify concrete type"
                        )
                    ),+
                ]))
            }
        }
    };

    (@get $self:ident[T0]) => { $self.0 };
    (@get $self:ident[T1]) => { $self.1 };
    (@get $self:ident[T2]) => { $self.2 };
    (@get $self:ident[T3]) => { $self.3 };
    (@get $self:ident[T4]) => { $self.4 };
    (@get $self:ident[T5]) => { $self.5 };
    (@get $self:ident[T6]) => { $self.6 };
    (@get $self:ident[T7]) => { $self.7 };
    (@get $self:ident[T8]) => { $self.8 };
    (@get $self:ident[T9]) => { $self.9 };
}

into_value_tuple!([
    (T0),
    (T0, T1),
    (T0, T1, T2),
    (T0, T1, T2, T3),
    (T0, T1, T2, T3, T4),
    (T0, T1, T2, T3, T4, T5),
    (T0, T1, T2, T3, T4, T5, T6),
    (T0, T1, T2, T3, T4, T5, T6, T7),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
]);
