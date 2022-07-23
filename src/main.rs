use std::ops::Mul;

pub fn main() {}

fn sin(x: f64) -> (f64, f64) {
    (x.sin(), x.cos())
}

fn scale(x: f64, val: f64) -> (f64, f64) {
    (x * val, val)
}

fn sin_2x(x: f64) -> (f64, f64) {
    let (g_x, gp_x) = scale(x, 2.0);
    let (f_x, fp_g_x) = sin(g_x);
    let diff = fp_g_x * gp_x;
    (f_x, diff)
}

///sin(2*x) = cos(2*x)*2 = f'(g(x))*g'(x)
fn c_diff<F1, F2, D, V>(f: F1, g: F2) -> impl Fn(V) -> (V, D)
where
    D: Mul<Output = D>,
    F1: Fn(V) -> (V, D),
    F2: Fn(V) -> (V, D),
{
    move |x| {
        let (g_x, gp_x) = g(x);
        let (f_x, fp_g_x) = f(g_x);
        let diff = fp_g_x * gp_x;
        (f_x, diff)
    }
}

#[cfg(test)]
mod test {
    use nalgebra::SMatrix;

    use crate::{c_diff, scale, sin, sin_2x};

    #[test]
    fn test_diff() {
        let x = 0.56;
        let res = sin_2x(x);
        assert_eq!(res.0, (2.0 * x).sin());
        assert_eq!(res.1, (2.0 * x).cos() * 2.0);
    }

    #[test]
    fn test_c_diff() {
        let f = c_diff(sin, |x| scale(x, 2.0));
        let x = 0.0;
        let res = f(x);
        assert_eq!(res.0, (2.0 * x).sin(), "assert f value");
        assert_eq!(res.1, (2.0 * x).cos() * 2.0, "assert derivative");
    }

    fn test_mat() {
        let m = SMatrix::<f64, 2, 2>::default();
    }
}
