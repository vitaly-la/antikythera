use std::f64::consts::PI;

pub fn stereographic_projection(alt: f64, az: f64) -> (f64, f64) {
    let zenith_angle = alt + PI / 2.0;
    let r = zenith_angle.sin() / (1.0 - zenith_angle.cos());
    (r * az.sin(), r * az.cos())
}

pub fn circle_from_three_points(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> (f64, f64, f64) {
    let (ax, ay) = a;
    let (bx, by) = b;
    let (cx, cy) = c;
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    let x = ((ax * ax + ay * ay) * (by - cy) + (bx * bx + by * by) * (cy - ay) + (cx * cx + cy * cy) * (ay - by)) / d;
    let y = ((ax * ax + ay * ay) * (cx - bx) + (bx * bx + by * by) * (ax - cx) + (cx * cx + cy * cy) * (bx - ax)) / d;
    let r = ((ax - x) * (ax - x) + (ay - y) * (ay - y)).sqrt();
    (x, y, r)
}
