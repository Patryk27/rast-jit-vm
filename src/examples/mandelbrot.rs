use crate::ast::*;

/// Returns a program that prints the Mandelbrot fractal.
///
/// It's RastJitVm's realization of:
///
/// ```
/// fn program(
///     viewport_width: i64,
///     viewport_height: i64,
///     x1: f32,
///     y1: f32,
///     x2: f32,
///     y2: f32,
///     max_iterations: i64,
/// ) {
///     for viewport_y in 0..viewport_height {
///         let y0 = y1 + (y2 - y1) * ((viewport_y as f32) / (viewport_height as f32));
///
///         for viewport_x in 0..viewport_width {
///             let x0 = x1 + (x2 - x1) * ((viewport_x as f32) / (viewport_width as f32));
///
///             let mut x = 0.0;
///             let mut y = 0.0;
///             let mut iterations = max_iterations;
///
///             while x * x + y * y <= 4.0 && iterations > 0 {
///                 let xtemp = x * x - y * y + x0;
///                 y = 2.0 * x * y + y0;
///                 x = xtemp;
///                 iterations -= 1;
///             }
///
///             let ch = "#%=-:,. "
///                 .chars()
///                 .nth((8.0 * ((iterations as f32) / (max_iterations as f32))) as _)
///                 .unwrap();
///
///             print!("{}", ch);
///         }
///
///         println!();
///     }
/// }
/// ```
pub fn program() -> Program<'static> {
    Program {
        input: Type::Tuple(vec![
            Type::Int,   // viewport_width
            Type::Int,   // viewport_height
            Type::Float, // x1
            Type::Float, // y1
            Type::Float, // x2
            Type::Float, // y2
            Type::Int,   // max_iterations
        ]),
        output: Type::Unit,
        body: Node::Block(vec![
            // var viewport_width = input.0
            Node::Declare {
                name: "viewport_width",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 0,
                }),
            },
            // var viewport_height = input.1
            Node::Declare {
                name: "viewport_height",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 1,
                }),
            },
            // var x1 = input.2
            Node::Declare {
                name: "x1",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 2,
                }),
            },
            // var y1 = input.3
            Node::Declare {
                name: "y1",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 3,
                }),
            },
            // var x2 = input.4
            Node::Declare {
                name: "x2",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 4,
                }),
            },
            // var y2 = input.5
            Node::Declare {
                name: "y2",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 5,
                }),
            },
            // var max_iterations = input.6
            Node::Declare {
                name: "max_iterations",
                value: Box::new(Node::ExtractTuple {
                    expr: Box::new(Node::Var("input")),
                    idx: 6,
                }),
            },
            // var viewport_y = 0
            Node::Declare {
                name: "viewport_y",
                value: Box::new(Node::Const(Value::Int(0))),
            },
            // while viewport_y < viewport_height
            Node::While {
                cond: Box::new(Node::Lt {
                    lhs: Box::new(Node::Var("viewport_y")),
                    rhs: Box::new(Node::Var("viewport_height")),
                }),
                body: Box::new(Node::Block(vec![
                    // var y0 = y1 + (y2 - y1) * ((viewport_y as float) / (viewport_height as float))
                    Node::Declare {
                        name: "y0",
                        value: Box::new(Node::Add {
                            lhs: Box::new(Node::Var("y1")),
                            rhs: Box::new(Node::Mul {
                                lhs: Box::new(Node::Sub {
                                    lhs: Box::new(Node::Var("y2")),
                                    rhs: Box::new(Node::Var("y1")),
                                }),
                                rhs: Box::new(Node::Div {
                                    lhs: Box::new(Node::Cast {
                                        expr: Box::new(Node::Var("viewport_y")),
                                        ty: Type::Float,
                                    }),
                                    rhs: Box::new(Node::Cast {
                                        expr: Box::new(Node::Var("viewport_height")),
                                        ty: Type::Float,
                                    }),
                                }),
                            }),
                        }),
                    },
                    // var viewport_x = 0
                    Node::Declare {
                        name: "viewport_x",
                        value: Box::new(Node::Const(Value::Int(0))),
                    },
                    // while viewport_x < viewport_width
                    Node::While {
                        cond: Box::new(Node::Lt {
                            lhs: Box::new(Node::Var("viewport_x")),
                            rhs: Box::new(Node::Var("viewport_width")),
                        }),
                        body: Box::new(Node::Block(vec![
                            // var x0 = x1 + (x2 - x1) * ((viewport_x as float) / (viewport_width as float))
                            Node::Declare {
                                name: "x0",
                                value: Box::new(Node::Add {
                                    lhs: Box::new(Node::Var("x1")),
                                    rhs: Box::new(Node::Mul {
                                        lhs: Box::new(Node::Sub {
                                            lhs: Box::new(Node::Var("x2")),
                                            rhs: Box::new(Node::Var("x1")),
                                        }),
                                        rhs: Box::new(Node::Div {
                                            lhs: Box::new(Node::Cast {
                                                expr: Box::new(Node::Var("viewport_x")),
                                                ty: Type::Float,
                                            }),
                                            rhs: Box::new(Node::Cast {
                                                expr: Box::new(Node::Var("viewport_width")),
                                                ty: Type::Float,
                                            }),
                                        }),
                                    }),
                                }),
                            },
                            // var x = 0.0
                            Node::Declare {
                                name: "x",
                                value: Box::new(Node::Const(Value::Float(0.0))),
                            },
                            // var y = 0.0
                            Node::Declare {
                                name: "y",
                                value: Box::new(Node::Const(Value::Float(0.0))),
                            },
                            // var iterations = max_iterations
                            Node::Declare {
                                name: "iterations",
                                value: Box::new(Node::Var("max_iterations")),
                            },
                            // while (x*x + y*y <= 4.0) && (iterations > 0)
                            Node::While {
                                cond: Box::new(Node::And {
                                    lhs: Box::new(Node::LtEq {
                                        lhs: Box::new(Node::Add {
                                            lhs: Box::new(Node::Mul {
                                                lhs: Box::new(Node::Var("x")),
                                                rhs: Box::new(Node::Var("x")),
                                            }),
                                            rhs: Box::new(Node::Mul {
                                                lhs: Box::new(Node::Var("y")),
                                                rhs: Box::new(Node::Var("y")),
                                            }),
                                        }),
                                        rhs: Box::new(Node::Const(Value::Float(4.0))),
                                    }),
                                    rhs: Box::new(Node::Gt {
                                        lhs: Box::new(Node::Var("iterations")),
                                        rhs: Box::new(Node::Const(Value::Int(0))),
                                    }),
                                }),
                                body: Box::new(Node::Block(vec![
                                    // var xtemp = x*x - y*y + x0
                                    Node::Declare {
                                        name: "xtemp",
                                        value: Box::new(Node::Add {
                                            lhs: Box::new(Node::Sub {
                                                lhs: Box::new(Node::Mul {
                                                    lhs: Box::new(Node::Var("x")),
                                                    rhs: Box::new(Node::Var("x")),
                                                }),
                                                rhs: Box::new(Node::Mul {
                                                    lhs: Box::new(Node::Var("y")),
                                                    rhs: Box::new(Node::Var("y")),
                                                }),
                                            }),
                                            rhs: Box::new(Node::Var("x0")),
                                        }),
                                    },
                                    // y = 2.0 * x * y + y0
                                    Node::Assign {
                                        name: "y",
                                        value: Box::new(Node::Add {
                                            lhs: Box::new(Node::Mul {
                                                lhs: Box::new(Node::Const(Value::Float(2.0))),
                                                rhs: Box::new(Node::Mul {
                                                    lhs: Box::new(Node::Var("x")),
                                                    rhs: Box::new(Node::Var("y")),
                                                }),
                                            }),
                                            rhs: Box::new(Node::Var("y0")),
                                        }),
                                    },
                                    // x = xtemp
                                    Node::Assign {
                                        name: "x",
                                        value: Box::new(Node::Var("xtemp")),
                                    },
                                    // iterations = iterations - 1
                                    Node::Assign {
                                        name: "iterations",
                                        value: Box::new(Node::Sub {
                                            lhs: Box::new(Node::Var("iterations")),
                                            rhs: Box::new(Node::Const(Value::Int(1))),
                                        }),
                                    },
                                ])),
                            },
                            // print("#%=-:,. "[(8.0 * (iterations as float) / (max_iterations as float)) as int])
                            Node::Print(vec![Node::ExtractArray {
                                expr: Box::new(Node::Const(Value::Str("#%=-:,. "))),
                                idx: Box::new(Node::Cast {
                                    expr: Box::new(Node::Mul {
                                        lhs: Box::new(Node::Const(Value::Float(8.0))),
                                        rhs: Box::new(Node::Div {
                                            lhs: Box::new(Node::Cast {
                                                expr: Box::new(Node::Var("iterations")),
                                                ty: Type::Float,
                                            }),
                                            rhs: Box::new(Node::Cast {
                                                expr: Box::new(Node::Var("max_iterations")),
                                                ty: Type::Float,
                                            }),
                                        }),
                                    }),
                                    ty: Type::Int,
                                }),
                            }]),
                            // viewport_x = viewport_x + 1
                            Node::Assign {
                                name: "viewport_x",
                                value: Box::new(Node::Add {
                                    lhs: Box::new(Node::Var("viewport_x")),
                                    rhs: Box::new(Node::Const(Value::Int(1))),
                                }),
                            },
                        ])),
                    },
                    // print("\n")
                    Node::Print(vec![Node::Const(Value::Str("\n"))]),
                    // viewport_y = viewport_y + 1
                    Node::Assign {
                        name: "viewport_y",
                        value: Box::new(Node::Add {
                            lhs: Box::new(Node::Var("viewport_y")),
                            rhs: Box::new(Node::Const(Value::Int(1))),
                        }),
                    },
                ])),
            },
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm;
    use std::io;
    use std::sync::{Arc, Mutex};

    const ARGS: (i64, i64, f32, f32, f32, f32, i64) = (
        50,    // viewport_width
        20,    // viewport_height
        -2.05, // x1
        -1.12, // y1
        0.47,  // x2
        1.12,  // y2
        100,   // max_iterations
    );

    const EXPECTED: &str = "
                                            .
                                            ..
                                            .####
                                    .  , ..###..      .
                                    .###############.##.
                                --,#################%
                        .   :      =#####################-
                        .#######..######################.
                    . .%###############################,
        #############################################.
                    . .%###############################,
                        .#######..######################.
                        .   :      =#####################-
                                --,#################%
                                    .###############.##.
                                    .  , ..###..      .
                                            .####
                                            ..
                                            .
    ";

    /// Runs `f` and returns a string containing all `print!()` printed while
    /// that function was running.
    fn run(f: impl FnOnce()) -> String {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        // HACK: both the compiler & evaluator are hard-coded to use `print!()`
        // so using `set_output_capture()` is the easiest way of extracting the
        // printed strings.
        //
        // In theory we could provide some option to pass `&mut dyn Writer`, but
        // let's not go crazy.
        let old_buffer = io::set_output_capture(Some(Arc::clone(&buffer)));

        f();

        io::set_output_capture(old_buffer);

        let buffer = buffer.lock().unwrap().clone();

        String::from_utf8(buffer).unwrap()
    }

    #[track_caller]
    fn assert(actual: &str) {
        let expected_lines = EXPECTED.lines();
        let actual_lines = actual.lines();

        for (expected_line, actual_line) in expected_lines.zip(actual_lines) {
            if expected_line.trim() != actual_line.trim() {
                panic!(
                    "Expectation failed.\n\nExpected:\n{}\n\nActual:\n{}",
                    EXPECTED, actual
                );
            }
        }
    }

    #[test]
    fn test_compile() {
        let fun = vm::compile::<_, ()>(program());
        let actual = run(|| fun(ARGS));

        assert(&actual);
    }

    #[test]
    fn test_eval() {
        let actual = run(|| vm::eval::<_, ()>(&program(), ARGS));

        assert(&actual);
    }
}
