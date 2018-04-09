
extern crate failure;
use failure::Error;

extern crate sac;
use sac::Sac;
use sac::Ops;
use sac::Functions;
use sac::Taper;
use sac::RMS;

#[test]
fn correlate() {
    correlate1().unwrap();
}
#[test]
fn convolve() {
    convolve1().unwrap();
}
#[test]
fn rtrend() {
    rtrend1().unwrap();
}
#[test]
fn rmean() {
    rmean1().unwrap();
}
#[test]
fn taper() {
    taper1().unwrap();
}
#[test]
fn reverse() {
    reverse1().unwrap();
}
#[test]
fn rms() {
    rms1().unwrap();
}


fn correlate1() -> Result<(),Error> {
    let a = Sac::impulse(100, 0.0, 0.1);
    let b = a.clone();

    let mut c = a.correlate(&b)?;
    let c0 = Sac::read("tests/correlate_imp.sac")?;
    c.write("tests/correlate_imp_rs.sac")?;
    assert_eq!(c0, c);

    println!("------------------------------------");
    let r1 = Sac::read("tests/rand1.sac")?;
    let r2 = Sac::read("tests/rand2.sac")?;
    let mut c = r1.correlate(&r2)?;
    c.user0 = 2.0;
    let c0 = Sac::read("tests/correlate_rand.sac")?;
    c.write("tests/correlate_rand_rs.sac")?;
    assert_eq!(c0,c);

    println!("------------------------------------");
    let r1 = Sac::read("tests/rand1b.sac")?;
    let r2 = Sac::read("tests/rand2b.sac")?;
    let mut c = r1.correlate(&r2)?;
    c.user0 = 2.0;
    let c0 = Sac::read("tests/correlate_rand_b.sac")?;
    c.write("tests/correlate_rand_b_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}

fn convolve1() -> Result<(),Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/rand1.sac")?;
    let r2 = Sac::read("tests/boxcar.sac")?;
    let mut c = r1.convolve(&r2)?;
    let c0 = Sac::read("tests/convolve_boxcar.sac")?;
    c.write("tests/convolve_boxcar_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}

fn rtrend1() -> Result<(),Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/rand1.sac")?;
    let mut c = r1.rtrend()?;
    let c0 = Sac::read("tests/rand1_rtr.sac")?;
    c.write("tests/rand1_rtr_rs.sac")?;
    assert_eq!(c0,c);

    println!("------------------------------------");
    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.rtrend()?;
    let c0 = Sac::read("tests/seismo_rtr.sac")?;
    c.write("tests/seismo_rtr_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}

fn rmean1() -> Result<(), Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/rand1.sac")?;
    let mut c = r1.rmean()?;
    let c0 = Sac::read("tests/rand1_rmean.sac")?;
    c.write("tests/rand1_rmean_rs.sac")?;
    assert_eq!(c0,c);

    println!("------------------------------------");
    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.rmean()?;
    let c0 = Sac::read("tests/seismo_rmean.sac")?;
    c.write("tests/seismo_rmean_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}

fn taper1() -> Result<(),Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.taper(0.05, Taper::Hanning)?;
    let c0 = Sac::read("tests/seismo_taper_han.sac")?;
    c.write("tests/seismo_taper_han_rs.sac")?;
    assert_eq!(c0,c);

    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.taper(0.05, Taper::Hamming)?;
    let c0 = Sac::read("tests/seismo_taper_ham.sac")?;
    c.write("tests/seismo_taper_ham_rs.sac")?;
    assert_eq!(c0,c);

    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.taper(0.05, Taper::Cosine)?;
    let c0 = Sac::read("tests/seismo_taper_cos.sac")?;
    c.write("tests/seismo_taper_cos_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}

fn reverse1() -> Result<(),Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/seismo.sac")?;
    let mut c = r1.reverse()?;
    let c0 = Sac::read("tests/seismo_reverse.sac")?;
    c.write("tests/seismo_taper_reverse_rs.sac")?;
    assert_eq!(c0,c);
    Ok(())
}


fn rms1() -> Result<(),Error> {
    println!("------------------------------------");
    let r1 = Sac::read("tests/seismo.sac")?;
    let v = r1.rms()?;
    assert_eq!(v, 0.33504116717806764);

    let r2 = r1.window(10.0, 12.0)?;
    let v = r2.rms()?;
    assert_eq!(v, 0.4782715153412428);

    let r2 = r1.window(10.0, 13.0)?;
    let v = r2.rms()?;
    assert_eq!(v, 0.5704636154547597);
    Ok(())

}
