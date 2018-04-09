
extern crate sac;
use sac::Sac;
use sac::Ops;
use sac::Functions;

#[test]
#[should_panic]
fn hilbert() {
    let s = Sac::impulse(1000, 0.0, 0.1);
    let s0 = Sac::read("tests/imp1000.sac").unwrap();
    assert_eq!(s0,s);

    let mut s = s.hilbert().unwrap();
    let s0 = Sac::read("tests/hilbert_imp.sac").unwrap();
    s.write("tests/hilbert_imp_rs.sac").unwrap();
    assert_eq!(s0,s);
}

#[test]
#[should_panic]
fn envelope() {
    let s = Sac::impulse(1000, 0.0, 0.1);
    let s0 = Sac::read("tests/imp1000.sac").unwrap();
    assert_eq!(s0,s);

    let mut s = s.envelope().unwrap();
    let s0 = Sac::read("tests/envelope_imp.sac").unwrap();
    s.write("tests/envelope_imp_rs.sac").unwrap();
    assert_eq!(s0,s);
}
