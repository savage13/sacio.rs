
use super::*;

use num_traits::Float;
use failure::Error;
use spec::sac_correlate_fft;
use spec::sac_convolve_fft;

// mark time
// mark value
// mark ptp
// rotate
// rq
// merge
// transfer
// hilbert
// hanning
// fir
// spectrogram

pub trait Time {
    fn is_time(&self) -> Result<(), Error>;
    fn amp(&self) -> &[f32];
}

impl Time for Sac {
    fn amp(&self) -> &[f32] {
        &self.y
    }
    fn is_time(&self) -> Result<(), Error> {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::XYZ  => Ok(()),
            SacFileType::AmpPhase |
            SacFileType::RealImag => Err(NotTime.into()),
        }
    }
}



pub trait Ops : Time + Sized {

    fn taper(&self, factor: f64, kind: Taper) -> Result<Self, Error>;
    fn rmean(&self) -> Result<Self, Error>;
    fn rtrend(&self) -> Result<Self, Error>;

    fn convolve(&self, other: &Self) -> Result<Self,Error>;
    fn correlate(&self, other: &Self) -> Result<Self,Error>;
    fn envelope(&self) -> Result<Self,Error>;
    fn hilbert(&self) -> Result<Self, Error>;

    fn decimate(&self, factor: usize) -> Result<Self, Error>;
    fn smooth(&self, half_width: usize) -> Result<Self, Error>;
    fn reverse(&self) -> Result<Self, Error>;

    fn stretch(&self, factor: f64) -> Result<Self, Error>;
    fn interpolate(&self, dt: f64) -> Result<Self, Error>;
    fn window(&self, b: f64, e: f64) -> Result<Window, Error>;
    //fn cut(&self, b: f64, t: f64) -> Result<Self, Error>;
}

use std::f64::consts::PI;
fn cos_taper(i: usize, nw: usize) -> f64 {
    let i = i as f64;
    let nw = nw as f64;
    (i * PI / (2.0 * nw)).sin()
}
fn han_taper(i: usize, nw: usize) -> f64 {
    let i = i as f64;
    let nw = nw as f64;
    0.50 - 0.50 * (i * PI/nw).cos()
}
fn ham_taper(i: usize, nw: usize) -> f64 {
    let i = i as f64;
    let nw = nw as f64;
    0.54 - 0.46 * (i * PI/nw).cos()
}

pub enum Taper {
    Cosine,
    Hanning,
    Hamming,
}

struct SacTime {
    i: usize,
    b: f64,
    dt: f64,
    n: usize,
}

fn time(s: &Sac) -> SacTime {
    SacTime {
        n: s.npts as usize,
        b: s.b as f64,
        dt: s.delta as f64,
        i: 0
    }
}

impl Iterator for SacTime {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        if self.i >= self.n {
            None
        } else {
            let ith = self.i as f64;
            self.i += 1;
            Some(self.b + self.dt * ith)
        }
    }
}
fn clamp64(v: f64, v0: f64, v1: f64) -> f64 {
    if v < v0 {
        v0 
    } else if v > v1 {
        v1
    } else {
        v
    }
}

#[derive(Debug)]
pub struct Window<'a> {
    inner: &'a Sac,
    n0: usize,
    n1: usize,
}
impl<'a> Time for Window<'a> {
    fn amp(&self) -> &[f32] {
        &self.inner.y[self.n0..self.n1+1]
    }
    fn is_time(&self) -> Result<(),Error> {
        self.inner.is_time()
    }
}

pub trait Calculus : Time + Sized {
    fn int(&self) -> Result<Self, Error>;
    fn dif(&self) -> Result<Self, Error>;
}

fn dif<T: Float>(y: &[T], dt: T) -> Vec<T> {
    let n = y.len()-1;
    (0..n).map(|i| dt.recip() * (y[i+1] - y[i])).collect()
}
fn int<T: Float>(y: &[T], dt: T) -> Vec<T>
    where f64: std::convert::From<T>
{
    let two = T::one() + T::one();
    let f = dt / two;
    let n = y.len()-1;
    let mut sum = T::zero();
    let mut out = vec![];
    for i in 0 .. n {
        sum = sum + ( f * (y[i] + y[i+1]) );
        out.push( sum );
    }
    out
}

impl Calculus for Sac {
    fn int(&self) -> Result<Sac, Error> {
        self.is_time()?;
        Ok(self.with_new_data(
            int(self.amp(), self.delta)
        ))
    }
    fn dif(&self) -> Result<Sac, Error> {
        self.is_time()?;
        Ok(self.with_new_data(
            dif(self.amp(), self.delta)
        ))
    }
}

