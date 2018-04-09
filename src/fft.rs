
use num_complex::Complex;


use super::sac;

impl sac {
    pub fn fft(&self) -> Self {
        *self
    }
    /// Create a new file from complex data (real,imag)
    pub fn from_complex<T: AsRef<Vec<Complex<f32>>>>(y: T) -> sac {
        let mut s = sac_new();
        s.npts = y.as_ref().len() as i32;
        s.y = y.as_ref().iter().map(|&x| x.re).collect();
        s.x = y.as_ref().iter().map(|&x| x.im).collect();
        //s.nsnpts = s.npts;
        //s.iftype = SacFileType::RealImaginary;
        //s.sb = b;
        //s.b = 0.0;
        //s.delta = 1.0 / (s.delta * s->npts);
        //s.e = s.b + nfreq * s.delta;
        s
    }
}
