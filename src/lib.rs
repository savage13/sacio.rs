
//! Library for reading and writing SAC files
//!
//! SAC - Seismic Analysis Code
//!
//! Reference: http://ds.iris.edu/files/sac-manual/
//!

extern crate fft;
extern crate num_complex;
extern crate num_traits;
#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
extern crate byteorder;

extern crate filter;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::io::prelude::*;

use failure::Error;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt, ReadBytesExt, NativeEndian};

mod enums;
use enums::*;

pub mod spec;
pub use spec::Spectral;
pub mod functions;
pub use functions::Functions;
pub mod time;
pub use time::Ops;
pub use time::Math;
pub use time::Taper;
pub use time::RMS;
pub use time::Calculus;
pub mod xfilter;
pub use xfilter::Filter;

#[cfg(target_endian = "big")]
type NonNativeEndian = LittleEndian;
#[cfg(target_endian = "big")]
type __NativeEndian = BigEndian;

#[cfg(target_endian = "little")]
type NonNativeEndian = BigEndian;
#[cfg(target_endian = "little")]
type __NativeEndian = LittleEndian;


macro_rules! sac_reals {
    ($s:ident, $function:ident) => { sac_reals!($s, ignore_idnet, ignore_type, $function); };
    ($s:ident, $z:ident, $function:ident) => { sac_reals!($s, $z, ignore_type, $function); };
    ($s:ident, $z:ident, $t:ty, $function:ident) => {
        $function!($s,$z,$t,
                   delta, depmin, depmax, scale, odelta, b, e, o, a, fmt,
                   t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, f,
                   resp0, resp1, resp2, resp3, resp4,
                   resp5, resp6, resp7, resp8, resp9,
                   stla, stlo, stel, stdp, evla, evlo, evel, evdp, mag,
                   user0, user1, user2, user3, user4,
                   user5, user6, user7, user8, user9,
                   dist, az, baz, gcarc, sb, sdelta,
                   depmen, cmpaz, cmpinc,
                   xminimum, xmaximum, yminimum, ymaximum,
                   unused6, unused7, unused8, unused9, unused10,
                   unused11, unused12
        );
    }
}


macro_rules! sac_ints {
    ($s:ident, $function:ident) => { sac_ints!($s, ignore_idnet, ignore_type, $function); };
    ($s:ident, $z:ident, $function:ident) => { sac_ints!($s, $z, ignore_type, $function); };
    ($s:ident, $z:ident, $t:ty, $function:ident) => {
        $function!($s,$z,$t,
                   nzyear, nzjday, nzhour, nzmin, nzsec, nzmsec, nvhdr,
                   norid, nevid, npts, nsnpts, nwfid,
                   nxsize, nysize, unused15, iftype, idep, iztype,
                   unused16, iinst, istreg, ievreg, ievtyp,
                   iqual, isynth, imagtyp, imagsrc,
                   unused19, unused20, unused21, unused22,
                   unused23, unused24, unused25, unused26,
                   leven, lpspol, lovrok, lcalda, unused27);

    };
}

macro_rules! sac_strings {
    ($s:ident, $function:ident) => { sac_strings!($s, ignore_ident, $function); };
    ($s:ident, $x:ident, $function:ident) => {
        $function!($s,$x,
                   kstnm, kevnm, khole, ko, ka,
                   kt0, kt1, kt2, kt3, kt4, kt5, kt6, kt7, kt8, kt9,
                   kf, kuser0, kuser1, kuser2, kcmpnm, knetwk, kdatrd, kinst);
    }
}

macro_rules! sac_u8_strings {
    ($s:ident, $function:ident) => { sac_u8_strings!($s, ignore_ident, $function); };
    ($s:ident, $z:ident, $function:ident) => {
        $function!($s,$z,
                   u8_kstnm, u8_kevnm, u8_khole, u8_ko, u8_ka,
                   u8_kt0, u8_kt1, u8_kt2, u8_kt3, u8_kt4,
                   u8_kt5, u8_kt6, u8_kt7, u8_kt8, u8_kt9,
                   u8_kf, u8_kuser0, u8_kuser1, u8_kuser2, u8_kcmpnm,
                   u8_knetwk, u8_kdatrd, u8_kinst);
    }
}


macro_rules! string_to_u8 {
    ($s:ident, $($a:ident, $b:ident),*) => {
        $(
            let mut tmp = if $s.$b.len() == 8 {
                format!("{:8}", $s.$a)
            } else {
                format!("{:16}", $s.$a)
            };
            tmp.truncate($s.$b.len());
            if tmp.trim_right().len() == 0 {
                tmp = format!("{:8}", "-12345");
            }
            $s.$b.copy_from_slice( tmp.as_bytes() );
        )*
    }
}

macro_rules! sac_strings_pair {
    ($s:ident, $function:ident) => {
        $function!($s,
                   kstnm, u8_kstnm, kevnm, u8_kevnm, khole, u8_khole,
                   ko, u8_ko, ka, u8_ka,
                   kt0, u8_kt0, kt1, u8_kt1, kt2, u8_kt2, kt3, u8_kt3, kt4, u8_kt4,
                   kt5, u8_kt5, kt6, u8_kt6, kt7, u8_kt7, kt8, u8_kt8, kt9, u8_kt9,
                   kf, u8_kf,
                   kuser0, u8_kuser0, kuser1, u8_kuser1, kuser2, u8_kuser2,
                   kcmpnm, u8_kcmpnm, knetwk,  u8_knetwk, kdatrd, u8_kdatrd,
                   kinst, u8_kinst
        );
    }
}

