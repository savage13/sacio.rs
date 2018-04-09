
extern crate sac;

use sac::Sac;
use sac::Spectral;
use sac::Functions;

#[test]
fn fft_test_sine() {
    // Input Sine
    let s = Sac::sine(100, 0.0, 1.0, 0.05, 0.0);
    let s0 = Sac::read("tests/sine.sac").unwrap();
    assert_eq!(s0, s);

    // fft(sine)
    let sf1 = s.fft().unwrap();
    let sf0 = Sac::read("tests/sine_fft_rlim.sac").unwrap();
    assert_eq!(sf0, sf1);

    // fft(sine sac)
    let sf2 = sf0.ifft().unwrap();
    let sf0 = Sac::read("tests/sine_fft_ifft.sac").unwrap();
    assert_eq!(sf0, sf2);
    // fft(ifft(sine))
    let sf3 = sf1.ifft().unwrap();
    assert_eq!(sf0, sf3);
}

#[test]
fn fft_test_imp() {
    // Input Impulse
    let s = Sac::impulse(100, 0.0, 0.1);
    let s0 = Sac::read("tests/imp.sac").unwrap();
    assert_eq!(s0, s);

    // fft() -> RealImag
    let sri = s.fft().unwrap();
    let s1 = Sac::read("tests/imp_fft_rlim.sac").unwrap();
    assert_eq!(s1, sri);

    // fft() -> AmpPhase
    let mut sf = s.fft().unwrap();
    sf.amph().unwrap();
    let s1 = Sac::read("tests/imp_fft_amph.sac").unwrap();
    assert_eq!(s1, sf);

    // fft() -> AmpPhase -> Real Imag
    sf.reim().unwrap();
    let s1 = Sac::read("tests/imp_fft_rlim.sac").unwrap();
    assert_eq!(s1, sf);

    // fft(ifft())
    let sfi = sf.ifft().unwrap();
    let sf0 = Sac::read("tests/imp_fft_ifft.sac").unwrap();
    assert_eq!(sf0, sfi);

    // ifft( sac (RealImag) )
    let s1 = Sac::read("tests/imp_fft_rlim.sac").unwrap();
    let s1i = s1.ifft().unwrap();
    assert_eq!(sf0, s1i);

    // ifft( sac (AmpPhase)
    let mut s1 = Sac::read("tests/imp_fft_rlim.sac").unwrap();
    s1.amph().unwrap();
    let s1i = s1.ifft().unwrap();
    assert_eq!(sf0, s1i);
}