pub trait RMS : Time {
    /// Compute Root Mean Square of a sequence
    fn rms(&self) -> Result<f64,Error> {
        self.is_time()?;
        let y = self.amp();
        let sqsum : f64 = y.iter().map(|&v| v as f64).map(|v| v*v).sum();
        let mean = sqsum / y.len() as f64;
        Ok( mean.sqrt() )
    }
}

impl RMS for Sac {}
impl<'a> RMS for Window<'a> {}


impl Ops for Sac {
    fn window(&self, t0: f64, t1: f64) -> Result<Window, Error> {
        if t0 < self.b.into() {
            bail!("Window start < data begin time {} < {}", t0, self.b);
        }
        if t0 > self.e.into() {
            bail!("Window start > data end time {} < {}", t0, self.e);
        }
        if t1 < self.b.into() {
            bail!("Window end < data begin time {} < {}", t1, self.b);
        }
        if t1 > self.e.into() {
            bail!("Window end > data end time {} < {}", t1, self.e);
        }
        let s = self;
        let (b,dt) = (s.b as f64, s.delta as f64);
        let n0 = (t0/dt).round() - (b/dt).round();
        let n1 = (t1/dt).round() - (b/dt).round();
        let n0 = clamp64(n0, 0.0, (s.npts - 1) as f64);
        let n1 = clamp64(n1, 0.0, (s.npts - 1) as f64);
        let n0 = n0 as usize;
        let n1 = n1 as usize;
        Ok(Window {inner: &self, n0, n1 })
    }
    fn interpolate(&self, _dt: f64) -> Result<Self, Error> { unimplemented!("interpolate"); }
    fn decimate(&self, _factor: usize) -> Result<Self, Error> { unimplemented!("decimate"); }
    fn rtrend(&self) -> Result<Self, Error> {
        // https://en.wikipedia.org/wiki/Ordinary_least_squares#Simple_regression_model
        // y_i = \alpha + \beta x_i + \epsilon
        // \hat{\beta} = \dfrac{\sum x_i y_i - 1/n \sum x_i \sum y_i}
        //                    {\sum x_i^2 - 1/n (\sum x_i)^2}
        // = Cov(x,y) / Var(x,x)
        // \hat{\alpha} = \bar{y} - \hat{\beta} \bar{x}
        let n = self.y.len() as f64;
        let sx : f64 = time(&self).sum();
        let sy : f64 = self.y.iter().map(|&y| y as f64).sum();
        let sxy : f64 = self.y.iter().zip(time(&self)).map(|(&y,t)| y as f64 * t).sum();
        let sx2 : f64 = time(&self).map(|t| t*t).sum();
        let slope = (sxy - sx*sy/n) / (sx2 - sx*sx/n);
        let inter = sy/n - slope * sx/n;
        let y : Vec<_> = self.y.iter().zip(time(&self))
            .map(|(&y,t)| y as f64 - (inter + t * slope))
            .map(|y| y as f32)
            .collect();
        println!("slope: {} intercept: {}", slope, inter);
        let mut s = self.clone();
        s.y = y;
        s.extrema();
        Ok(s)
    }
    fn convolve (&self, other: &Self) -> Result<Self,Error> {
        let c = sac_convolve_fft(self, other)?;
        Ok(c)
    }
    fn correlate(&self, other: &Self) -> Result<Self,Error> {
        let c = sac_correlate_fft(self, other)?;
        Ok(c)
    }

    fn stretch(&self, _factor: f64) -> Result<Self, Error> {
        unimplemented!("stretch");
    }

    fn envelope (&self) -> Result<Self,Error> {
        let (r,h) = self.analytic()?;
        let (a, b) = (r.y, h.y);
        let mut s = self.clone();
        s.y = (0..a.len())
            .map(|i| (a[i]*a[i]+b[i]*b[i]).sqrt())
            .collect();
        s.extrema();
        Ok(s)
    }

    fn hilbert(&self) -> Result<Self, Error> {
        let (_, s) = self.analytic()?;
        Ok(s)
    }

