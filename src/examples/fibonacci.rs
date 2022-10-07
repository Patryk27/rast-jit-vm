use crate::ast::*;

/// Returns a program that returns given nth Fibonacci number.
///
/// It's RastJitVm's realization of:
///
/// ```
/// fn program(mut n: i64) -> i64 {
///     let mut x = 0;
///     let mut y = 1;
///     let mut z = 1;
///
///     while n > 0 {
///         x = y;
///         y = z;
///         z = x + y;
///         n -= 1;
///     }
///
///     x
/// }
/// ```
pub fn program() -> Program<'static> {
    Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Block(vec![
            // var x = 0
            Node::Declare {
                name: "x",
                value: Box::new(Node::Const(Value::Int(0))),
            },
            // var y = 1
            Node::Declare {
                name: "y",
                value: Box::new(Node::Const(Value::Int(1))),
            },
            // var z = 1
            Node::Declare {
                name: "z",
                value: Box::new(Node::Const(Value::Int(1))),
            },
            // var n = input
            Node::Declare {
                name: "n",
                value: Box::new(Node::Var("input")),
            },
            // while n > 0
            Node::While {
                cond: Box::new(Node::Gt {
                    lhs: Box::new(Node::Var("n")),
                    rhs: Box::new(Node::Const(Value::Int(0))),
                }),
                body: Box::new(Node::Block(vec![
                    // x = y
                    Node::Assign {
                        name: "x",
                        value: Box::new(Node::Var("y")),
                    },
                    // y = z
                    Node::Assign {
                        name: "y",
                        value: Box::new(Node::Var("z")),
                    },
                    // z = x + y
                    Node::Assign {
                        name: "z",
                        value: Box::new(Node::Add {
                            lhs: Box::new(Node::Var("x")),
                            rhs: Box::new(Node::Var("y")),
                        }),
                    },
                    // n = n - 1
                    Node::Assign {
                        name: "n",
                        value: Box::new(Node::Sub {
                            lhs: Box::new(Node::Var("n")),
                            rhs: Box::new(Node::Const(Value::Int(1))),
                        }),
                    },
                ])),
            },
            Node::Var("x"),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm;

    fn test_compile(input: i64, expected: i64) {
        let fun = vm::compile(program());
        let actual = fun(input);

        assert_eq!(expected, actual);
    }

    fn test_eval(input: i64, expected: i64) {
        let actual = vm::eval(&program(), input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test() {
        let cases = vec![
            // (input, expected)
            (0, 0),
            (1, 1),
            (2, 1),
            (3, 2),
            (50, 12_586_269_025),
        ];

        for (input, expected) in cases {
            test_compile(input, expected);
            test_eval(input, expected);
        }
    }
}
