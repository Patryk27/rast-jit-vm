fn program(
    viewport_width: i64,
    viewport_height: i64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    max_iterations: i64,
) {
    for viewport_y in 0..viewport_height {
        let y0 = y1 + (y2 - y1) * ((viewport_y as f32) / (viewport_height as f32));

        for viewport_x in 0..viewport_width {
            let x0 = x1 + (x2 - x1) * ((viewport_x as f32) / (viewport_width as f32));

            let mut x = 0.0;
            let mut y = 0.0;
            let mut iterations = max_iterations;

            while x * x + y * y <= 4.0 && iterations > 0 {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iterations -= 1;
            }

            let ch = "#%=-:,. "
                .chars()
                .nth((8.0 * ((iterations as f32) / (max_iterations as f32))) as _)
                .unwrap();

            print!("{}", ch);
        }

        println!();
    }
}

fn main() {
    program(120, 60, -2.05, -1.12, 0.47, 1.12, 100000);
}
