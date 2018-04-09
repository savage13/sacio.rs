
use num_complex::Complex;
use failure::Error;

use fft::fft;
use super::*;

pub trait Spectral : Sized {
    fn fft(&self) -> Result<Self,Error>;
    fn amph(&mut self) -> Result<(),Error>;
    fn reim(&mut self) -> Result<(),Error>;
    fn mul_omega(&mut self) -> Result<(),Error>;
    fn div_omega(&mut self) -> Result<(),Error>;
    fn ifft(&self) -> Result<Self,Error>;
    fn analytic(&self) -> Result<(Self,Self), Error>;
}

impl Spectral for Sac {
    fn div_omega(&mut self) -> Result<(),Error> {
        unimplemented!("div_omega");
    }
    fn mul_omega(&mut self) -> Result<(),Error> {
        use std::f64::consts::PI;
        self.is_spectral()?;
        let nf = self.npts as usize/ 2;
        let dw = 2.0 * PI * self.delta as f64;
        if self.is_realimag() {
            for i in 0 .. nf-1 {
                let mut z = Complex::new(self.y[i] as f64,
                                         self.x[i] as f64);
                z *= Complex::new(0.0f64, dw * i as f64);
                self.y[i] =  z.re as f32;
                self.x[i] =  z.im as f32;
                let k = self.npts as usize - i;
                self.y[k] =  z.re as f32;
                self.x[k] = -z.im as f32;
            }
        } else if self.is_ampphase() {
            for i in 0 .. nf-1 {
                let mut z = Complex::from_polar(&(self.y[i] as f64),
                                                &(self.x[i] as f64));
                z *= Complex::new(0.0, dw * i as f64);
                self.y[i] =  z.norm() as f32;
                self.x[i] =  z.arg() as f32;
                let k = self.npts as usize - i;
                self.y[k] =  z.norm() as f32;
                self.x[k] = -z.arg() as f32;
            }
        }
        Ok(())
    }
    fn amph(&mut self) -> Result<(),Error> {
        self.is_spectral()?;
        if self.is_realimag() {
            for i in 0 .. self.y.len() {
                let z = Complex::new(self.y[i], self.x[i]);
                self.y[i] = z.norm();
                self.x[i] = z.arg();
            }
            self.iftype = SacFileType::AmpPhase.into();
            self.extrema_amp();
        }
        Ok(())
    }
    fn reim(&mut self) -> Result<(),Error> {
        self.is_spectral()?;
        if self.is_ampphase() {
            for i in 0 .. self.y.len() {
                let z = Complex::from_polar(&self.y[i], &self.x[i]);
                self.y[i] = z.re;
                self.x[i] = z.im;
            }
            self.iftype = SacFileType::RealImag.into();
            self.extrema_amp();
        }
        Ok(())

    }
    fn ifft(&self) -> Result<Self, Error> {
        let mut z : Vec<_> = match self.iftype.into() {
            SacFileType::RealImag => self.y.iter().zip(self.x.iter())
                .map(|(re,im)| (*re as f64, *im as f64))
                .map(|(re,im)| Complex{ re, im })
                .collect(),
            SacFileType::AmpPhase => self.y.iter().zip(self.x.iter())
                .map(|(re,im)| (*re as f64, *im as f64))
                .map(|(r,a)| (r * a.cos(), r * a.sin()) )
                .map(|(re,im)| Complex{ re, im })
                .collect(),
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::XYZ => return Err(NotSpectral.into()),
        };

        fft::ifft0(&mut z);

        let factor = 1.0/ self.sdelta as f64;
        for mut zi in z.iter_mut() {
            *zi = zi.scale(factor);
        }
        let y : Vec<_> = z.into_iter().map(|z| z.re as f32)
            .take(self.nsnpts as usize)
            .collect();

        let mut s = Sac::new();
        s.copy_header(&self);
        s.y = y;
        s.scale = 1.0 / (self.sdelta * self.npts as f32);
        s.npts  = s.nsnpts;
        s.b     = s.sb;
        s.delta = s.sdelta;

        s.iftype = SacFileType::Time.into();
        s.leven = true as i32;
        s.extrema();

        //s.nsnpts = SacIntUndef;

        Ok(s)
    }
    fn fft(&self) -> Result<Sac, Error> {
        let mut npts_new = 1usize;

        let mut s = Sac::new();
        s.copy_header(&self);

        /* Copy data to vec of next power of 2 */
        while npts_new < self.npts as usize {
            npts_new *= 2;
        }

        let mut z : Vec<_> = self.y.iter()
            .map(|&z| z)
            .map(|z| Complex::new(z as f64,0.0f64)).collect();
        /* Pad to next power of 2 */
        for _ in z.len() .. npts_new {
            z.push(Complex::new(0.,0.));
        }
        // Perform the FFT in place
        fft::fft0(&mut z);

        // Apply scale factor
        let factor = self.delta as f64;
        for mut zi in z.iter_mut() {
            *zi = zi.scale(factor);
        }

        let nfreq = npts_new as usize / 2;

        /* Seperate Real-Imaginary Components */
        s.y = z.iter().map(|z| z.re as f32).collect();
        s.x = z.iter().map(|z| z.im as f32).collect();

        // Meta data from original time series file
        s.nsnpts = self.npts;
        s.sb     = self.b;
        s.sdelta = self.delta;

        // Current File
        s.leven  = true as i32;
        s.npts   = s.y.len() as i32;
        s.b      = 0.0;
        s.delta  = 1.0 / (s.delta * s.npts as f32);
        s.e      = s.b + (s.delta * nfreq as f32);

        s.iftype = SacFileType::RealImag.into();
        s.iztype = SacZeroTime::None.into();

        s.extrema_amp();

        Ok(s)
    }
    fn analytic(&self) -> Result<(Self,Self), Error> {
        let mut z = fftn(&self.y, self.y.len());

        // Compute Analytic Signal using 2 * Step Function
        let n = z.len();
        let n2 = n / 2;
        if n % 2 == 0 {
            let m = n2;
            for i in 1   .. m { z[i] *= 2.0; }
            for i in m+1 .. n { z[i] *= 0.0; }
        } else {
            let m = (n+1)/2;
            for i in 1 .. m { z[i] *= 2.0; }
            for i in m .. n { z[i] *= 0.0; }
        }

        fft::ifft0(&mut z);

        let mut sx = Sac::new();
        sx.copy_header(&self);
        let mut sy = Sac::new();
        sy.copy_header(&self);

        /* Seperate Real-Imaginary Components */
        sx.y = z.iter().map(|z| z.re as f32).collect();
        sy.y = z.iter().map(|z| z.im as f32).collect();

        sx.y.truncate(sx.npts as usize);
        sy.y.truncate(sx.npts as usize);

        sx.extrema_amp();
        sy.extrema_amp();

        Ok((sx,sy))
    }

}

