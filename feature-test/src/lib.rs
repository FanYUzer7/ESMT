pub fn xy2d(n: i32, mut x: i32, mut y: i32) -> i32 {
    let mut rx = 0;
    let mut ry = 0;
    let mut s = n / 2;
    let mut d = 0;
    while s > 0 {
        rx = ((x & s) > 0) as i32;
        ry = ((y & s) > 0) as i32;
        d += s * s * ((3 * rx) ^ ry);
        rot(s, &mut x, &mut y, rx, ry);
        s /= 2;
    }
    println!("{},{}", rx, ry);
    d
}

fn rot(n: i32, x: &mut i32, y: &mut i32, rx: i32, ry: i32) {
    if ry == 0 {
        if rx == 1 {
            *x = n-1-(*x);
            *y = n-1-(*y);
        }
        std::mem::swap(x, y);
    }
}