macro_rules! copy_values {
    ($s:ident, $from:ident, $t:ty, $($x:ident),*) => ( $($s.$x = $from.$x;  )* );
}
macro_rules! clone_values {
    ($s:ident, $from:ident, $($x:ident),*) => ( $($s.$x = $from.$x.clone();  )* );
}
macro_rules! write_real {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $fp.write_f32::<$t>($s.$x)?; );
}
macro_rules! write_reals {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( write_real!($s,$fp,$t,$x); )+ );
}
macro_rules! write_int {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $fp.write_i32::<$t>($s.$x)?; );
}
macro_rules! write_ints {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( write_int!($s,$fp,$t,$x); )+ );
}
macro_rules! read_real {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $s.$x = $fp.read_f32::<$t>()?; );
}
macro_rules! read_reals {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( read_real!($s,$fp,$t,$x); )+ );
}
macro_rules! read_int {
    ($s:ident, $fp:ident, $t:ty, $x:ident) => ( $s.$x = $fp.read_i32::<$t>()?; );
}
macro_rules! read_ints {
    ($s:ident, $fp:ident, $t:ty, $($x:ident),+) => ( $( read_int!($s,$fp,$t,$x); )+ );
}
macro_rules! read_strings {
    ($s:ident, $fp:ident, $($x:ident),+) => ( $( $fp.read_exact(&mut $s.$x)?; )+ );
}
macro_rules! write_strings {
    ($s:ident, $fp:ident, $($x:ident),+) => ( $( $fp.write_all(&$s.$x)?; )+ );
}

macro_rules! u8_to_string {
    ($s:ident, $($x:ident, $u8x:ident),*) => (
        $( $s.$x = String::from_utf8($s.$u8x.to_vec()).unwrap(); )*
    );
}

const SAC_INT_UNDEF : i32 = -12345;
const SAC_FLOAT_UNDEF : f32 = -12345.0;

macro_rules! f32_undef {
    ($s:ident, $q:ident, $t:ty, $($x:ident),*) => ( $( $s.$x = SAC_FLOAT_UNDEF; )* );
}
macro_rules! i32_undef {
    ($s:ident, $q:ident, $t:ty, $($x:ident),*) => ( $( $s.$x = SAC_INT_UNDEF; )* );
}
macro_rules! str_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $( $s.$x = String::from("-12345  "); )* );
}
macro_rules! u8s_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $(
        for i in 0 .. $s.$x.len() {
            $s.$x[i] = 32;
        }
        $s.$x[0] = 45;
        $s.$x[1] = 49;
        $s.$x[2] = 50;
        $s.$x[3] = 51;
        $s.$x[4] = 52;
        $s.$x[5] = 53;
    )* );
}

fn sac_u8_to_strings(s: &mut Sac) {
    sac_strings_pair!(s, u8_to_string);
}

fn sac_strings_to_u8(s: &mut Sac) {
    sac_strings_pair!(s, string_to_u8);
}

fn sac_data_read_comp<T: Read>(file: &mut T, swap: bool, npts: usize) -> Result<Vec<f32>,Error>{
    let mut y = vec![0.0; npts];
    if swap {
        file.read_f32_into::<NonNativeEndian>(&mut y)?;
    } else {
        file.read_f32_into::<NativeEndian>(&mut y)?;
    }
    Ok(y)
}
fn sac_data_read<T: Read>(file: &mut T, h: &mut Sac) -> Result<(),Error>{
    let npts = h.npts as usize;
    h.y = sac_data_read_comp(file, h.swap, npts)?;
    if h.ncomps() == 2 {
        h.x = sac_data_read_comp(file, h.swap, npts)?;
    }
    Ok(())
}

fn sac_data_write<F: Write>(file: &mut F, s: &mut Sac, npts: usize) -> Result<(),Error> {
    if npts != s.y.len() {
        panic!("Inconsistent Data: npts [{}] != data len [{}]", npts, s.y.len());
    }
    if s.swap {
        s.y.iter().map(|&y| file.write_f32::<NonNativeEndian>(y))
            .collect::<Result<Vec<()>,_>>()?;
    } else {
        s.y.iter().map(|&y| file.write_f32::<NativeEndian>(y))
            .collect::<Result<Vec<()>,_>>()?;
    }
    Ok(())
}

macro_rules! ri32 {
    ($file: expr, $t: tt) => ( $file.read_i32::<$t>()? )
}

fn sac_header_is_swapped<T: Read + Seek>(file: &mut T) -> Result<bool,Error> {
    use std::io::SeekFrom;
    file.seek(SeekFrom::Start(70*4 + 6*4))?;
    let n = ri32!(file, NativeEndian);

    let swap = if n > 5 && n <= 8 {
        false
    } else {
        file.seek(SeekFrom::Start(70*4 + 6*4))?;
        let n = ri32!(file, NonNativeEndian);
        if n < 0 || n > 10 {
            panic!("Unknown file type: {}", n);
        }
        true
    };
    file.seek(SeekFrom::Start(0))?;

    Ok(swap)
}
//const HEADER_SIZE : usize = 632;


