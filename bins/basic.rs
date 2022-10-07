use rast_jit_vm::prelude::*;

fn main() {
    let mul2 = Program {
        input: Type::Int,
        output: Type::Int,
        body: Node::Mul {
            lhs: Box::new(Node::Var("input")),
            rhs: Box::new(Node::Const(Value::Int(2))),
        },
    };

    let mul2 = vm::compile::<_, i32>(mul2);

    println!("{}", mul2(15));
}
