
extern crate sac;

use sac::Sac;
use sac::Functions;
use sac::Calculus;

#[test]
fn tri_test() {
    let dt = 0.1;
    for w in 1 .. 10 {
        let mut t = Sac::triangle(w as f64, dt);
        let t = t.int().unwrap();
        let yf = t.y[t.y.len()-1];
        println!("yf: {}", yf);
        assert!((yf - 1.0).abs() < 1e-5)
    }
    let t = Sac::triangle(2.5, dt);
    let t0 = Sac::read("tests/tri.sac").unwrap();
    assert_eq!(t0,t);
}