fn sac_header_read<T: Read + Seek>(file: &mut T, h: &mut Sac) -> Result<(),Error>{
    use std::io::SeekFrom;

    h.swap = sac_header_is_swapped(file)?;
    file.seek(SeekFrom::Start(0))?;

    if h.swap {
        sac_reals!(h, file, NonNativeEndian, read_reals );
        sac_ints!(h, file, NonNativeEndian, read_ints);
    } else {
        sac_reals!(h, file, NativeEndian, read_reals );
        sac_ints!(h, file, NativeEndian, read_ints);
    }
    sac_u8_strings!(h, file, read_strings);
    Ok(())
}

fn sac_header_write<F: Write>(file: &mut F, s: &mut Sac) -> Result<(),Error>{

    if s.swap {
        sac_reals!(s, file, NonNativeEndian, write_reals );
        sac_ints!(s, file, NonNativeEndian, write_ints);
    } else {
        sac_reals!(s, file, NativeEndian, write_reals );
        sac_ints!(s, file, NativeEndian, write_ints);
    }
    sac_u8_strings!(s, file, write_strings);
    Ok(())
}

macro_rules! xeq {
    ($a:ident,$b:ident,$t:ty,$($x:ident),*) => {
        $( if $a.$x != $b.$x {
            println!("field {}:  {} != {}", stringify!($x),$a.$x, $b.$x);
            return false;
        } )*
    };
}
macro_rules! xeqf {
    ($a:ident,$b:ident,$t:ty,$($x:ident),*) => {
        $( if ($a.$x - $b.$x).abs() > 1e-5 {
            let dx = ($a.$x - $b.$x).abs();
            println!("field {}: {} != {} [{}]", stringify!($x),$a.$x, $b.$x, dx);
            return false;
        } )*
    };
}

fn veq(a: &[f32], b: &[f32], tol: f32) -> bool {
    if a.len() != b.len() {
        println!("Data Lenghts unequal: {} vs {}", a.len(), b.len());
        return false;
    }
    if a != b {
        for i in 0 .. a.len() {
            println!("{:6} {:21.15e} {:21.15e} {:21.15e}", i, a[i], b[i], (a[i]-b[i]).abs());
            if (a[i] - b[i]).abs() > tol {
                println!("{}: {} {} tol: {}", i, a[i], b[i], tol);
                return false;
            }
        }
        return true;
    }
    true
}

impl PartialEq for Sac {
    fn eq(&self, other: &Sac) -> bool {
        //println!("eq ints");
        sac_ints!(self,    other, xeq);
        //println!("eq strings");
        sac_strings!(self, other, xeq);
        //println!("eq reals");
        sac_reals!(self,   other, xeqf);
        //println!("eq npts");
        if self.npts != other.npts {
            println!("npts not equal {} {}",self.npts, other.npts);
            return false;
        }
        //println!("y len");
        if self.y.len() != other.y.len() {
            println!("npts not equal in vec, :/ {} {}", self.y.len(), other.y.len());
            return false;
        }
        //println!("y compare {}", self.y.len());
        veq(&self.y, &other.y, 1e-5) &&
            veq(&self.x, &other.x, 1e-5)
    }
}

use std::fmt;
impl fmt::Debug for Sac {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sac {{ file: {}, npts: {} }}", self.file, self.npts)
    }
}


#[derive(Fail, Debug)]
#[fail(display = "Operation attempted on a non-spectral data type.")]
pub struct NotSpectral;
#[derive(Fail, Debug)]
#[fail(display = "Operation attempted on a non-temporal data type.")]
pub struct NotTime;
#[derive(Fail, Debug)]
#[fail(display = "Encountered NaN Value.")]
pub struct NaN;

