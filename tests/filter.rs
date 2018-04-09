
extern crate failure;
extern crate sac;

use failure::Error;

use sac::Sac;
use sac::Functions;
use sac::Filter;

fn filter() -> Result<(),Error> {
    let s1 = Sac::impulse(100, 0.0, 0.1);
    let s0  = Sac::read("tests/imp.sac")?;
    assert_eq!(s0, s1);

    let s = s1.bp(0.1, 0.5)?;
    let s0 = Sac::read("tests/imp_bp_0.1_0.5.sac")?;
    assert_eq!(s0, s);

    let s = s1.hp(0.5)?;
    let s0 = Sac::read("tests/imp_hp_0.5.sac")?;
    assert_eq!(s0, s);

    let s = s1.br(0.1, 0.5)?;
    let s0 = Sac::read("tests/imp_br_0.1_0.5.sac")?;
    assert_eq!(s0, s);

    let s = s1.lp(0.5)?;
    let s0 = Sac::read("tests/imp_lp_0.5.sac")?;
    assert_eq!(s0, s);

    Ok(())
}

#[test]
fn test_filter() {
    filter().unwrap();
}
