use rast_jit_vm::{ast::*, vm};

#[test]
fn test_const() {
    let prog = Program {
        input: Type::Unit,
        output: Type::Int,
        body: Node::Const(Value::Int(1234)),
    };

    assert_eq!(1234, vm::eval(&prog, ()));
    assert_eq!(1234, (vm::compile(prog))(()));
}

#[test]
fn test_var() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Var("input"),
    };

    assert_eq!(1234, vm::eval(&prog, 1234));
    assert_eq!(1234, (vm::compile(prog))(1234));
}

#[test]
fn test_extract_tuple() {
    let prog = Program {
        input: Type::Tuple(vec![Type::Str, Type::Int, Type::Float]),
        output: Type::Int,
        body: Node::ExtractTuple {
            expr: Box::new(Node::Var("input")),
            idx: 1,
        },
    };

    assert_eq!(123, vm::eval(&prog, ("Hello!", 123, 45.67)));
    assert_eq!(123, (vm::compile(prog))(("Hello!", 123, 45.67)));
}

#[test]
fn test_extract_array() {
    let prog = Program {
        input: Type::Str,
        output: Type::Char,
        body: Node::ExtractArray {
            expr: Box::new(Node::Var("input")),
            idx: Box::new(Node::Const(Value::Int(2))),
        },
    };

    assert_eq!('l', vm::eval(&prog, "Hello!"));
    assert_eq!('l', (vm::compile(prog))("Hello!"));
}

#[test]
fn test_cast_float_to_int() {
    let prog = Program {
        input: Type::Float,
        output: Type::Int,
        body: Node::Cast {
            expr: Box::new(Node::Var("input")),
            ty: Type::Int,
        },
    };

    assert_eq!(123, vm::eval(&prog, 123.45));
    assert_eq!(123, (vm::compile(prog))(123.45));
}

#[test]
fn test_cast_int_to_float() {
    let prog = Program {
        input: Type::Int,
        output: Type::Float,
        body: Node::Cast {
            expr: Box::new(Node::Var("input")),
            ty: Type::Float,
        },
    };

    assert_eq!(123.0f32, vm::eval(&prog, 123));
    assert_eq!(123.0f32, (vm::compile(prog))(123));
}

#[test]
fn test_add_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Add {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    assert_eq!(13, vm::eval(&prog, 3));
    assert_eq!(13, (vm::compile(prog))(3));
}

#[test]
fn test_add_float_float() {
    let prog = Program {
        input: Type::Float,
        output: Type::Float,
        body: Node::Add {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Float(10.0))),
        },
    };

    assert_eq!(13.0, vm::eval(&prog, 3.0));
    assert_eq!(13.0, (vm::compile(prog))(3.0));
}

#[test]
fn test_sub_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Sub {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    assert_eq!(-7, vm::eval(&prog, 3));
    assert_eq!(-7, (vm::compile(prog))(3));
}

#[test]
fn test_sub_float_float() {
    let prog = Program {
        input: Type::Float,
        output: Type::Float,
        body: Node::Sub {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Float(10.0))),
        },
    };

    assert_eq!(-7.0, vm::eval(&prog, 3.0));
    assert_eq!(-7.0, (vm::compile(prog))(3.0));
}

#[test]
fn test_mul_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Mul {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    assert_eq!(30, vm::eval(&prog, 3));
    assert_eq!(30, (vm::compile(prog))(3));
}

#[test]
fn test_mul_float_float() {
    let prog = Program {
        input: Type::Float,
        output: Type::Float,
        body: Node::Mul {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Float(10.0))),
        },
    };

    assert_eq!(30.0, vm::eval(&prog, 3.0));
    assert_eq!(30.0, (vm::compile(prog))(3.0));
}

#[test]
fn test_div_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Div {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    assert_eq!(3, vm::eval(&prog, 38));
    assert_eq!(3, (vm::compile(prog))(38));
}

#[test]
fn test_div_float_float() {
    let prog = Program {
        input: Type::Float,
        output: Type::Float,
        body: Node::Div {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Float(10.0))),
        },
    };

    assert_eq!(5.0, vm::eval(&prog, 50.0));
    assert_eq!(5.0, (vm::compile(prog))(50.0));
}

#[test]
fn test_mod_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Mod {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    assert_eq!(8, vm::eval(&prog, 38));
    assert_eq!(8, (vm::compile(prog))(38));
}

#[test]
fn test_eq_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Bool,
        body: Node::Eq {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    for n in 0..20 {
        let actual = n == 10;

        assert_eq!(vm::eval::<_, bool>(&prog, n), actual);
        assert_eq!((vm::compile::<_, bool>(prog.clone()))(n), actual);
    }
}

#[test]
fn test_gt_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Bool,
        body: Node::Gt {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    for n in 0..20 {
        let actual = n > 10;

        assert_eq!(vm::eval::<_, bool>(&prog, n), actual);
        assert_eq!((vm::compile::<_, bool>(prog.clone()))(n), actual);
    }
}

#[test]
fn test_gteq_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Bool,
        body: Node::GtEq {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    for n in 0..20 {
        let actual = n >= 10;

        assert_eq!(vm::eval::<_, bool>(&prog, n), actual);
        assert_eq!((vm::compile::<_, bool>(prog.clone()))(n), actual);
    }
}

#[test]
fn test_lt_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Bool,
        body: Node::Lt {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    for n in 0..20 {
        let actual = n < 10;

        assert_eq!(vm::eval::<_, bool>(&prog, n), actual);
        assert_eq!((vm::compile::<_, bool>(prog.clone()))(n), actual);
    }
}

#[test]
fn test_lteq_int_int() {
    let prog = Program {
        input: Type::Int,
        output: Type::Bool,
        body: Node::LtEq {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(10))),
        },
    };

    for n in 0..20 {
        let actual = n <= 10;

        assert_eq!(vm::eval::<_, bool>(&prog, n), actual);
        assert_eq!((vm::compile::<_, bool>(prog.clone()))(n), actual);
    }
}

#[test]
fn test_and_bool_bool() {
    let prog = Program {
        input: Type::Bool,
        output: Type::Bool,
        body: Node::And {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Bool(true))),
        },
    };

    assert!(vm::eval::<_, bool>(&prog, true));
    assert!(!vm::eval::<_, bool>(&prog, false));

    assert!((vm::compile::<_, bool>(prog.clone()))(true));
    assert!(!(vm::compile::<_, bool>(prog))(false));
}

#[test]
fn test_or_bool_bool() {
    let prog = Program {
        input: Type::Bool,
        output: Type::Bool,
        body: Node::Or {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Bool(true))),
        },
    };

    assert!(vm::eval::<_, bool>(&prog, true));
    assert!(vm::eval::<_, bool>(&prog, false));

    assert!((vm::compile::<_, bool>(prog.clone()))(true));
    assert!((vm::compile::<_, bool>(prog))(false));
}