    fn taper(&self, width: f64, kind: Taper) -> Result<Self, Error> {
        let nw = (width * (self.npts + 1) as f64) as usize;
        let nw = std::cmp::max(nw,2);
        let n = self.y.len();
        let f = match kind {
            Taper::Cosine  => cos_taper,
            Taper::Hanning => han_taper,
            Taper::Hamming => ham_taper,
        };
        let mut s = self.clone();
        (0..nw).for_each(|i| s.y[i]     *= f(i,nw) as f32);
        (0..nw).for_each(|i| s.y[n-i-1] *= f(i,nw) as f32);
        s.extrema();
        Ok(s)

    }
    fn smooth(&self, w: usize) -> Result<Self, Error> {
        let use_mean = true;
        let use_median = ! use_mean;
        self.is_time()?;
        let mut y = vec![];
        let n = ((2 * w) + 1) as f64;
        for i in 0 .. self.y.len() {
            if i >= w && i + w < self.y.len() {
                // Mean
                if use_mean {
                    let v : f64 = (i-w .. i+w+1)
                        .map(|j| self.y[j] as f64)
                        .sum();
                    y.push( v / n );
                }
                if use_median {
                    // Median
                    let mut v : Vec<_> = (i-w .. i+w+1)
                        .map(|j| R64::new(self.y[j] as f64))
                        .collect::<Result<Vec<_>,_>>()?;
                    v.sort();
                    let v : Vec<_> = v.into_iter().map(|v| v.into()).collect();
                    let n = v.len() / 2;
                    if n == 1 {
                        y.push( v[0] )
                    } else if n % 2 != 0 {
                        y.push( v[n/2] );
                    } else {
                        y.push( (v[n/2]+v[n/2-1])/2.0 );
                    }
                }
            }
        }
        let y : Vec<_> = y.into_iter().map(|v| v as f32).collect();
        let mut s = self.clone();
        s.npts = y.len() as i32;
        s.y = y;
        s.b = s.b + s.delta * w as f32;
        s.extrema();
        Ok(s)
    }

    fn rmean(&self) -> Result<Self, Error> {
        self.is_time()?;
        let mut s = self.clone();
        let n = self.y.len() as f64;
        let sy : f64 = s.y.iter().map(|&v| v as f64).sum();
        let mean = sy / n;
        s.y.iter_mut().for_each(|y| *y = *y - mean as f32);
        s.extrema();
        Ok(s)
    }

    fn reverse(&self) -> Result<Self, Error> {
        let mut s = self.clone();
        s.y.reverse();
        Ok(s)
    }

}

pub trait Math : Sized {
    fn sqr(&mut self) -> Result<(), Error>;
    fn sqrt(&mut self) -> Result<(), Error>;
    fn abs(&mut self) -> Result<(), Error>;
    fn log(&mut self) -> Result<(), Error>;
    fn log10(&mut self) -> Result<(), Error>;
    fn exp(&mut self) -> Result<(), Error>;
    fn exp10(&mut self) -> Result<(), Error>;
    fn add(&mut self, v: f64) -> Result<(), Error>;
    fn sub(&mut self, v: f64) -> Result<(), Error>;
    fn mul(&mut self, v: f64) -> Result<(), Error>;
    fn div(&mut self, v: f64) -> Result<(), Error>;
    fn norm(&mut self) -> Result<(), Error>;
}

impl Math for Sac {
    /// Compute exp() of all data points
    fn exp(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = v.exp());
        self.extrema_amp();
        Ok(())
    }
    fn exp10(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = (10.0f32).powf(*v));
        self.extrema_amp();
        Ok(())
    }
    fn log(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = v.log(2.0));
        self.extrema_amp();
        Ok(())
    }
    fn log10(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = v.log(10.0));
        self.extrema_amp();
        Ok(())
    }
    fn abs(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = v.abs());
        self.extrema_amp();
        Ok(())
    }
    fn sqr(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = *v * *v);
        self.extrema_amp();
        Ok(())
    }
    fn sqrt(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = v.sqrt());
        self.extrema_amp();
        Ok(())
    }
    fn add(&mut self, x: f64) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = *v + x as f32);
        self.extrema_amp();
        Ok(())
    }
    fn sub(&mut self, x: f64) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = *v - x as f32);
        self.extrema_amp();
        Ok(())
    }
    fn mul(&mut self, x: f64) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = *v * x as f32);
        self.extrema_amp();
        Ok(())
    }
    fn div(&mut self, x: f64) -> Result<(),Error> {
        //self.is_time()?;
        self.y.iter_mut().for_each(|v| *v = *v / x as f32);
        self.extrema_amp();
        Ok(())
    }
    fn norm(&mut self) -> Result<(),Error> {
        //self.is_time()?;
        let v = if self.depmin.abs() > self.depmax.abs() {
            self.depmin.abs()
        } else {
            self.depmax.abs()
        };
        self.div( v as f64 )
    }

}

use std::cmp::Ordering;
#[derive(PartialEq,PartialOrd)]
struct R64(f64);
impl R64 {
    fn new(val: f64) -> Result<R64,Error> {
        if val.is_nan() {
            Err(NaN.into())
        } else {
            Ok(R64(val))
        }
    }
}

impl From<R64> for f64 {
    fn from(v: R64) -> f64 {
        v.0
    }
}

impl Eq for R64 {}

impl Ord for R64 {
    fn cmp(&self, other: &R64) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
