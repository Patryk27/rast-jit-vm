# rast-jit-vm

rast-jit-vm is a simplistic, proof-of-concept~ish compiler / virtual machine
that transforms syntax tree into a Rust function, type-checking it on the way:

``` rust
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
```

Instead of transforming AST into bytecode, rast-jit-vm uses a bit lesser known
technique that's oriented around thunks - basically, instead of doing:

``` rust
fn eval(node: Node) -> Value {
    match node {
        Node::Add { lhs, rhs } => lhs + rhs,
        /* ... */
    }
}
```

... what rast-jit-vm does is:

``` rust
type Thunk = Box<dyn Fn() -> Value>;

fn compile(node: Node) -> (Type, Thunk) {
    match node {
        Node::Add { lhs, rhs } => {
            let (lhs_ty, lhs) = compile(lhs);
            let (rhs_ty, rhs) = compile(rhs);
 
            match (lhs_ty, rhs_ty) {
                (Type::Int, Type::Int) => {
                    let ty = Type::Int;
                    let thunk = Box::new(|| lhs() + rhs());
                    
                    (ty, thunk)
                }
                
                (lhs_ty, rhs_th) => {
                    panic!("unsupported op: {:?} + {:?}", lhs_ty, rhs_ty);
                }
            }
        },
        
        /* ... */
    }
}
```

And while it might seem like a lot more work compared to just evaluating stuff,
the gain here comes from the fact that during the compilation we can e.g.
preallocate variables into stack-slots:

``` rust
struct CompilationContext {
    stack: Vec<Type>,
    vars: BTreeMap<String, usize>, // var-name => stack-slot
}

struct RuntimeContext {
    stack: Vec<Value>,
}

type Thunk = Box<dyn Fn(&mut RuntimeContext) -> Value>;

fn compile(ctxt: &mut CompilationContext, node: Node) -> (Type, Thunk) {
    match node {
        /// `let name = value`
        Node::Let { name, value } => {
            let (ty, value) = compile(ctxt, value);
            let id = ctxt.stack.len();

            ctxt.stack.push(ty);
            ctxt.vars.insert(name, ty);
            
            let thunk = Box::new(move |ctxt: &mut RuntimeContext| {
                // Voilà, no HashMap / BTreeMap needed at runtime!
                ctxt.stack[id] = value(ctxt);
            });
            
            (ty, thunk)
        },
        
        /// `name`
        Node::Var(name) => {
            let id = *vars.get(name).unwrap_or_else(|| {
                panic!("variable not defined: {}", name);
            });
            
            let thunk = Box::new(move |ctxt: &mut RuntimeContext| {
                // Voilà, no HashMap / BTreeMap needed at runtime!
                ctxt.stack[id].clone()
            });
            
            (ty, thunk)
        },
        
        /* ... */
    }
}
```

Thanks to this preallocation, during the runtime there's no need to use any
`var name => var value` map, since each variable has its own stack slot
determined at compile time.

## Running locally

``` shell
$ git clone https://github.com/Patryk27/rast-jit-vm
$ cd rast-jit-vm

# Uses the compilation technique as described above:
$ cargo run --bin mandelbrot-compile

# Uses just a regular interpreter-based evaluation approach:
$ cargo run --bin mandelbrot-eval

# Note that printing Mandelbrot fractal at this scale is pretty fast, so if you
# want to see a difference in the times, compare:
$ cargo run --release --bin mandelbrot-compile-100k
$ cargo run --release --bin mandelbrot-eval-100k 
```

## Thoughts

### Performance

Using the Mandelbrot fractal as a benchmark, tuned to 100k iterations for extra
synthetic-benchmarking-juice, I've been able to obtain the following times:

- pure Rust¹: 1.5s
- Python²: 115s
- rast-jit-vm, evaluated³: 130s
- **rast-jit-vm, compiled⁴: 50s**

_(technology: time to render the fractal; less is better)_

¹ `cd benches && rustc -O mandelbrot.rs && ./mandelbrot`    
² `cd benches && python3 ./mandelbrot.py`    
³ `cargo run --release --bin mandelbrot-eval-100k`    
⁴ `cargo run --release --bin mandelbrot-compile-100k`

### Missing features

- pretty error handling (it's just `panic!()` all over the place),
- both the compiler and the evaluator don't understand scoping - it's possible
  to declare a variable inside a `while` block and use it outside
  (to do something like `while false { let x = true; } println!("{}", x);`),
- rast-jit-vm's AST is Turing-incomplete (rough proof: all programs have
  statically allocated stack and there's no runtime malloc/free-like facilities,
  making recursion impossible; though arguably that's somewhere in-between a
  missing feature and a design decision).

## License

Copyright (c) 2022, Patryk Wychowaniec <pwychowaniec@pm.me>.    
Licensed under the MIT license.