impl Sac {
    pub fn is_time(&self) -> Result<(), Error> {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::XYZ  => Ok(()),
            SacFileType::AmpPhase |
            SacFileType::RealImag => Err(NotTime.into()),
        }
    }
    pub fn is_spectral(&self) -> Result<(),Error> {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::XYZ  => Err(NotSpectral.into()),
            SacFileType::AmpPhase |
            SacFileType::RealImag => Ok(()),
        }
    }
    pub fn is_realimag(&self) -> bool {
        self.iftype == SacFileType::RealImag.into()
    }
    pub fn is_ampphase(&self) -> bool {
        self.iftype == SacFileType::AmpPhase.into()
    }

    /// Determine the number of data components
    pub fn ncomps(&self) -> usize {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY => {
                if self.is_even() { 1 } else { 2 }
            },
            SacFileType::XYZ => 1,
            SacFileType::RealImag |
            SacFileType::AmpPhase => 2,
        }
    }

    /// Set Refernce Time
    pub fn set_time(&mut self,
                    year: i32, oday: i32, hour: i32,
                    min: i32, sec: i32, msec: i32) {
        self.nzyear = year;
        self.nzjday = oday;
        self.nzhour = hour;
        self.nzmin  = min;
        self.nzsec  = sec;
        self.nzmsec = msec;
    }
    /// Read a SAC file
    pub fn read<T: AsRef<Path>>(path: T) -> Result<Sac,Error> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);
        let mut s = Sac::new();
        sac_header_read(&mut file, &mut s)?;
        sac_u8_to_strings(&mut s);
        sac_data_read(&mut file, &mut s)?;
        Ok(s)
    }
    /// Write a SAC file
    pub fn write<T: AsRef<Path>>(&mut self, path: T) -> Result<(),Error>{
        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        let npts = self.npts as usize;
        sac_strings_to_u8(self);
        sac_header_write(&mut file, self)?;
        sac_data_write(&mut file, self, npts)?;
        Ok(())
    }
    /// Create an empty SAC file
    pub fn new() -> Sac {
        let mut s0 : Sac = Sac { .. Default::default() };
        sac_reals!(s0, f32_undef);
        sac_ints!(s0,  i32_undef);
        sac_strings!(s0, str_undef);
        sac_u8_strings!(s0, u8s_undef);
        s0.iztype   = SacZeroTime::None.into();
        s0.iftype   = SacFileType::Time.into();
        s0.ievtyp   = SacEventType::None.into();
        s0.idep     = SacDataType::None.into();
        s0.nvhdr    = 6;
        s0.lpspol   = 0;
        s0.lovrok   = 1;
        s0.lcalda   = 1;
        s0.unused27 = 0;
        s0.y = vec![];
        s0.x = vec![];
        s0
    }
    /// Compute the maximum amplitude
    pub fn max_amp(&self) -> f32 {
        let mut vmax = self.y[0];
        for v in self.y.iter() { if *v > vmax { vmax = *v; } }
        vmax
    }
    /// Compute the minimum amplitude
    pub fn min_amp(&self) -> f32 {
        let mut vmin = self.y[0];
        for v in self.y.iter() { if *v < vmin { vmin = *v; } }
        vmin
    }
    /// Compute the mean amplitude
    pub fn mean_amp(&self) -> f32 {
        let vmean : f64 = self.y.iter().map(|x| *x as f64).sum();
        (vmean / self.npts as f64) as f32
    }
    /// Compute and set min, man and mean amplitudes
    pub fn extrema_amp(&mut self) {
        self.depmax = self.max_amp();
        self.depmin = self.min_amp();
        self.depmen = self.mean_amp();
    }
    /// Compute and set extremas in time and amplitude
    pub fn extrema(&mut self) {
        self.extrema_amp();
        self.e = self.b + self.delta * ((self.npts-1) as f32);
    }
    pub fn with_new_data(&self, y: Vec<f32>) -> Self {
        let mut s = self.clone();
        s.y = y;
        s.npts = s.y.len() as i32;
        s.extrema();
        s
    }
    /// Create new sac file from data and name
    pub fn from_amp_with_name(y: Vec<f32>, b: f64, dt: f64, file: &str) -> Sac {
        let mut s = Sac::from_amp(y, b, dt);
        s.file = String::from(file);
        return s;
    }
    /// Create new sac from data
    pub fn from_amp(y: Vec<f32>, b: f64, dt: f64) -> Sac {
        let mut s = Sac::new();
        s.npts   = y.len() as i32;
        s.delta  = dt as f32;
        s.b      = b as f32;
        s.y      = y;
        s.iftype = SacFileType::Time.into();
        s.leven  = true as i32;
        s.extrema();
        return s;
    }
    pub fn copy_header(&mut self, from: &Sac) {
        sac_reals!(self, from, copy_values );
        sac_ints!(self, from, copy_values );
        sac_strings!(self, from, clone_values );
    }

    /// Determine if all data is finite, not NaN, inf
    pub fn is_finite(&self) -> bool {
        self.y.iter().all(|x| x.is_finite() == true)
    }

    pub fn zero_time(&self) -> SacZeroTime {
        self.iztype.into()
    }
    pub fn is_even(&self) -> bool {
        if self.leven == 0 { false } else { true }
    }
    pub fn is_dist_az(&self) -> bool {
        if self.lcalda == 0 { false } else { true }
    }
    pub fn calc_dist_az(&mut self, value: bool) {
        self.lcalda = value as i32;
    }
    pub fn is_mutable(&self) -> bool {
        if self.lovrok == 0 { false } else { true }
    }
    pub fn mutable(&mut self) {
        self.lovrok = true as i32;
    }
    pub fn immutable(&mut self) {
        self.lovrok = false as i32;
    }
    pub fn event_type(&self) -> SacEventType {
        self.ievtyp.into()
    }
    pub fn amp_type(&self) -> SacDataType {
        self.idep.into()
    }
    pub fn file_type(&self) -> SacFileType {
        self.iftype.into()
    }
    pub fn fval(&self, key: &str) -> Option<f32> {
        match key {
            "a" | "A" => Some(self.a),
            "f" | "F" => Some(self.f),
            _ => None
        }
    }
}
use std::ops;

/* Missing Neg */

