use std::cmp::{min, max};

pub fn get_line_pixels(num_pixels: usize, p0: (usize, usize), p1: (usize, usize)) -> Vec<usize> {
    let (x0, y0) = p0;
    let (x1, y1) = p1;

    if x0 == x1 { 
        return (min(y0, y1)..=max(y0, y1)).map(|yi| x0 + yi * num_pixels).collect();
    }

    if y0 == y1 {
        return (min(x0, x1)..=max(x0, x1)).map(|xi| xi + y0 * num_pixels).collect();
    }

    if y1.abs_diff(y0) < x1.abs_diff(x0) {
        if x0 > x1 {
            return plot_line_low(num_pixels, x1, y1, x0, y0);
        } else {
            return plot_line_low(num_pixels, x0, y0, x1, y1);
        }
    }

    else {
        if y0 > y1 {
            return plot_line_high(num_pixels, x1, y1, x0, y0);
        } else {
            return plot_line_high(num_pixels, x0, y0, x1, y1);
        }
    }
}

pub fn plot_line_low(num_pixels: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<usize> {

    let dx = x1 - x0;
    let dy = (y1 as isize - y0 as isize).abs() as usize;
    let mut out = Vec::new();

    #[allow(non_snake_case)]
    let mut D = 2 * dy as isize - dx as isize;
    let mut y = y0;

    for xi in x0..=x1 {
        out.push(xi + y * num_pixels);
        if D > 0 {
            y = if y1 > y0 { y + 1 } else { y.saturating_sub(1) };
            D -= 2 * dx as isize;
        }
        D += 2 * dy as isize;
    }

    out
}

pub fn plot_line_high(num_pixels: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<usize> {

    let dy = y1 - y0;
    let dx = (x1 as isize - x0 as isize).abs() as usize;
    let mut out = Vec::new();

    #[allow(non_snake_case)]
    let mut D = 2 * dx as isize - dy as isize;
    let mut x = x0;

    for yi in y0..=y1 {
        out.push(x + yi * num_pixels);
        if D > 0 {
            x = if x1 > x0 { x + 1 } else { x.saturating_sub(1) };
            D -= 2 * dy as isize;
        }
        D += 2 * dx as isize;
    }

    out
}

