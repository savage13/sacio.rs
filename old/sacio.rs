#![allow(dead_code)]

use std::mem;
use std::slice;
use std::fs::File;
use std::error::Error;
use std::path::PathBuf;
use std::io::prelude::*;

#[macro_export]
macro_rules! sac_reals {
    ($s:ident, $function:ident) => { sac_reals!($s, ignore_idnet, $function); };
    ($s:ident, $z:ident, $function:ident) => {
        $function!($s,$z,
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
    ($s:ident, $function:ident) => { sac_ints!($s, ignore_idnet, $function); };
    ($s:ident, $z:ident, $function:ident) => {
        $function!($s,$z,
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
                   kuser0, kuser1, kuser2, kcmpnm, knetwk, kdatrd, kinst);
    }
}

macro_rules! sac_u8_strings {
    ($s:ident, $function:ident) => {
        $function!($s,
                   u8_kstnm, u8_khole, u8_ko, u8_ka,
                   u8_kt0, u8_kt1, u8_kt2, u8_kt3, u8_kt4,
                   u8_kt5, u8_kt6, u8_kt7, u8_kt8, u8_kt9,
                   u8_kuser0, u8_kuser1, u8_kuser2, u8_kcmpnm,
                   u8_knetwk, u8_kdatrd, u8_kinst);
    }
}


macro_rules! string_to_u8 {
    ($s:ident, $($a:ident, $b:ident),*) => {
        $(
            let mut tmp : String;
            if $s.$b.len() == 8 {
                tmp = format!("{:8}", $s.$a);
            } else {
                tmp = format!("{:16}", $s.$a);
            }
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
                   kuser0, u8_kuser0, kuser1, u8_kuser1, kuser2, u8_kuser2,
                   kcmpnm, u8_kcmpnm, knetwk,  u8_knetwk, kdatrd, u8_kdatrd,
                   kinst, u8_kinst
        );
    }
}

macro_rules! swap_reals {
    ($s:ident, $empty:ident, $($x:ident),*) => ( $( $s.$x = swap_f32($s.$x); )* );
}
macro_rules! swap_ints {
    ($s:ident, $empty:ident, $($x:ident),*) => ( $( $s.$x = $s.$x.swap_bytes(); )* );
}

//use std::str::from_utf8;
macro_rules! u8_to_string {
    ($s:ident, $($x:ident, $u8x:ident),*) => (
        $( $s.$x = String::from_utf8($s.$u8x.to_vec()).unwrap(); )*
    );
}

macro_rules! f32_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $( $s.$x = -12345.0; )* );
}
macro_rules! i32_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $( $s.$x = -12345; )* );
}
macro_rules! str_undef {
    ($s:ident, $q:ident, $($x:ident),*) => ( $( $s.$x = String::from("-12345  "); )* );
}
macro_rules! u8s_undef {
    ($s:ident, $($x:ident),*) => ( $( $s.$x = [45,49,50,51,52,53,32,32]; )* );
}
macro_rules! u8_to_str {
    ($s:ident, $u8x:ident) => ( from_utf8(&$s.$u8x).unwrap(); );
}

fn sac_u8_to_strings(s: &mut sac) {
    sac_strings_pair!(s, u8_to_string);
}

fn sac_strings_to_u8(s: &mut sac) {
    sac_strings_pair!(s, string_to_u8);
}

fn swap_f32(x: f32) -> f32 {
    use std::mem::transmute;
    let y : f32;
    unsafe {
        let mut b : u32 = transmute(x);
        b = b.swap_bytes();
        y = transmute(b);
    }
    return y;
}

fn sac_data_swap(h: &mut sac) {
    //let mut y = h.y;
    for v in h.y.iter_mut() {
        *v = swap_f32(*v);
    }
}

fn sac_header_swap(h: &mut sac)  {
    sac_reals!( h, swap_reals );
    sac_ints!(  h, swap_ints  );
}

