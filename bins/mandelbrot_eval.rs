use rast_jit_vm::prelude::*;

fn main() {
    vm::eval::<_, ()>(
        &rast_jit_vm::examples::mandelbrot(),
        (
            120,   // viewport_width
            60,    // viewport_height
            -2.05, // x1
            -1.12, // y1
            0.47,  // x2
            1.12,  // y2
            100,   // max_iterations
        ),
    );
}
