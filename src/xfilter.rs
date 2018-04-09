
use super::*;

use time::Time;

pub trait Filter : Time + Sized {
    fn bp(&self, flow: f64, fhigh: f64) -> Result<Self, Error>;
    fn br(&self, flow: f64, fhigh: f64) -> Result<Self, Error>;
    fn lp(&self, fc: f64) -> Result<Self, Error>;
    fn hp(&self, fc: f64) -> Result<Self, Error>;
}

fn into_v32(y: Vec<f64>) -> Vec<f32> {
    y.into_iter().map(|x| x as f32).collect()
}
fn to_v64(y: &[f32]) -> Vec<f64> {
    y.iter().map(|&x| x as f64).collect()
}

impl Filter for Sac {
    fn bp(&self, flow: f64, fhigh: f64) -> Result<Self, Error> {
        let mut y = to_v64(self.amp());
        filter::bp(&mut y, flow, fhigh, self.delta as f64);
        Ok( self.with_new_data( into_v32( y )) )
    }
    fn br(&self, flow: f64, fhigh: f64) -> Result<Self, Error> {
        let mut y = to_v64(self.amp());
        filter::br(&mut y, flow, fhigh, self.delta as f64);
        Ok( self.with_new_data( into_v32( y )) )
    }
    fn lp(&self, fc: f64) -> Result<Self, Error> {
        let mut y = to_v64(self.amp());
        filter::lp(&mut y, fc, self.delta as f64);
        Ok( self.with_new_data( into_v32( y )) )
    }
    fn hp(&self, fc: f64) -> Result<Self, Error> {
        let mut y = to_v64(self.amp());
        filter::hp(&mut y, fc, self.delta as f64);
        Ok( self.with_new_data( into_v32( y )) )
    }
}