fn struct_to_u8(s: &mut sac, hsize: usize) -> &mut [u8] {
    return unsafe { slice::from_raw_parts_mut(s as *mut _ as *mut u8, hsize) };
}
fn vec_to_u8(y: &mut Vec<f32>, hsize: usize) -> &mut [u8] {
    return unsafe { slice::from_raw_parts_mut(y.as_mut_ptr() as *mut u8, hsize) };
}

fn sac_data_read(file: &mut File, h: &mut sac) {
        /* Create a vector with capacity */
    let npts = h.npts as usize;
    let hsize : usize = npts * mem::size_of::<f32>();
    h.y = vec![0.0; npts];
    let buffer = vec_to_u8(&mut h.y, hsize);
    file.read_exact(buffer).unwrap();
}

fn sac_data_write(file: &mut File, s: &mut sac, npts: usize) {
    let hsize : usize = npts * mem::size_of::<f32>();
    let hslice = vec_to_u8(&mut s.y, hsize);
    file.write_all(hslice).unwrap();
}

const HEADER_SIZE : usize = 632;
fn sac_header_read(file: &mut File, h: &mut sac) {
    let hslice = struct_to_u8(h, HEADER_SIZE);
    file.read_exact(hslice).unwrap();
}

fn sac_header_write(file: &mut File, s: &mut sac) {
    let hslice = struct_to_u8(s, HEADER_SIZE);
    file.write_all(hslice).unwrap();
}

fn sac_new() -> sac {
    let mut s0 : sac = unsafe {mem::zeroed()};
    sac_reals!(s0, f32_undef);
    sac_ints!(s0,  i32_undef);
    sac_strings!(s0, str_undef);
    sac_u8_strings!(s0, u8s_undef);
    s0.u8_kevnm = [45,49,50,51,52,53,32,32,
                   32,32,32,32,32,32,32,32];
    s0.nvhdr  = 6;
    return s0;
}

fn sac_create_file(path: PathBuf) -> File {
    let display = path.as_path().display();
    let file : File = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };
    return file;
}

fn sac_open_file(path: PathBuf) -> File {
    let display = path.as_path().display();
    let file : File = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
    return file;
}

fn sac_read(path: PathBuf) -> sac {
    let mut h = sac_new();
    let mut file = sac_open_file(path);
    sac_header_read(&mut file, &mut h);

    h.swap = false;
    if h.nvhdr != 6  {
        if h.nvhdr.swap_bytes() != 6 {
            panic!("Unknown file type");
        }
        h.swap = true;
        sac_header_swap(&mut h);
    }

    sac_u8_to_strings(&mut h);

    sac_data_read(&mut file, &mut h);
    if h.swap {
        sac_data_swap(&mut h);
    }
    return h;
}


fn sac_write(s: &mut sac, path: PathBuf) {
    let mut file = sac_create_file(path);
    let npts = s.npts as usize;
    sac_strings_to_u8(s);
    if s.swap {
        let mut s0 = s.clone();
        sac_data_swap(&mut s0);
        sac_header_swap(&mut s0);
        sac_header_write(&mut file, &mut s0);
        sac_data_write(&mut file, &mut s0, npts);
    } else {
        sac_header_write(&mut file, s);
        sac_data_write(&mut file, s, npts);
    }
}

//h.kstnm = String::from_utf8(h._kstnm.to_vec()).unwrap();
//println!("kstnm >{0}<", from_utf8(&h.u8_kstnm).unwrap());
//println!("kstnm >{0}<", u8_to_str!(h, u8_kstnm));
//println!("kstnm >{0}<", String::from_utf8(h.u8_kstnm.to_vec()).unwrap());
//println!("kstnm >{0}<", h.kstnm);


macro_rules! xeq {
    ($a:ident,$b:ident,$($x:ident),*) => {
        $( if $a.$x != $b.$x {
            println!("a != b :: {} a: {} b: {}", stringify!($x),$a.$x, $b.$x);
            return false;
        } )*
    };
}

