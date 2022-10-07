viewport_width = 120
viewport_height = 60
x1 = -2.05
y1 = -1.12
x2 = 0.47
y2 = 1.12
max_iterations = 100000

for viewport_y in range(viewport_height):
    y0 = y1 + (y2 - y1) * (float(viewport_y) / float(viewport_height))

    for viewport_x in range(viewport_width):
        x0 = x1 + (x2 - x1) * (float(viewport_x) / float(viewport_width))

        x = 0.0
        y = 0.0
        iterations = max_iterations

        while x * x + y * y <= 4.0 and iterations > 0:
            xtemp = x * x - y * y + x0
            y = 2.0 * x * y + y0
            x = xtemp
            iterations -= 1

        print("#%=-:,. "[int(8.0 * float(iterations) / float(max_iterations))], end="")

    print()