/* A op B */
fn f32add (a:f32,b:f32)->f32 {a+b}
fn f32sub (a:f32,b:f32)->f32 {a-b}
fn f32mul (a:f32,b:f32)->f32 {a*b}
fn f32div (a:f32,b:f32)->f32 {a/b}
/* B op A */
fn f32addi(a:f32,b:f32)->f32 {b+a}
fn f32subi(a:f32,b:f32)->f32 {b-a}
fn f32muli(a:f32,b:f32)->f32 {b*a}
fn f32divi(a:f32,b:f32)->f32 {b/a}

fn sac_op_sac<F>(s1: Sac, s2: Sac, f: F) -> Sac where F: Fn(f32,f32) -> f32 {
    let mut s = s1.clone();
    let npts = s.npts as usize;
    for i in 0 .. npts {
        s.y[i] = f(s.y[i], s2.y[i]);
    }
    s.extrema_amp();
    return s;
}
fn sac_op_f32<F>(x: Sac, fval: f32, f: F) -> Sac where F: Fn(f32,f32) -> f32 {
    let mut s = x.clone();
    for v in s.y.iter_mut() {
        *v = f(*v, fval);
    }
    s.extrema_amp();
    return s;
}

impl ops::Add<Sac> for f32 {    type Output = Sac;
    fn add(self, x: Sac) -> Sac { sac_op_f32(x, self, f32addi) }
}
impl ops::Sub<Sac> for f32 {    type Output = Sac;
    fn sub(self, x: Sac) -> Sac { sac_op_f32(x, self, f32subi) }
}
impl ops::Mul<Sac> for f32 {    type Output = Sac;
    fn mul(self, x: Sac) -> Sac { sac_op_f32(x, self, f32muli) }
}
impl ops::Div<Sac> for f32 {    type Output = Sac;
    fn div(self, x: Sac) -> Sac { sac_op_f32(x, self, f32divi) }
}

/* OP ASSIGN */
impl ops::AddAssign<f32> for Sac {
    fn add_assign(&mut self, x: f32) {
        self.y.iter_mut().for_each(|y| *y += x);
    }
}
impl ops::SubAssign<f32> for Sac {
    fn sub_assign(&mut self, x: f32) {
        self.y.iter_mut().for_each(|y| *y -= x);
    }
}
impl ops::MulAssign<f32> for Sac {
    fn mul_assign(&mut self, x: f32) {
        self.y.iter_mut().for_each(|y| *y *= x);
    }
}
impl ops::DivAssign<f32> for Sac {
    fn div_assign(&mut self, x: f32) {
        match self.iftype.into() {
            SacFileType::RealImag => {
                self.y.iter_mut().for_each(|v| *v /= x);
                self.x.iter_mut().for_each(|v| *v /= x);
            },
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::AmpPhase |
            SacFileType::XYZ => self.y.iter_mut().for_each(|v| *v /= x),
        }
    }
}

/* SAC op SAC */
impl ops::Add<Sac> for Sac {    type Output = Sac;
    fn add(self, x: Sac) -> Sac { sac_op_sac(self, x, f32add) }
}
impl ops::Sub<Sac> for Sac {    type Output = Sac;
    fn sub(self, x: Sac) -> Sac { sac_op_sac(self, x, f32sub) }
}
impl ops::Mul<Sac> for Sac {    type Output = Sac;
    fn mul(self, x: Sac) -> Sac { sac_op_sac(self, x, f32mul) }
}
impl ops::Div<Sac> for Sac {    type Output = Sac;
    fn div(self, x: Sac) -> Sac { sac_op_sac(self, x, f32div) }
}

/* SAC op F32 */
impl ops::Add<f32> for Sac {    type Output = Sac;
    fn add(self, x: f32) -> Sac { sac_op_f32(self, x, f32add) }
}
impl ops::Sub<f32> for Sac {    type Output = Sac;
    fn sub(self, x: f32) -> Sac { sac_op_f32(self, x, f32sub) }
}
impl ops::Mul<f32> for Sac {    type Output = Sac;
    fn mul(self, x: f32) -> Sac { sac_op_f32(self, x, f32mul) }
}
impl ops::Div<f32> for Sac {    type Output = Sac;
    fn div(self, x: f32) -> Sac { sac_op_f32(self, x, f32div) }
}
/* Neg SAC */
impl ops::Neg for Sac { type Output = Sac;
    fn neg(self) -> Sac { sac_op_f32(self, -1.0, f32mul) }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_time() {
        let mut s = Sac::from_amp(vec![0.,-1.,2.], 0.0, 1.0);
        s.file = format!("{}","create_time");
        assert_eq!(s.depmin, -1.0);
        assert_eq!(s.depmax,  2.0);
        assert_eq!(s.b,       0.0);
        assert_eq!(s.delta,   1.0);
        assert_eq!(s.depmen,  1.0/3.0);
        assert_eq!(s.e,       2.0);
        //assert_eq!(s.iztype,  SacZeroTime::B as i32);
        assert_eq!(s.iftype,  SacFileType::Time.into());
        assert_eq!(s.leven,   true as i32);
        assert_eq!(s.nvhdr,   6);
        assert_eq!(s.y,     vec![0.,-1.,2.0]);
    }