impl PartialEq for sac {
    fn eq(&self, other: &sac) -> bool {
        sac_ints!(self, other, xeq);
        sac_strings!(self, other, xeq);
        sac_reals!(self, other, xeq);
        return true;
    }
}

use std::fmt;
impl fmt::Debug for sac {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sac {{ file: {}, npts: {} }}", self.file, self.npts)
    }
}

pub enum SacFileType {
    Real          = 0,
    Time          = 1,
    RealImaginary = 2,
    AmpPhase      = 3,
    XY            = 4,
    IXYZ          = 51,
}

pub enum SacZeroTime {
    B   = 9,
    Day = 10,
    O   = 11,
    A   = 12,
    T0  = 13,
    T1  = 14,
    T2  = 15,
    T3  = 16,
    T4  = 17,
    T5  = 18,
    T6  = 19,
    T7  = 20,
    T8  = 21,
    T9  = 22,
}

pub enum SacEventType {
    NuclearShot       = 37,
    NuclearPreShot    = 38,
    NuclearPostShot   = 39,
    Earthquake        = 40,
    Foreshock         = 41,
    Aftershock        = 42,
    ChemicalExplosion = 43,
    Other             = 44,
    QuarryBlast       = 72,
    QuarryBlast1      = 73,
    QuarryBlast2      = 74,
}

pub enum SacDataType {
    Displacement = 6,
    Velocity     = 7,
    Acceleration = 8,
    Volts        = 50,
}

impl sac {
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
    pub fn read(path: PathBuf) -> sac {
        return sac_read(path);
    }
    pub fn write(&mut self, path: PathBuf) {
        return sac_write(self, path);
    }
    pub fn new() -> sac {
        sac_new()
    }
    pub fn max_amp(&self) -> f32 {
        let mut vmax = self.y[0];
        for v in self.y.iter() { if *v > vmax { vmax = *v; } }
        vmax
    }
    pub fn min_amp(&self) -> f32 {
        let mut vmin = self.y[0];
        for v in self.y.iter() { if *v < vmin { vmin = *v; } }
        vmin
    }
    pub fn mean_amp(&self) -> f32 {
        let mut vmean : f64 = 0.0;
        for v in self.y.iter() { vmean = vmean + (*v as f64); }
        (vmean / (self.npts as f64)) as f32
    }
    pub fn extrema_amp(&mut self) {
        self.depmax = self.max_amp();
        self.depmin = self.min_amp();
        self.depmen = self.mean_amp();
    }
    pub fn extrema(&mut self) {
        self.extrema_amp();
        self.e = self.b + self.delta * ((self.npts-1) as f32);
    }
    pub fn from_amp_with_name(y: Vec<f32>, b: f32, dt: f32, file: &str) -> sac {
        let mut s = sac::from_amp(y, b, dt);
        s.file = String::from(file);
        return s;
    }
    pub fn from_amp(y: Vec<f32>, b: f32, dt: f32) -> sac {
        let mut s = sac_new();
        s.npts   = y.len() as i32;
        s.delta  = dt;
        s.b      = b;
        s.y      = y;
        s.iztype = SacZeroTime::B as i32;
        s.iftype = SacFileType::Time as i32;
        s.leven  = true as i32;
        s.extrema();
        return s;
    }
    pub fn exp(&mut self) {
        for v in self.y.iter_mut() {
            *v = v.exp();
        }
        self.extrema_amp();
    }
    pub fn is_finite(&self) -> bool {
        self.y.iter().all(|x| x.is_finite() == true)
    }
    pub fn integrate(&mut self) {
        let hstep = 0.5 * self.delta;
        let mut yt = 0.0;
        let npts = (self.npts - 1) as usize;
        // Trapezodial Integration
        {
            for i in 0 .. npts {
                let pt = hstep * (self.y[i] + self.y[i+1]);
                yt = yt + pt;
                self.y[i] = yt;
            }
            self.y.pop(); // Remove Last Point 
        }
        self.npts = self.npts - 1;
        self.extrema();
    }
    pub fn differentiate(&mut self) {
        // 2-pt Differentiation
        let factor = 1.0 / self.delta;
        let npts = (self.npts - 1) as usize;
        {
            for i in 0 .. npts {
                self.y[i] = factor * (self.y[i+1] - self.y[i]);
            }
            self.y.pop();
        }
        self.npts = self.npts - 1;
        self.extrema();
    }
}



