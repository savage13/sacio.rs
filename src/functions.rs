
use super::*;

pub trait Functions : Sized {
    fn impulse(n: usize, b: f64, dt: f64) -> Self;
    fn sine(n: usize, b: f64, dt: f64, frequency: f64, phase: f64) -> Self;
    fn triangle(half_width: f64, dt: f64) -> Self;
    // boxcar
    // trapezoid
    // line
    // quadratic
    // cubic
    // polynomial
    // random
    // impulse_string
    // seismogram
    // step
}

pub fn triangle_from_mag(mag: f64, dt: f64) -> Sac {
    // Wells and Coppersmith (1994), Table 2A (First group. All)
    let a = 5.08;
    let b = 1.16;
    let vr = 2.86; // km/s = Vs * 0.85 =  3.36 km/s * 0.85
    let length = (10.0f64).powf((mag - a) / b);
    let tr = length / vr;
    Sac::triangle(tr, dt)
}

impl Functions for Sac {
    fn triangle(half_width: f64, dt: f64) -> Sac {
        let n = (half_width * 2.0 / dt) as usize + 1;
        let h = half_width;
        let w = half_width * 2.0;
        let area = (h * w) / 2.0;
        let y : Vec<_> = (0..n).map(|i| i as f64)
            .map(|i| i * dt)
            .map(|t| if t < 0.0 {
                0.0
            } else if t <= half_width {
                t
            } else if t <= w {
                -t + 2.0 * h
            } else {
                0.0
            })
            .map(|v| v / area)
            .map(|v| v as f32)
            .collect();
        let mut s = Sac::from_amp(y, 0.0, dt);
        s.kevnm = format!("{:-16}", "FUNCGEN: TRIANGLE");
        s.kevnm.truncate(16);
        s
    }

    fn impulse(n: usize, b: f64, dt: f64) -> Sac {
        let mut y = vec![0.0f32; n];
        y[(n-1)/2] = 1.0;
        let mut s = Sac::from_amp(y, b, dt);
        s.kevnm = format!("{:-16}", "FUNCGEN: IMPULSE");
        s
    }

    fn sine(n: usize, b: f64, dt: f64, frequency: f64, phase: f64) -> Sac {
        use std::f64::consts::PI;
        let phase = 2.0 * PI * (frequency * (b as f64) + phase / 360.0);
        let y : Vec<_> = (0..n)
            .map(|i| i as f64)
            .map(|i| (phase + (2.0 * PI * i * frequency * dt as f64)).sin() )
            .map(|v| v as f32)
            .collect();
        let mut s = Sac::from_amp(y, b, dt);
        s.kevnm = format!("{:-16}", "FUNCGEN: SINE");
        s
    }
}
