
extern crate sac;

use sac::Sac;
use sac::Math;
use sac::Calculus;

#[test]
fn test_integrate() {
    let s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
    //s.file = String::from("test_integrate");
    let s = s.int().unwrap();
    assert_eq!(s.y, [1.5, 4.]);
}
#[test]
fn test_differentiate() {
    let s = Sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "diff");
    let s = s.dif().unwrap();
    assert_eq!(s.y, [1.0, 1.0]);
}

#[test]
fn test_exp() {
    let mut s = Sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "test_exp");
    s.exp().unwrap();
    let v = vec![(1f32).exp(), (2f32).exp(), (3f32).exp()];
    assert_eq!(s.y, v);
}