/*
impl Drop for sac {
    fn drop(&mut self) {
        println!("> Dropping {:?}", self);
    }
}
*/
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

/* F32 op SAC */
fn sac_op_sac<F>(s1: sac, s2: sac, f: F) -> sac where F: Fn(f32,f32) -> f32 {
    let mut s = s1.clone();
    let npts = s.npts as usize;
    for i in 0 .. npts {
        s.y[i] = f(s.y[i], s2.y[i]);
    }
    s.extrema_amp();
    return s;
}
fn sac_op_f32<F>(x: sac, fval: f32, f: F) -> sac where F: Fn(f32,f32) -> f32 {
    let mut s = x.clone();
    for v in s.y.iter_mut() {
        *v = f(*v, fval);
    }
    s.extrema_amp();
    return s;
}

impl ops::Add<sac> for f32 {    type Output = sac;
    fn add(self, x: sac) -> sac { sac_op_f32(x, self, f32addi) }
}
impl ops::Sub<sac> for f32 {    type Output = sac;
    fn sub(self, x: sac) -> sac { sac_op_f32(x, self, f32subi) }
}
impl ops::Mul<sac> for f32 {    type Output = sac;
    fn mul(self, x: sac) -> sac { sac_op_f32(x, self, f32muli) }
}
impl ops::Div<sac> for f32 {    type Output = sac;
    fn div(self, x: sac) -> sac { sac_op_f32(x, self, f32divi) }
}

/* OP ASSIGN */
impl ops::AddAssign<f32> for sac {
    fn add_assign(&mut self, x: f32) { *self = self.clone() + x; }
}
impl ops::SubAssign<f32> for sac {
    fn sub_assign(&mut self, x: f32) { *self = self.clone() - x; }
}
impl ops::MulAssign<f32> for sac {
    fn mul_assign(&mut self, x: f32) { *self = self.clone() * x; }
}
impl ops::DivAssign<f32> for sac {
    fn div_assign(&mut self, x: f32) { *self = self.clone() / x; }
}

/* SAC op SAC */
impl ops::Add<sac> for sac {    type Output = sac;
    fn add(self, x: sac) -> sac { sac_op_sac(self, x, f32add) }
}
impl ops::Sub<sac> for sac {    type Output = sac;
    fn sub(self, x: sac) -> sac { sac_op_sac(self, x, f32sub) }
}
impl ops::Mul<sac> for sac {    type Output = sac;
    fn mul(self, x: sac) -> sac { sac_op_sac(self, x, f32mul) }
}
impl ops::Div<sac> for sac {    type Output = sac;
    fn div(self, x: sac) -> sac { sac_op_sac(self, x, f32div) }
}

/* SAC op F32 */
impl ops::Add<f32> for sac {    type Output = sac;
    fn add(self, x: f32) -> sac { sac_op_f32(self, x, f32add) }
}
impl ops::Sub<f32> for sac {    type Output = sac;
    fn sub(self, x: f32) -> sac { sac_op_f32(self, x, f32sub) }
}
impl ops::Mul<f32> for sac {    type Output = sac;
    fn mul(self, x: f32) -> sac { sac_op_f32(self, x, f32mul) }
}
impl ops::Div<f32> for sac {    type Output = sac;
    fn div(self, x: f32) -> sac { sac_op_f32(self, x, f32div) }
}
/* Neg SAC */
impl ops::Neg for sac { type Output = sac;
    fn neg(self) -> sac { sac_op_f32(self, -1.0, f32mul) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
    }