    #[test]
    fn test_finite_ok() {
        let s = Sac::from_amp_with_name(vec![1.,2.,3.], 0., 1., "finite_ok");
        assert!(s.is_finite())
    }
    #[test]
    fn test_finite_pos_inf() {
        let s = Sac::from_amp_with_name(vec![1.,2.,1./0.], 0., 1., "finite_nan");
        assert!(!s.is_finite())
    }
    #[test]
    fn test_finite_neg_inf() {
        let s = Sac::from_amp_with_name(vec![1.,2.,-1./0.], 0., 1., "finite_neg_nan");
        assert!(!s.is_finite())
    }
    #[test]
    fn test_neg() {
        let mut s = Sac::from_amp_with_name(vec![1.,2.,3.], 0., 1., "neg");
        s = -s;
        assert_eq!(s.y, [-1.,-2.,-3.]);
    }
    #[test]
    fn test_add() {
        let s = Sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "add_s #1");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s + 1.0;
        s1.file = String::from("test_add_s+1");
        assert_eq!(s1.y, [2.,3.,4.]);

        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_add_s #2");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = 1.0 + s;
        s1.file = String::from("test_add_1+s");
        assert_eq!(s1.y, [2.,3.,4.]);
    }

    #[test]
    fn test_add_vec() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        let mut t = Sac::from_amp(vec![2.,3.,4.], 0.0, 1.0);
        s.file = String::from("test_add_vec s");
        t.file = String::from("test_add_vec t");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s + t;
        s1.file = String::from("test_add_vec s+t");
        assert_eq!(s1.y, [3.,5.,7.]);

        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_add_vec s clone?");
        let mut s1 = s.clone() + s;
        s1.file = String::from("test_add_vec s1");
        assert_eq!(s1.y, [2.,4.,6.]);
    }
    #[test]
    fn test_sub() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_sub s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s - 1.0;
        s1.file = String::from("test_sub s1");
        assert_eq!(s1.y, [0.,1.,2.]);
    }
    #[test]
    fn test_mul() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_mul s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s * 2.0;
        s1.file = String::from("test_mul s1");
        assert_eq!(s1.y, [2.,4.,6.]);
    }
    #[test]
    fn test_div() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_div s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s / 2.0;
        s1.file = String::from("test_div s1");
        assert_eq!(s1.y, [0.5,1.,1.5]);
    }
    #[test]
    fn test_clone() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_clone s");
        let mut cc = s.clone();
        cc.file = String::from("test_clone cc");
        let mut s1 = s / 2.0;
        s1.file = String::from("test_clone s1");
        assert_eq!(s1.y, [0.5,1.,1.5]);
        assert_eq!(cc.y, [1.,2.,3.]);
    }
    #[test]
    fn test_assign_op() {
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_clone s");
        s += 1.0;
        assert_eq!(s.y, [2.,3.,4.]);
        s *= 2.0;
        assert_eq!(s.y, [4.,6.,8.]);
        s -= 3.0;
        assert_eq!(s.y, [1.,3.,5.]);
        s /= 2.0;
        assert_eq!(s.y, [0.5,1.5,2.5]);
    }

    #[test]
    fn read_file() {
        use std::path::Path;
        let mut s = Sac::read("tests/file.sac.swap").unwrap();
        s.file = String::from("tests/file.sac.swap");

        let mut s0 = Sac::new();
        s0.file = String::from("tests/file.sac.swap true");
        s0.set_time(1981, 88, 10, 38, 14, 0);
        s0.norid  = 0;
        s0.nevid  = 0;

        // Wow, this is kinda cheating
        s0.y = s.y.clone();

        s0.lovrok = true as i32;
        s0.lpspol = true as i32;
        s0.lcalda = true as i32;
        s0.leven  = true as i32;
        s0.unused27 = false as i32;

        s0.npts   = 1000;
        s0.iftype = SacFileType::Time.into();
        s0.idep   = SacDataType::Volts as i32;
        s0.iztype = SacZeroTime::B as i32;
        s0.ievtyp = SacEventType::Aftershock as i32;

        s0.kstnm = format!("{:-8}", "CDV");
        s0.kevnm = format!("{:-16}", "K8108838");

        s0.delta  = 0.01;
        s0.depmin = -1.56928;
        s0.depmax = 1.52064;
        s0.b      = 9.459999;
        s0.e      = 19.449999;
        s0.o      = -41.43;
        s0.a      = 10.464;
        s0.stla   = 48.0;
        s0.stlo   = -120.0;
        s0.evla   = 48.0;
        s0.evlo   = -125.0;
        s0.evdp   = 15.0;
        s0.dist   = 3.7306274e+02;
        s0.az     = 8.814721e+01;
        s0.baz    = 2.7185278e+02;
        s0.gcarc  = 3.3574646e+00;
        s0.depmen = -9.8547176e-02;
        s0.cmpaz  = 0.0;
        s0.cmpinc = 0.0;

        println!("compare s and s0");
        assert_eq!(s,s0);
        println!("write file");
        let path = Path::new("tmp.sac");
        s.write(path.to_path_buf()).unwrap();

        println!("write file with long kevnm");
        s.kevnm = format!("{}", "123456789012345678901234567890");
        let path = Path::new("tmp2.sac");
        s.write(path.to_path_buf()).unwrap();

        println!("write file with short kevnm");
        s.kevnm = format!("{}", "12");
        let path = Path::new("tmp3.sac");
        s.write(path.to_path_buf()).unwrap();
        {
            for elem in s.y.iter_mut() {
                *elem += 1.0;
            }
        }
    }

}