fn _next_power_of_two(n: usize) -> usize {
    let mut npts_new = 1usize;
    /* Copy data to vec of next power of 2 */
    while npts_new < n {
        npts_new *= 2;
    }
    npts_new
}

fn fftn(y: &[f32], n: usize) -> Vec<Complex<f64>> {
    let mut z : Vec<_> = y.iter()
        .map(|&z| Complex::new(z as f64, 0.0)).collect();
    let m = n-z.len();
    z.extend( vec![Complex::new(0.0, 0.0); m] );

    // Perform the FFT in place
    fft::fft0(&mut z);

    z
}

fn rclone(v: &[f32]) -> Vec<f32> {
    let mut v = v.to_vec();
    v.reverse();
    v
}

pub fn convolve_fft(a: &[f32], b: &[f32]) -> Vec<f32> {
    let n = a.len() + b.len() - 1;

    let af = fftn(a, n);
    let bf = fftn(b, n);

    let mut z : Vec<_> = af.into_iter().zip(bf.into_iter())
        .map(|(x,y)| x*y)
        .collect();

    fft::ifft0(&mut z);

    z.into_iter().map(|z| z.re as f32).collect()
}

pub fn sac_correlate_fft(a: &Sac, b: &Sac) -> Result<Sac, Error> {
    let y = convolve_fft(&rclone(&a.y), &b.y);

    // Create new Sac file
    let mut s = Sac::new();
    s.copy_header(&a);
    s.npts   = y.len() as i32;
    s.y      = y;
    s.iftype = SacFileType::Time.into();
    s.leven  = true as i32;
    s.b      = (1 - a.npts) as f32 * b.delta + b.b - a.b;
    s.extrema();
    Ok(s)
}
pub fn sac_convolve_fft(a: &Sac, b: &Sac) -> Result<Sac, Error> {
    let y = convolve_fft(&a.y, &b.y);

    // Create new Sac file
    let mut s = Sac::new();
    s.copy_header(&b);
    s.npts   = y.len() as i32;
    s.y      = y;
    s.iftype = SacFileType::Time.into();
    s.leven  = true as i32;
    s.extrema();
    Ok(s)
}