    #[test]
    fn create_time() {
        let mut s = sac::from_amp(vec![0.,-1.,2.], 0.0, 1.0);
        s.file = format!("{}","create_time");
        assert_eq!(s.depmin, -1.0);
        assert_eq!(s.depmax,  2.0);
        assert_eq!(s.b,       0.0);
        assert_eq!(s.delta,   1.0);
        assert_eq!(s.depmen,  1.0/3.0);
        assert_eq!(s.e,       2.0);
        assert_eq!(s.iztype,  SacZeroTime::B as i32);
        assert_eq!(s.iftype,  SacFileType::Time as i32);
        assert_eq!(s.leven,   true as i32);
        assert_eq!(s.nvhdr,   6);
        assert_eq!(s.y,     vec![0.,-1.,2.0]);
    }
    #[test]
    fn test_exp() {
        let mut s = sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "test_exp");
        s.exp();
        let v = vec![(1f32).exp(), (2f32).exp(), (3f32).exp()];
        assert_eq!(s.y, v);
    }

    #[test]
    fn test_finite() {
        let s = sac::from_amp_with_name(vec![1.,2.,3.], 0., 1., "finite_ok");
        assert_eq!(s.is_finite(), true);
        let s = sac::from_amp_with_name(vec![1.,2.,1./0.], 0., 1., "finite_nan");
        assert_eq!(s.is_finite(), false);
        let s = sac::from_amp_with_name(vec![1.,2.,-1./0.], 0., 1., "finite_neg_nan");
        assert_eq!(s.is_finite(), false);
    }
    #[test]
    fn test_neg() {
        let mut s = sac::from_amp_with_name(vec![1.,2.,3.], 0., 1., "neg");
        s = -s;
        assert_eq!(s.y, [-1.,-2.,-3.]);
    }
    #[test]
    fn test_integrate() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_integrate");
        s.integrate();
        assert_eq!(s.y, [1.5, 4.]);
    }
    #[test]
    fn test_differentiate() {
        let mut s = sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "diff");
        s.differentiate();
        assert_eq!(s.y, [1.0, 1.0]);
    }

    #[test]
    fn test_add() {
        let s = sac::from_amp_with_name(vec![1.,2.,3.], 0.0, 1.0, "add_s #1");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s + 1.0;
        s1.file = String::from("test_add_s+1");
        assert_eq!(s1.y, [2.,3.,4.]);

        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_add_s #2");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = 1.0 + s;
        s1.file = String::from("test_add_1+s");
        assert_eq!(s1.y, [2.,3.,4.]);
    }

    #[test]
    fn test_add_vec() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        let mut t = sac::from_amp(vec![2.,3.,4.], 0.0, 1.0);
        s.file = String::from("test_add_vec s");
        t.file = String::from("test_add_vec t");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s + t;
        s1.file = String::from("test_add_vec s+t");
        assert_eq!(s1.y, [3.,5.,7.]);

        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_add_vec s clone?");
        let mut s1 = s.clone() + s;
        s1.file = String::from("test_add_vec s1");
        assert_eq!(s1.y, [2.,4.,6.]);
    }
    #[test]
    fn test_sub() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_sub s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s - 1.0;
        s1.file = String::from("test_sub s1");
        assert_eq!(s1.y, [0.,1.,2.]);
    }
    #[test]
    fn test_mul() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_mul s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s * 2.0;
        s1.file = String::from("test_mul s1");
        assert_eq!(s1.y, [2.,4.,6.]);
    }
    #[test]
    fn test_div() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        s.file = String::from("test_div s");
        assert_eq!(s.y, [1.,2.,3.]);
        let mut s1 = s / 2.0;
        s1.file = String::from("test_div s1");
        assert_eq!(s1.y, [0.5,1.,1.5]);
    }
    #[test]
    fn test_clone() {
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
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
        let mut s = sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
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
        let path = Path::new("file.sac.swap");
        let mut s = sac::read(path.to_path_buf());
        s.file = String::from("file.sac.swap");
        let mut s0 = sac::new();
        s0.file = String::from("file.sac.swap true");
        s0.set_time(1981, 88, 10, 38, 14, 0);
        s0.norid  = 0;
        s0.nevid  = 0;

        s0.lovrok = true as i32;
        s0.lpspol = true as i32;
        s0.lcalda = true as i32;
        s0.leven  = true as i32;
        s0.unused27 = false as i32;

        s0.npts   = 1000;
        s0.iftype = SacFileType::Time as i32;
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

        assert_eq!(s,s0);

        let path = Path::new("tmp.sac");
        s.write(path.to_path_buf());

        s.kevnm = format!("{}", "123456789012345678901234567890");
        let path = Path::new("tmp2.sac");
        s.write(path.to_path_buf());

        s.kevnm = format!("{}", "12");
        let path = Path::new("tmp3.sac");
        s.write(path.to_path_buf());

        {
            for elem in s.y.iter_mut() {
                *elem += 1.0;
            }
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct sac {
    pub delta: f32,               /* RF time increment, sec    */
    pub depmin: f32,               /*    minimum amplitude      */
    pub depmax: f32,               /*    maximum amplitude      */
    scale: f32,                /*    amplitude scale factor */
    odelta: f32,               /*    observed time inc      */
    pub b: f32,                    /* RD initial value, time    */
    pub e: f32,                    /* RD final value, time      */
    o: f32,                    /*    event start, sec < nz. */
    a: f32,                    /*    1st arrival time       */
    fmt: f32,                  /*    internal use           */
    t0: f32,                   /*    user-defined time pick */
    t1: f32,                   /*    user-defined time pick */
    t2: f32,                   /*    user-defined time pick */
    t3: f32,                   /*    user-defined time pick */
    t4: f32,                   /*    user-defined time pick */
    t5: f32,                   /*    user-defined time pick */
    t6: f32,                   /*    user-defined time pick */
    t7: f32,                   /*    user-defined time pick */
    t8: f32,                   /*    user-defined time pick */
    t9: f32,                   /*    user-defined time pick */
    f: f32,                    /*    event end, sec > nz    */
    resp0: f32,                /*    instrument respnse parm */
    resp1: f32,                /*    instrument respnse parm */
    resp2: f32,                /*    instrument respnse parm */
    resp3: f32,                /*    instrument respnse parm */
    resp4: f32,                /*    instrument respnse parm */
    resp5: f32,                /*    instrument respnse parm */
    resp6: f32,                /*    instrument respnse parm */
    resp7: f32,                /*    instrument respnse parm */
    resp8: f32,                /*    instrument respnse parm */
    resp9: f32,                /*    instrument respnse parm */
    stla: f32,                 /*  T station latititude     */
    stlo: f32,                 /*  T station longitude      */
    stel: f32,                 /*  T station elevation, m   */
    stdp: f32,                 /*  T station depth, m      */
    evla: f32,                 /*    event latitude         */
    evlo: f32,                 /*    event longitude        */
    evel: f32,                 /*    event elevation        */
    evdp: f32,                 /*    event depth            */
    mag: f32,                  /*    reserved for future use */
    user0: f32,                /*    available to user      */
    user1: f32,                /*    available to user      */
    user2: f32,                /*    available to user      */
    user3: f32,                /*    available to user      */
    user4: f32,                /*    available to user      */
    user5: f32,                /*    available to user      */
    user6: f32,                /*    available to user      */
    user7: f32,                /*    available to user      */
    user8: f32,                /*    available to user      */
    user9: f32,                /*    available to user      */
    dist: f32,                 /*    stn-event distance, km */
    az: f32,                   /*    event-stn azimuth      */
    baz: f32,                  /*    stn-event azimuth      */
    gcarc: f32,                /*    stn-event dist, degrees */
    sb: f32,                   /*    internal use           */
    sdelta: f32,               /*    internal use           */
    depmen: f32,               /*    mean value, amplitude  */
    cmpaz: f32,                /*  T component azimuth     */
    cmpinc: f32,               /*  T component inclination */
    xminimum: f32,             /*    reserved for future use */
    xmaximum: f32,             /*    reserved for future use */
    yminimum: f32,             /*    reserved for future use */
    ymaximum: f32,             /*    reserved for future use */
    unused6: f32,              /*    reserved for future use */
    unused7: f32,              /*    reserved for future use */
    unused8: f32,              /*    reserved for future use */
    unused9: f32,              /*    reserved for future use */
    unused10: f32,             /*    reserved for future use */
    unused11: f32,             /*    reserved for future use */
    unused12: f32,             /*    reserved for future use */
    nzyear: i32,                 /*  F zero time of file, yr  */
    nzjday: i32,                 /*  F zero time of file, day */
    nzhour: i32,                 /*  F zero time of file, hr  */
    nzmin: i32,                  /*  F zero time of file, min */
    nzsec: i32,                  /*  F zero time of file, sec */
    nzmsec: i32,                 /*  F zero time of file, msec */
    pub nvhdr: i32,                  /*    internal use           */
    norid: i32,                  /*    origin ID              */
    nevid: i32,                  /*    event ID               */
    pub npts: i32,                   /* RF number of samples      */
    nsnpts: i32,                 /*    internal use           */
    nwfid: i32,                  /*    waveform ID            */
    nxsize: i32,                 /*    reserved for future use */
    nysize: i32,                 /*    reserved for future use */
    unused15: i32,               /*    reserved for future use */
    iftype: i32,                 /* RA type of file          */
    idep: i32,                   /*    type of amplitude      */
    iztype: i32,                 /*    zero time equivalence  */
    unused16: i32,               /*    reserved for future use */
    iinst: i32,                  /*    recording instrument   */
    istreg: i32,                 /*    stn geographic region  */
    ievreg: i32,                 /*    event geographic region */
    ievtyp: i32,                 /*    event type             */
    iqual: i32,                  /*    quality of data        */
    isynth: i32,                 /*    synthetic data flag    */
    imagtyp: i32,                /*    reserved for future use */
    imagsrc: i32,                /*    reserved for future use */
    unused19: i32,               /*    reserved for future use */
    unused20: i32,               /*    reserved for future use */
    unused21: i32,               /*    reserved for future use */
    unused22: i32,               /*    reserved for future use */
    unused23: i32,               /*    reserved for future use */
    unused24: i32,               /*    reserved for future use */
    unused25: i32,               /*    reserved for future use */
    unused26: i32,               /*    reserved for future use */
    leven: i32,                  /* RA data-evenly-spaced flag */
    lpspol: i32,                 /*    station polarity flag  */
    lovrok: i32,                 /*    overwrite permission   */
    lcalda: i32,                 /*    calc distance, azimuth */
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

    kstnm: String,              /*  F station name           */
    kevnm: String,             /*    event name             */
    khole: String,              /*    man-made event name    */
    ko: String,                 /*    event origin time id   */
    ka: String,                 /*    1st arrival time ident */
    kt0: String,                /*    time pick 0 ident      */
    kt1: String,                /*    time pick 1 ident      */
    kt2: String,                /*    time pick 2 ident      */
    kt3: String,                /*    time pick 3 ident      */
    kt4: String,                /*    time pick 4 ident      */
    kt5: String,                /*    time pick 5 ident      */
    kt6: String,                /*    time pick 6 ident      */
    kt7: String,                /*    time pick 7 ident      */
    kt8: String,                /*    time pick 8 ident      */
    kt9: String,                /*    time pick 9 ident      */
    kf: String,                 /*    end of event ident     */
    kuser0: String,             /*    available to user      */
    kuser1: String,             /*    available to user      */
    kuser2: String,             /*    available to user      */
    kcmpnm: String,             /*  F component name         */
    knetwk: String,             /*    network name           */
    kdatrd: String,             /*    date data read         */
    kinst: String,              /*    instrument name        */

    pub swap: bool,

    pub y: Vec<f32>,
    file: String,
}