/// SAC file data and metadata
///
///
#[derive(Clone, Default)]
#[repr(C)]
pub struct Sac {
    /// Time sampling
    pub delta: f32,               /* RF time increment, sec    */
    /// Miniumum Value of y
    pub depmin: f32,               /*    minimum amplitude      */
    /// Maximum value of y
    pub depmax: f32,               /*    maximum amplitude      */
    pub scale: f32,                /*    amplitude scale factor */
    pub odelta: f32,               /*    observed time inc      */
    /// Begin time value of the data
    pub b: f32,                    /* RD initial value, time    */
    /// End time value of the data
    pub e: f32,                    /* RD final value, time      */
    pub o: f32,                    /*    event start, sec < nz. */
    pub a: f32,                    /*    1st arrival time       */
    pub fmt: f32,                  /*    internal use           */
    pub t0: f32,                   /*    user-defined time pick */
    pub t1: f32,                   /*    user-defined time pick */
    pub t2: f32,                   /*    user-defined time pick */
    pub t3: f32,                   /*    user-defined time pick */
    pub t4: f32,                   /*    user-defined time pick */
    pub t5: f32,                   /*    user-defined time pick */
    pub t6: f32,                   /*    user-defined time pick */
    pub t7: f32,                   /*    user-defined time pick */
    pub t8: f32,                   /*    user-defined time pick */
    pub t9: f32,                   /*    user-defined time pick */
    pub f: f32,                    /*    event end, sec > nz    */
    pub resp0: f32,                /*    instrument respnse parm */
    pub resp1: f32,                /*    instrument respnse parm */
    pub resp2: f32,                /*    instrument respnse parm */
    pub resp3: f32,                /*    instrument respnse parm */
    pub resp4: f32,                /*    instrument respnse parm */
    pub resp5: f32,                /*    instrument respnse parm */
    pub resp6: f32,                /*    instrument respnse parm */
    pub resp7: f32,                /*    instrument respnse parm */
    pub resp8: f32,                /*    instrument respnse parm */
    pub resp9: f32,                /*    instrument respnse parm */
    pub stla: f32,                 /*  T station latititude     */
    pub stlo: f32,                 /*  T station longitude      */
    pub stel: f32,                 /*  T station elevation, m   */
    pub stdp: f32,                 /*  T station depth, m      */
    pub evla: f32,                 /*    event latitude         */
    pub evlo: f32,                 /*    event longitude        */
    pub evel: f32,                 /*    event elevation        */
    pub evdp: f32,                 /*    event depth            */
    pub mag: f32,                  /*    reserved for future use */
    pub user0: f32,                /*    available to user      */
    pub user1: f32,                /*    available to user      */
    pub user2: f32,                /*    available to user      */
    pub user3: f32,                /*    available to user      */
    pub user4: f32,                /*    available to user      */
    pub user5: f32,                /*    available to user      */
    pub user6: f32,                /*    available to user      */
    pub user7: f32,                /*    available to user      */
    pub user8: f32,                /*    available to user      */
    pub user9: f32,                /*    available to user      */
    pub dist: f32,                 /*    stn-event distance, km */
    pub az: f32,                   /*    event-stn azimuth      */
    pub baz: f32,                  /*    stn-event azimuth      */
    pub gcarc: f32,                /*    stn-event dist, degrees */
    pub sb: f32,                   /*    internal use           */
    pub sdelta: f32,               /*    internal use           */
    pub depmen: f32,               /*    mean value, amplitude  */
    pub cmpaz: f32,                /*  T component azimuth     */
    pub cmpinc: f32,               /*  T component inclination */
    pub xminimum: f32,             /*    reserved for future use */
    pub xmaximum: f32,             /*    reserved for future use */
    pub yminimum: f32,             /*    reserved for future use */
    pub ymaximum: f32,             /*    reserved for future use */
    unused6: f32,              /*    reserved for future use */
    unused7: f32,              /*    reserved for future use */
    unused8: f32,              /*    reserved for future use */
    unused9: f32,              /*    reserved for future use */
    unused10: f32,             /*    reserved for future use */
    unused11: f32,             /*    reserved for future use */
    unused12: f32,             /*    reserved for future use */
    pub nzyear: i32,                 /*  F zero time of file, yr  */
    pub nzjday: i32,                 /*  F zero time of file, day */
    pub nzhour: i32,                 /*  F zero time of file, hr  */
    pub nzmin: i32,                  /*  F zero time of file, min */
    pub nzsec: i32,                  /*  F zero time of file, sec */
    pub nzmsec: i32,                 /*  F zero time of file, msec */
    /// Header Version, Should be 6
    pub nvhdr: i32,                  /*    internal use           */
    pub norid: i32,                  /*    origin ID              */
    pub nevid: i32,                  /*    event ID               */
    /// Number of data points
    pub npts: i32,                   /* RF number of samples      */
    pub nsnpts: i32,                 /*    internal use           */
    pub nwfid: i32,                  /*    waveform ID            */
    pub nxsize: i32,                 /*    reserved for future use */
    pub nysize: i32,                 /*    reserved for future use */
    unused15: i32,               /*    reserved for future use */
    pub iftype: i32,                 /* RA type of file          */
    pub idep: i32,                   /*    type of amplitude      */
    pub iztype: i32,                 /*    zero time equivalence  */
    unused16: i32,               /*    reserved for future use */
    pub iinst: i32,                  /*    recording instrument   */
    pub istreg: i32,                 /*    stn geographic region  */
    pub ievreg: i32,                 /*    event geographic region */
    pub ievtyp: i32,                 /*    event type             */
    pub iqual: i32,                  /*    quality of data        */
    pub isynth: i32,                 /*    synthetic data flag    */
    pub imagtyp: i32,                /*    reserved for future use */
    pub imagsrc: i32,                /*    reserved for future use */
    unused19: i32,               /*    reserved for future use */
    unused20: i32,               /*    reserved for future use */
    unused21: i32,               /*    reserved for future use */
    unused22: i32,               /*    reserved for future use */
    unused23: i32,               /*    reserved for future use */
    unused24: i32,               /*    reserved for future use */
    unused25: i32,               /*    reserved for future use */
    unused26: i32,               /*    reserved for future use */
    pub leven: i32,                  /* RA data-evenly-spaced flag */
    pub lpspol: i32,                 /*    station polarity flag  */
    pub lovrok: i32,                 /*    overwrite permission   */
    pub lcalda: i32,                 /*    calc distance, azimuth */
    unused27: i32,               /*    reserved for future use */

