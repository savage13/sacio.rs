
use sacio::Sac;

#[test]
fn multiply_test() {
    let mut s = Sac::from_amp([0.,1.,2.].to_vec(),0.0,1.0);
    for y in &mut s.y {
        *y *= 2.0;
    }
    s.extrema();
    assert_eq!(s.max_amp(), 4.0);
    assert_eq!(s.min_amp(), 0.0);
}

#[test]
fn add_test() {
    let mut s = Sac::from_amp([0.,1.,2.].to_vec(),0.0,1.0);
    for y in &mut s.y {
        *y += 10.0;
    }
    s.extrema();
    assert_eq!(s.max_amp(), 12.0);
    assert_eq!(s.min_amp(), 10.0);
}