    u8_kstnm: [u8; 8],              /*  F station name           */
    u8_kevnm: [u8; 16],             /*    event name             */
    u8_khole: [u8; 8],              /*    man-made event name    */
    u8_ko: [u8; 8],                 /*    event origin time id   */
    u8_ka: [u8; 8],                 /*    1st arrival time ident */
    u8_kt0: [u8; 8],                /*    time pick 0 ident      */
    u8_kt1: [u8; 8],                /*    time pick 1 ident      */
    u8_kt2: [u8; 8],                /*    time pick 2 ident      */
    u8_kt3: [u8; 8],                /*    time pick 3 ident      */
    u8_kt4: [u8; 8],                /*    time pick 4 ident      */
    u8_kt5: [u8; 8],                /*    time pick 5 ident      */
    u8_kt6: [u8; 8],                /*    time pick 6 ident      */
    u8_kt7: [u8; 8],                /*    time pick 7 ident      */
    u8_kt8: [u8; 8],                /*    time pick 8 ident      */
    u8_kt9: [u8; 8],                /*    time pick 9 ident      */
    u8_kf: [u8; 8],                 /*    end of event ident     */
    u8_kuser0: [u8; 8],             /*    available to user      */
    u8_kuser1: [u8; 8],             /*    available to user      */
    u8_kuser2: [u8; 8],             /*    available to user      */
    u8_kcmpnm: [u8; 8],             /*  F component name         */
    u8_knetwk: [u8; 8],             /*    network name           */
    u8_kdatrd: [u8; 8],             /*    date data read         */
    u8_kinst: [u8; 8],              /*    instrument name        */

    pub kstnm: String,              /*  F station name           */
    pub kevnm: String,             /*    event name             */
    pub khole: String,              /*    man-made event name    */
    pub ko: String,                 /*    event origin time id   */
    pub ka: String,                 /*    1st arrival time ident */
    pub kt0: String,                /*    time pick 0 ident      */
    pub kt1: String,                /*    time pick 1 ident      */
    pub kt2: String,                /*    time pick 2 ident      */
    pub kt3: String,                /*    time pick 3 ident      */
    pub kt4: String,                /*    time pick 4 ident      */
    pub kt5: String,                /*    time pick 5 ident      */
    pub kt6: String,                /*    time pick 6 ident      */
    pub kt7: String,                /*    time pick 7 ident      */
    pub kt8: String,                /*    time pick 8 ident      */
    pub kt9: String,                /*    time pick 9 ident      */
    pub kf: String,                 /*    end of event ident     */
    pub kuser0: String,             /*    available to user      */
    pub kuser1: String,             /*    available to user      */
    pub kuser2: String,             /*    available to user      */
    pub kcmpnm: String,             /*  F component name         */
    pub knetwk: String,             /*    network name           */
    pub kdatrd: String,             /*    date data read         */
    pub kinst: String,              /*    instrument name        */


    /// Dependent variable
    ///   - Amplitude for time series data
    ///   - Amplitude for Amplitude Phase data
    ///   - Real for Real Imaginary data
    ///   - Z for XYZ data
    ///   - Y for XY data
    pub y: Vec<f32>,
    /// Indepenent variable, Time for time series data
    ///   - Timing for time series data, only for uneven data
    ///   - Phase for Amplitude Phase data
    ///   - Imaginary for Real Imaginary data
    ///   - X for XY data
    pub x: Vec<f32>,
    /// Filename of the Sac Data File
    pub file: String,
    /// If data is swapped from native byte order
    pub swap: bool,
}


