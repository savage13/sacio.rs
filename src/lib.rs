
/*! Library for reading and writing Seismic Analysis Code (SAC) files

Reference: [SAC Manual](http://ds.iris.edu/files/sac-manual/)

```
use sacio::Sac;
# use sacio::SacError;
use sacio::SacString;

let mut s = Sac::from_file("tests/file.sac")?;

assert_eq!(s.mean_amp(), -0.09854721);
assert_eq!(s.min_amp(), -1.56928);
assert_eq!(s.max_amp(), 1.52064);

s.y.iter_mut().for_each(|v| *v *= 2.0);

s.extrema_amp();

assert_eq!(s.mean_amp(), -0.09854721 * 2.0);
assert_eq!(s.min_amp(), -1.56928 * 2.0);
assert_eq!(s.max_amp(), 1.52064 * 2.0);

s.set_string(SacString::Network, "CI");
s.set_string(SacString::Station, "PAS");
s.set_string(SacString::Location, "10");
s.set_string(SacString::T1, "PKIKP");
s.set_string(SacString::T1, "SKJKS");

assert_eq!(s.dist_deg(), 3.3574646);

s.to_file("tests/main.sac")?;

# std::fs::remove_file("tests/main.sac")?;
# Ok::<(), SacError>(())
```
*/
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::io::prelude::*;
use geographiclib::Geodesic;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Datelike;
use chrono::Timelike;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt, ReadBytesExt, NativeEndian};

mod enums;
use enums::*;
pub use enums::SacString;
pub use enums::SacZeroTime;
pub use enums::SacFileType;
pub use enums::SacDataType;

#[cfg(target_endian = "big")]
type NonNativeEndian = LittleEndian;
#[cfg(target_endian = "big")]
type __NativeEndian = BigEndian;

#[cfg(target_endian = "little")]
type NonNativeEndian = BigEndian;
#[cfg(target_endian = "little")]
type __NativeEndian = LittleEndian;

//const HEADER_SIZE : usize = 632;
const SAC_INT_UNDEF : i32 = -12345;
const SAC_FLOAT_UNDEF : f32 = -12345.0;
const SAC_STRING_UNDEF : &'static str = "-12345  ";

#[inline]
fn fis(x: f32) -> bool {
    x != SAC_FLOAT_UNDEF
}
#[inline]
fn iis(x: i32) -> bool {
    x != SAC_INT_UNDEF
}

#[macro_use] mod macros;
mod eq;

pub mod doc;

/// Value containing an absolute or relative time
pub enum TimeValue {
    /// Relative time in seconds
    Relative(Duration),
    /// Absolute time year-month-day, HH:MM:SS
    Absolute(NaiveDateTime),
}


/// Convert [u8] to Strings
fn sac_u8_to_strings(s: &mut Sac) {
    sac_strings_pair!(s, u8_to_string);
}

/// Convert Strings into [u8]
fn sac_strings_to_u8(s: &mut Sac) {
    sac_strings_pair!(s, string_to_u8);
}

/// Read a single data component
fn sac_data_read_comp<T: Read>(file: &mut T, swap: bool, npts: usize) -> Result<Vec<f32>,SacError>{
    let mut y = vec![0.0; npts];
    if swap {
        file.read_f32_into::<NonNativeEndian>(&mut y)?;
    } else {
        file.read_f32_into::<NativeEndian>(&mut y)?;
    }
    Ok(y)
}
/// Read sac data from a file
fn sac_data_read<T: Read>(file: &mut T, h: &mut Sac) -> Result<(),SacError>{
    let npts = h.npts as usize;
    h.y = sac_data_read_comp(file, h.swap, npts)?;
    if h.ncomps() == 2 {
        h.x = sac_data_read_comp(file, h.swap, npts)?;
    }
    Ok(())
}

/// Write a sac data to a file
fn sac_data_write<F: Write>(file: &mut F, s: &mut Sac, npts: usize) -> Result<(),SacError> {
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

/// Determine if the sac file is byte swapped
fn sac_header_is_swapped<T: Read + Seek>(file: &mut T) -> Result<bool,SacError> {
    use std::io::SeekFrom;
    file.seek(SeekFrom::Start(70*4 + 6*4))?;
    let n = file.read_i32::<NativeEndian>()?;

    let swap = if n > 5 && n <= 8 {
        false
    } else {
        file.seek(SeekFrom::Start(70*4 + 6*4))?;
        let n = file.read_i32::<NonNativeEndian>()?;
        if n < 0 || n > 10 {
            panic!("Unknown file type: {}", n);
        }
        true
    };
    file.seek(SeekFrom::Start(0))?;

    Ok(swap)
}

/// Read a sac file header
fn sac_header_read<T: Read + Seek>(file: &mut T, h: &mut Sac) -> Result<(),SacError>{
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


/// Write a sac file header
fn sac_header_write<F: Write>(file: &mut F, s: &mut Sac) -> Result<(),SacError>{

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

use std::fmt;
impl fmt::Debug for Sac {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sac {{ file: {}, npts: {} }}", self.file, self.npts)
    }
}




/// Errors associated with Reading and Writing Sac Files
#[derive(Debug)]
pub enum SacError {
    NotSpectral,
    NotTime,
    NaN,
    BadLatitude,
    BadLongitude,
    BadAzimuth,
    BadInclination,
    Io(std::io::Error),
    BadKey,
}

impl std::fmt::Display for SacError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SacError::NotSpectral => write!(f, "Not a spectral file"),
            SacError::NotTime => write!(f, "Not a time value"),
            SacError::NaN => write!(f, "NaN encountered in data"),
            SacError::BadLatitude => write!(f, "Invalid Latitude value"),
            SacError::BadLongitude => write!(f, "Invalid Longitude value"),
            SacError::BadAzimuth => write!(f, "Invalid Azimuth value"),
            SacError::BadInclination => write!(f, "Invalid Inclination value"),
            SacError::BadKey => write!(f, "Invalid key"),
            SacError::Io(e) => write!(f, "{}", e),
        }
    }
}

/// Wrap an std::io::Error
impl From<std::io::Error> for SacError {
    fn from(err: std::io::Error) -> Self {
        SacError::Io(err)
    }
}

fn duration_to_f64(dt: Duration) -> f64 {
    dt.num_seconds() as f64 + (dt.num_milliseconds() as f64 / 1_000.0)
}

fn time_from_parts(year: i32, doy: i32,
                   hour: i32, min: i32, sec: i32, msec: i32) -> NaiveDateTime {
    NaiveDateTime::new(NaiveDate::from_yo(year, doy as u32),
                       NaiveTime::from_hms_milli(hour as u32,
                                                 min as u32,
                                                 sec as u32,
                                                 msec as u32))
}

/// Sac Implementation
impl Sac {
    /// Read a sac file
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.delta(), 0.01);
    /// # Ok::<(), SacError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Sac,SacError> {
        let file = File::open(path)?;
        let mut file = BufReader::new(file);
        Sac::read(&mut file)
    }
    /// Read a sac file from a buffer
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// // This simulates having the data already in memory
    /// let mut buf = std::fs::read("tests/file.sac")?;
    /// let mut rdr = std::io::Cursor::new(&mut buf);
    ///
    /// let s = Sac::read(&mut rdr)?;
    /// assert_eq!(s.delta(), 0.01);
    /// # Ok::<(), SacError>(())
    /// ```
    pub fn read<R: Read + Seek>(buf: &mut R) -> Result<Sac,SacError> {
        let mut s = Sac::new();
        sac_header_read(buf, &mut s)?;
        sac_u8_to_strings(&mut s);
        sac_data_read(buf, &mut s)?;
        Ok(s)
    }
    /// Write a sac file
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// 
    /// s.to_file("tests/to_file.sac")?;
    ///
    /// let s2 = Sac::from_file("tests/to_file.sac")?;
    /// assert_eq!(s.delta(), s2.delta());
    /// assert_eq!(s.npts(), s2.npts());
    ///
    /// # std::fs::remove_file("tests/to_file.sac")?;
    /// # Ok::<(), SacError>(())
    /// ```
    pub fn to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(),SacError> {
        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        self.write(&mut file)
    }
    /// Write a sac file to a buffer
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// let mut buf = vec![];
    /// s.write(&mut buf)?;
    ///
    /// let mut rdr = std::io::Cursor::new(&mut buf);
    /// let s2 = Sac::read(&mut rdr)?;
    /// assert_eq!(s.delta(), s2.delta());
    /// assert_eq!(s.npts(), s2.npts());
    /// # Ok::<(), SacError>(())
    /// ```
    /// Write a SAC file
    pub fn write<W: Write>(&mut self, buf: &mut W) -> Result<(),SacError> {
        let npts = self.npts as usize;
        sac_strings_to_u8(self);
        sac_header_write(buf, self)?;
        sac_data_write(buf, self, npts)?;
        Ok(())
    }
    /// Determine if file is to be swapped on output
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.swapped(), false);
    ///
    /// let mut s = Sac::from_file("tests/file.sac.swap")?;
    /// assert_eq!(s.swapped(), true);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn swapped(&self) -> bool {
        self.swap
    }

    /// Determine if file is to be swapped on output
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.swapped(), false);
    ///
    /// s.set_swap(true);
    /// s.to_file("tests/set_swap.sac")?;
    ///
    /// let mut s = Sac::from_file("tests/set_swap.sac")?;
    /// assert_eq!(s.swapped(), true);
    ///
    /// # std::fs::remove_file("tests/set_swap.sac")?;
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn set_swap(&mut self, swap: bool) {
        self.swap = swap;
    }
    /// Create an empty SAC file
    ///
    /// ```
    /// use sacio::Sac;
    /// use sacio::SacString;
    /// use sacio::SacZeroTime;
    /// use sacio::SacFileType;
    /// use sacio::SacDataType;
    ///
    /// let s = Sac::new();
    /// assert_eq!(s.delta(), -12345.0);
    /// assert_eq!(s.string(SacString::EventName), "-12345  ");
    /// assert_eq!(s.zero_time(), SacZeroTime::None);
    /// assert_eq!(s.file_type(), SacFileType::Time);
    /// assert_eq!(s.data_type(), SacDataType::None);
    ///
    /// assert_eq!(s.version(), 6);
    /// assert_eq!(s.station_polarity(), false);
    /// assert_eq!(s.mutability(), true);
    /// assert_eq!(s.calc_dist_az(), true);
    ///
    /// assert_eq!(s.npts(), 0);
    /// ```
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
        s0.npts     = 0;
        s0.y = vec![];
        s0.x = vec![];
        s0
    }
    /// Check if file is spectral
    ///
    ///```
    /// use sacio::Sac;
    /// let s = Sac::from_amp(vec![0.,1.,2.], 0.0, 1.0);
    /// assert!( ! s.is_spectral() );
    /// ```
    pub fn is_spectral(&self) -> bool {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY |
            SacFileType::XYZ  => false,
            SacFileType::AmpPhase |
            SacFileType::RealImag => true,
        }
    }
    /// Check is file is a Real/Imaginary Pair
    ///
    ///     use sacio::Sac;
    ///     let s = Sac::from_amp(vec![0.,1.,2.],0.0, 1.0);
    ///     assert!( ! s.is_real_imag() );
    ///
    pub fn is_real_imag(&self) -> bool {
        self.iftype == SacFileType::RealImag.into()
    }
    /// Check if file is a Amplitude/Phase Pair
    ///
    ///     use sacio::Sac;
    ///     let s = Sac::from_amp(vec![0.,1.,2.],0.0, 1.0);
    ///     assert!( ! s.is_amp_phase() );
    ///
    pub fn is_amp_phase(&self) -> bool {
        self.iftype == SacFileType::AmpPhase.into()
    }
    /// Check if file is a time series file
    ///
    ///     use sacio::Sac;
    ///     let s = Sac::from_amp(vec![0.,1.,2.],0.0, 1.0);
    ///     assert!( s.is_time() );
    ///
    pub fn is_time(&self) -> bool {
        self.iftype == SacFileType::Time.into()
    }
    /// Get File type (iftype)
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use sacio::SacFileType;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.file_type(), SacFileType::Time);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn file_type(&self) -> SacFileType {
        self.iftype.into()
    }
    /// Set File type (iftype)
    pub fn set_file_type(&mut self, file_type: SacFileType) {
        self.iftype = file_type.into();
    }

    /// Determine the number of data components
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.ncomps(), 1);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn ncomps(&self) -> usize {
        match self.iftype.into() {
            SacFileType::Time |
            SacFileType::XY => {
                if self.evenly_spaced() { 1 } else { 2 }
            },
            SacFileType::XYZ => 1,
            SacFileType::RealImag |
            SacFileType::AmpPhase => 2,
        }
    }
    /// Get number of data points
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.npts(), 1000);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn npts(&self) -> i32 {
        self.npts
    }
    /// Get Header Version
    ///
    /// Should be 6
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.version(), 6);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn version(&self) -> i32 {
        self.nvhdr
    }
    /// Get Reference Time
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// let date = NaiveDate::from_yo(1981, 88);
    /// let time = NaiveTime::from_hms_milli(10, 38, 14, 0);
    /// assert_eq!(s.time()?, NaiveDateTime::new(date, time));
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn time(&self) -> Result<NaiveDateTime, SacError> {
        if iis(self.nzyear) && iis(self.nzjday) && iis(self.nzhour) &&
            iis(self.nzmin) && iis(self.nzsec) && iis(self.nzmsec) {
                Ok(time_from_parts(self.nzyear, self.nzjday,
                                   self.nzhour, self.nzmin, self.nzsec,
                                   self.nzmsec))
            } else {
                Err(SacError::NotTime)
            }
    }
    /// Set Reference Time
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// let date = NaiveDate::from_yo(1984, 29);
    /// let time = NaiveTime::from_hms_milli(15, 12, 59, 456);
    /// let when = NaiveDateTime::new(date, time);
    /// s.set_time(when);
    ///
    /// assert_eq!(s.time()?, when);
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn set_time(&mut self, time: NaiveDateTime) {
        self.nzyear = time.year();
        self.nzjday = time.ordinal() as i32;
        self.nzhour = time.hour() as i32;
        self.nzmin  = time.minute() as i32;
        self.nzsec  = time.second() as i32;
        self.nzmsec = time.nanosecond() as i32 / 1_000_000;
    }

    fn time_as_duration(&self, which: &str) -> Result<Duration, SacError> {
        let t0 = match which {
            "z" |
            "b"    => self.b ,
            "day"  => unimplemented!("Start of day timing"),
            "o"    => self.o,
            "a"    => self.a,
            "e"    => self.e,
            "t0"   => self.t0,
            "t1"   => self.t1,
            "t2"   => self.t2,
            "t3"   => self.t3,
            "t4"   => self.t4,
            "t5"   => self.t5,
            "t6"   => self.t6,
            "t7"   => self.t7,
            "t8"   => self.t8,
            "t9"   => self.t9,
            _ => return Err(SacError::BadKey),
        };
        if t0 == SAC_FLOAT_UNDEF {
            return Err(SacError::NotTime);
        }
        Ok(Duration::seconds(t0.trunc() as i64) +
            Duration::nanoseconds((t0.fract() * 1e9) as i64))
    }

    /// Get the Date and Time of a timing mark
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use chrono::Duration;
    ///
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// let dt = s.time()?;
    /// let b = s.b();
    /// assert_eq!(s.b(), 9.459999);
    /// let bt = dt + Duration::seconds(b.trunc() as i64) +
    ///               Duration::nanoseconds((b.fract() * 1e9) as i64);
    /// assert_eq!(s.datetime("b")?, bt);
    /// assert!(s.datetime("t9").is_err());
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn datetime(&self, which: &str) -> Result<NaiveDateTime, SacError> {
        let tref = self.time()?; // Absolute Reference time
        let dt = self.time_as_duration(which)?;
        Ok(tref + dt)
    }

    /// Compute the maximum amplitude
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.calc_max_amp(), s.max_amp());
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn calc_max_amp(&self) -> f32 {
        let mut vmax = self.y[0];
        for v in self.y.iter() { if *v > vmax { vmax = *v; } }
        vmax
    }
    /// Compute the minimum amplitude
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.calc_min_amp(), s.min_amp());
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn calc_min_amp(&self) -> f32 {
        let mut vmin = self.y[0];
        for v in self.y.iter() { if *v < vmin { vmin = *v; } }
        vmin
    }
    /// Compute the mean amplitude
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.calc_mean_amp(), s.mean_amp());
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn calc_mean_amp(&self) -> f32 {
        let vmean : f64 = self.y.iter().map(|x| *x as f64).sum();
        (vmean / self.npts as f64) as f32
    }
    /// Compute and set min, man and mean amplitudes of the y component
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    ///
    /// assert_eq!(s.mean_amp(), -0.09854721);
    /// assert_eq!(s.min_amp(), -1.56928);
    /// assert_eq!(s.max_amp(), 1.52064);
    ///
    /// s.y.iter_mut().for_each(|v| *v *= 2.0);
    ///
    /// s.extrema_amp();
    ///
    /// assert_eq!(s.mean_amp(), -0.09854721 * 2.0);
    /// assert_eq!(s.min_amp(), -1.56928 * 2.0);
    /// assert_eq!(s.max_amp(), 1.52064 * 2.0);
    ///
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn extrema_amp(&mut self) {
        self.depmax = self.calc_max_amp();
        self.depmin = self.calc_min_amp();
        self.depmen = self.calc_mean_amp();
    }

    /// Compute and set extremas in x and y
    ///
    ///  File Type      | y-min   | y-max    | y-mean   | x-min      | x-max
    /// ----------------|---------|----------|----------|------------|-------------
    ///  time / even    | min(amp)| min(amp) | mean(amp)| b          | e = b + (n-1)*dt
    ///  time / uneven  | min(amp)| min(amp) | mean(amp)| b = min(x) | e = max(x)
    ///  general xy     | min(amp)| min(amp) | mean(amp)| b          | e = b + (n-1)*dt
    ///  amp / phase    | -       | -        | -        | b = 0      | e = f_nyquist
    ///  real/ imag     | -       | -        | -        | b = 0      | e = f_nyquist
    ///  general xyz    | -       | -        | -        | -          | -
    ///
    ///
    pub fn extrema(&mut self) {
        self.extrema_amp();
        self.calc_be();
    }
    fn calc_be(&mut self) {
        if self.evenly_spaced() {
            match self.iftype.into() {
                SacFileType::Time |
                SacFileType::XY =>
                    self.e = self.b + self.delta * ((self.npts-1) as f32),
                SacFileType::RealImag |
                SacFileType::AmpPhase => {
                    let nfreq = if self.npts % 2 == 0 {
                        self.npts / 2
                    } else {
                        (self.npts - 1) / 2
                    };
                    self.e = self.b + self.delta * nfreq as f32;
                },
                SacFileType::XYZ => {},
            }
        } else if self.x.len() > 0 {
            let mut xmin = self.x[0];
            let mut xmax = self.x[0];
            for xi in self.x.iter() { if *xi < xmin { xmin = *xi; } }
            for xi in self.x.iter() { if *xi > xmax { xmax = *xi; } }
            self.b = xmin;
            self.e = xmax;
        }
    }
    /// Create a SAC File with new data, copying header values
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    ///
    /// let t = s.with_new_data(vec![0., 1., 0.0]);
    ///
    /// assert_eq!(t.min_amp(), 0.0);
    /// assert_eq!(t.max_amp(), 1.0);
    /// assert_eq!(t.npts(), 3);
    ///
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn with_new_data(&self, y: Vec<f32>) -> Self {
        let mut s = self.clone();
        s.y = y;
        s.npts = s.y.len() as i32;
        s.extrema();
        s
    }
    /// Create new sac from data from amplitude, begin value, `b`, and sample rate, `dt`
    ///
    ///     use sacio::Sac;
    ///     let s = Sac::from_amp(vec![0., 1., 2.], 0.0, 0.1);
    ///     assert!( s.is_time() );
    ///     assert!( s.y == &[0., 1., 2.] );
    ///
    pub fn from_amp(y: Vec<f32>, b: f64, dt: f64) -> Sac {
        let mut s = Sac::new();
        s.npts   = y.len() as i32;
        s.delta  = dt as f32;
        s.b      = b as f32;
        s.y      = y;
        s.iftype = SacFileType::Time.into();
        s.leven  = true as i32;
        s.extrema();
        s
    }

    /// Determine if all data is finite, not NaN, inf
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    ///
    /// let s = Sac::from_amp(vec![0.,1.], 0.0, 1.0);
    /// assert_eq!(s.is_finite(), true);
    ///
    /// # Ok::<(), SacError>(())
    /// ```
    ///
    pub fn is_finite(&self) -> bool {
        self.y.iter().all(|x| x.is_finite() == true)
    }
    /// Get Zero Time Equivalent
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use sacio::SacZeroTime;
    ///
    /// let s = Sac::from_file("tests/file.sac")?;
    /// assert_eq!(s.zero_time(), SacZeroTime::B);
    ///
    /// # Ok::<(), SacError>(())
    /// ```
    pub fn zero_time(&self) -> SacZeroTime {
        self.iztype.into()
    }
    /// Get Station_polarity
    pub fn station_polarity(&self) -> bool {
        self.lpspol != 0
    }
    /// Set Station polarity
    pub fn set_station_polarity(&mut self, flipped: bool) {
        self.lpspol = flipped as i32;
    }
    /// Determine is file is evenly spaced
    pub fn evenly_spaced(&self) -> bool {
        self.leven != 0
    }
    /// Determine is file should calculate the distance and azimuth (lcalda)
    pub fn calc_dist_az(&self) -> bool {
        self.lcalda != 0
    }
    /// Set if file should calculate the distance and azimuth (lcalda)
    pub fn set_calc_dist_az(&mut self, value: bool) {
        self.lcalda = value as i32;
    }
    /// Check is file can be overwritten (lovrok)
    pub fn mutability(&self) -> bool {
        self.lovrok != 0
    }
    /// Set the file as mutable (lovrok)
    pub fn set_mutability(&mut self, value: bool) {
        self.lovrok = value as i32;
    }
    /// Get Event type (ievtyp)
    pub fn event_type(&self) -> SacEventType {
        self.ievtyp.into()
    }
    /// Set Event ytpe (ievtyp)
    pub fn set_event_type(&mut self, etype: SacEventType) {
        self.ievtyp = etype.into();
    }
    /// Get Data Quality
    pub fn data_quality(&self) -> SacQuality {
        self.iqual.into()
    }
    /// Set Data Quality
    pub fn set_data_quality(&mut self, qual: SacQuality) {
        self.iqual = qual.into();
    }
    /// Get Amplitude Type (idep)
    pub fn data_type(&self) -> SacDataType {
        self.idep.into()
    }

    /// Set synthetic flag (isynth)
    pub fn synthetic(&self) -> bool {
        self.isynth == 1
    }
    /// Set synthetic flag (isynth)
    pub fn set_synthetic(&mut self, synth: bool) {
        self.isynth = if synth { 1 } else { 0 };
    }
    /// Set Zero Time Type
    pub fn set_zero_time_type(&mut self, ztype: SacZeroTime) {
        self.iztype = ztype.into();
    }
    /// Get Magnitude Type
    pub fn magnitude_type(&self) -> SacMagnitudeType {
        self.imagtyp.into()
    }
    /// Set Magnitude Type
    pub fn set_magnitude_type(&mut self, mag: SacMagnitudeType) {
        self.imagtyp = mag.into();
    }
    /// Get Magnitude Source
    pub fn magnitude_source(&self) -> SacMagnitudeSource {
        self.imagsrc.into()
    }
    /// Set Magnitude Source
    pub fn set_magnitude_source(&mut self, magsrc: SacMagnitudeSource) {
        self.imagsrc = magsrc.into();
    }
    /// Get Instrument Type
    ///
    /// This type is historical, you probably want SacStrings::Instrument
    ///
    pub fn instrument_type(&self) -> SacInstrument {
        self.iinst.into()
    }
    /// Set Instrument Type
    ///
    /// This type is historical, you probably want SacStrings::Instrument
    ///
    pub fn set_instrument_type(&mut self, itype: SacInstrument) {
        self.iinst = itype.into();
    }

    /// Set Amplitude Type (idep)
    pub fn set_amp_type(&mut self, amp_type: SacDataType) {
        self.idep = amp_type.into();
    }
    /// Compute Distance and Azimuth between station and event
    pub fn compute_dist_az(&mut self) {
        if ! self.calc_dist_az() {
            return;
        }
        if fis(self.stlo) && fis(self.stla) && fis(self.evlo) && fis(self.evla) {
            let g = Geodesic::wgs84();
            let (a12, s12, az1, az2) = g.inverse(self.stla as f64, self.stlo as f64,
                                                 self.evla as f64, self.evlo as f64);
            self.gcarc = a12 as f32;
            self.dist  = (s12 / 1000.0) as f32;
            self.az    = az1 as f32;
            self.baz   = az2 as f32;
        }
    }
    /// Get event region
    pub fn event_region(&self) -> i32 {
        self.ievreg
    }
    /// Get station region
    pub fn station_region(&self) -> i32 {
        self.istreg
    }
    /// Update event and station regions
    ///
    /// This assumes the station and event locations are defined
    pub fn update_regions(&mut self) {
        use flinn_engdahl as fe;
        if fis(self.stlo) && fis(self.stla) {
            if let Ok(n) = fe::region_number(self.stla as f64,
                                             self.stlo as f64) {
                self.istreg = n as i32;
            }
        }
        if fis(self.evlo) && fis(self.evla) {
            if let Ok(n) = fe::region_number(self.evla as f64,
                                             self.evlo as f64) {
                self.ievreg = n as i32;
            }
        }
    }

    /// Get Current filename
    pub fn filename(&self) -> &str {
        &self.file
    }
    /// Set Filename
    pub fn set_filename(&mut self, filename: &str) {
        self.file = filename.to_string();
    }
    pub fn id(&self, key: SacInt) -> i32 {
        match key {
            SacInt::OriginID => self.norid,
            SacInt::EventID => self.nevid,
            SacInt::WaveformID => self.nwfid,
        }
    }
    pub fn set_id(&mut self, key: SacInt, value: i32) {
        match key {
            SacInt::OriginID   => self.norid = value,
            SacInt::EventID    => self.nevid = value,
            SacInt::WaveformID => self.nwfid = value,
        }
    }
    /// Return a `nslc` code for this data
    ///  net.stat.loc.chan
    ///
    /// ```
    /// use sacio::Sac;
    /// # use sacio::SacError;
    /// use sacio::SacString;
    ///
    /// let mut s = Sac::from_file("tests/file.sac")?;
    /// s.set_string(SacString::Network, "CI");
    /// s.set_string(SacString::Station, "PAS");
    /// s.set_string(SacString::Location, "");
    /// s.set_string(SacString::Channel, "BHZ");
    /// assert_eq!(s.nslc(), "CI.PAS..BHZ");
    /// # Ok::<(), SacError>(())
    /// ```
    pub fn nslc(&self) -> String {
        let mut cmp = [""; 4];
        let keys = [SacString::Network, SacString::Station,
                    SacString::Location, SacString::Channel];
        for (v,c) in keys.iter().zip(cmp.iter_mut()) {
            let s = self.string(*v);
            if s != SAC_STRING_UNDEF {
                *c = s.trim();
            }
        }
        cmp.join(".")
    }

    pub fn string(&self, key: SacString) -> &str {
        match key {
            SacString::Station     => &self.kstnm,
            SacString::EventName   => &self.kevnm,
            SacString::Hole        => &self.khole,
            SacString::Location    => &self.khole,
            SacString::O           => &self.ko,
            SacString::A           => &self.ka,
            SacString::T0          => &self.kt0,
            SacString::T1          => &self.kt1,
            SacString::T2          => &self.kt2,
            SacString::T3          => &self.kt3,
            SacString::T4          => &self.kt4,
            SacString::T5          => &self.kt5,
            SacString::T6          => &self.kt6,
            SacString::T7          => &self.kt7,
            SacString::T8          => &self.kt8,
            SacString::T9          => &self.kt9,
            SacString::EventEnd    => &self.kf,
            SacString::User0       => &self.kuser0,
            SacString::User1       => &self.kuser1,
            SacString::User2       => &self.kuser2,
            SacString::Component   => &self.kcmpnm,
            SacString::Channel     => &self.kcmpnm,
            SacString::Network     => &self.knetwk,
            SacString::DateRead    => &self.kdatrd,
            SacString::Instrument  => &self.kinst,
        }
    }
    pub fn set_string(&mut self, key: SacString, value: &str) {
        let v = match key {
            SacString::Station     => &mut self.kstnm,
            SacString::EventName   => &mut self.kevnm,
            SacString::Hole        => &mut self.khole,
            SacString::Location    => &mut self.khole,
            SacString::O           => &mut self.ko,
            SacString::A           => &mut self.ka,
            SacString::T0          => &mut self.kt0,
            SacString::T1          => &mut self.kt1,
            SacString::T2          => &mut self.kt2,
            SacString::T3          => &mut self.kt3,
            SacString::T4          => &mut self.kt4,
            SacString::T5          => &mut self.kt5,
            SacString::T6          => &mut self.kt6,
            SacString::T7          => &mut self.kt7,
            SacString::T8          => &mut self.kt8,
            SacString::T9          => &mut self.kt9,
            SacString::EventEnd    => &mut self.kf,
            SacString::User0       => &mut self.kuser0,
            SacString::User1       => &mut self.kuser1,
            SacString::User2       => &mut self.kuser2,
            SacString::Component   => &mut self.kcmpnm,
            SacString::Channel     => &mut self.kcmpnm,
            SacString::Network     => &mut self.knetwk,
            SacString::DateRead    => &mut self.kdatrd,
            SacString::Instrument  => &mut self.kinst,
        };
        *v = value.to_string();
    }
    /// Get Sampling 
    pub fn delta(&self) -> f32 { self.delta  }
    /// Get Mean amplitude value
    pub fn mean_amp(&self)  -> f32 { self.depmen }
    /// Get Minimum amplitude value
    pub fn min_amp(&self)   -> f32 { self.depmin }
    /// Get Maximum amplitude value
    pub fn max_amp(&self)   -> f32 { self.depmax }

    pub fn station_lat(&self) -> f32 { self.stla }
    pub fn station_lon(&self) -> f32 { self.stlo }
    pub fn station_elevation(&self) -> f32 { self.stel }
    pub fn event_lat(&self)   -> f32 { self.evla }
    pub fn event_lon(&self)   -> f32 { self.evlo }
    pub fn event_depth(&self) -> f32 { self.evdp }
    pub fn dist_km(&self) -> f32 { self.dist }
    pub fn dist_deg(&self) -> f32  { self.gcarc }
    pub fn az(&self) -> f32 { self.az }
    pub fn baz(&self) -> f32 { self.baz }
    pub fn set_station_location(&mut self, lat: f32, lon: f32, elev: f32) -> Result<(),SacError>{
        if lat.abs() > 90.0 {
            return Err(SacError::BadLatitude);
        }
        if lon.abs() > 360.0 {
            return Err(SacError::BadLongitude);
        }
        self.stla = lat;
        self.stlo = lon;
        self.stel = elev;
        self.update_regions();
        self.compute_dist_az();
        Ok(())
    }
    pub fn set_event_location(&mut self, lat: f32, lon: f32, depth: f32) -> Result<(),SacError>{
        if lat.abs() > 90.0 {
            return Err(SacError::BadLatitude);
        }
        if lon.abs() > 360.0 {
            return Err(SacError::BadLongitude);
        }
        self.evla = lat;
        self.evlo = lon;
        self.evel = depth;
        self.update_regions();
        self.compute_dist_az();
        Ok(())
    }
    /// Get Component Azimuth
    ///
    /// North is 0 degrees, with positive values rotating clockwise.
    /// This is a geographic coordinate system.
    ///
    ///  Direction | Value
    ///  ----------|------
    ///   North    |   0
    ///   East     |  90
    ///   South    | 180
    ///   West     | 270 or -90
    ///
    pub fn cmpaz(&self) -> f32 {
        self.cmpaz
    }
    /// Set Component Azimuth
    ///
    /// Accepted must be between [-360, 360]
    ///
    pub fn set_cmpaz(&mut self, az: f32) -> Result<(),SacError> {
        if az.abs() > 360.0 {
            return Err(SacError::BadAzimuth);
        }
        self.cmpaz = az;
        Ok(())
    }
    /// Get Component Inclination or Incident angle
    ///
    /// Values are defined from vertical
    ///
    ///  Diretion    | Value
    ///  ------------|------
    ///  Vertical Up | 0
    ///  Horizontal  | 90
    ///
    pub fn cmpinc(&self) -> f32 {
        self.cmpaz
    }
    /// Set Component Inclination
    ///
    /// Values must be between [-180, 180]
    pub fn set_cmpinc(&mut self, inc: f32) -> Result<(),SacError> {
        if inc.abs() > 180.0 {
            return Err(SacError::BadInclination)
        }
        self.cmpinc = inc;
        Ok(())
    }
    /// Get Beginning time value
    pub fn b(&self) -> f32 { self.b }
    /// Get Ending time value
    pub fn e(&self) -> f32 { self.e }
    /// Get Origin time value
    pub fn o(&self) -> f32 { self.o }

    /// Set beginning time value
    pub fn set_b(&mut self, time: TimeValue) -> Result<(), SacError> {
        match time {
            TimeValue::Relative(v) => self.b = duration_to_f64(v) as f32,
            TimeValue::Absolute(v) => {
                // Requires knowledge of the reference time
                let dt = self.time()? - v;
                self.b = duration_to_f64(dt) as f32;
            },
        }
        self.calc_be();
        Ok(())
    }
    /// Set origin time value
    pub fn set_o(&mut self, time: TimeValue) -> Result<(), SacError> {
        match time {
            TimeValue::Relative(v) => self.o = duration_to_f64(v) as f32,
            TimeValue::Absolute(v) => {
                // Requires knowledge of the reference time
                let dt = self.time()? - v;
                self.o = duration_to_f64(dt) as f32;
            }
        }
        Ok(())
    }
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
        let s = Sac::from_amp(vec![1.,2.,3.], 0., 1.);
        assert!(s.is_finite())
    }
    #[test]
    fn test_finite_pos_inf() {
        let s = Sac::from_amp(vec![1.,2.,1./0.], 0., 1.);
        assert!(!s.is_finite())
    }
    #[test]
    fn test_finite_neg_inf() {
        let s = Sac::from_amp(vec![1.,2.,-1./0.], 0., 1.);
        assert!(!s.is_finite())
    }


    #[test]
    fn read_file() {
        use std::path::Path;
        let mut s = Sac::from_file("tests/file.sac.swap").unwrap();
        s.file = String::from("tests/file.sac.swap");

        let mut s0 = Sac::new();
        s0.file = String::from("tests/file.sac.swap true");
        s0.set_time(time_from_parts(1981, 88, 10, 38, 14, 0));
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
        let path = Path::new("tests/tmp.sac");
        s.to_file(path).unwrap();

        println!("write file with long kevnm");
        s.kevnm = format!("{}", "123456789012345678901234567890");
        let path = Path::new("tests/tmp2.sac");
        s.to_file(path).unwrap();

        println!("write file with short kevnm");
        s.kevnm = format!("{}", "12");
        let path = Path::new("tests/tmp3.sac");
        s.to_file(path).unwrap();
        {
            for elem in s.y.iter_mut() {
                *elem += 1.0;
            }
        }
        std::fs::remove_file("tests/tmp.sac").unwrap();
        std::fs::remove_file("tests/tmp2.sac").unwrap();
        std::fs::remove_file("tests/tmp3.sac").unwrap();
    }
    #[test]
    fn stringy() {
        let mut s = Sac::new();
        s.set_string(SacString::Network, "IU");
        assert!(s.string(SacString::Network) == "IU");
    }

}

/// SAC file data and metadata
///
#[derive(Default, Clone)]
pub struct Sac {
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
    swap: bool,

    /// Time sampling
    delta: f32,               /* RF time increment, sec    */
    /// Miniumum Value of y
    depmin: f32,               /*    minimum amplitude      */
    /// Maximum value of y
    depmax: f32,               /*    maximum amplitude      */
    scale: f32,                /*    amplitude scale factor */
    odelta: f32,               /*    observed time inc      */
    /// Begin time value of the data
    b: f32,                    /* RD initial value, time    */
    /// End time value of the data
    e: f32,                    /* RD final value, time      */
    o: f32,                    /*    event start, sec < nz. */
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
    stla: f32,                 /*  T station latititude     */
    stlo: f32,                 /*  T station longitude      */
    stel: f32,                 /*  T station elevation, m   */
    stdp: f32,                 /*  T station depth, m      */
    evla: f32,                 /*    event latitude         */
    evlo: f32,                 /*    event longitude        */
    evel: f32,                 /*    event elevation        */
    evdp: f32,                 /*    event depth            */
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
    /// Header Version, Should be 6
    nvhdr: i32,                  /*    internal use           */
    norid: i32,                  /*    origin ID              */
    nevid: i32,                  /*    event ID               */
    /// Number of data points
    npts: i32,                   /* RF number of samples      */
    nsnpts: i32,                 /*    internal use           */
    nwfid: i32,                  /*    waveform ID            */
    nxsize: i32,                 /*    reserved for future use */
    nysize: i32,                 /*    reserved for future use */
    unused15: i32,               /*    reserved for future use */
    /// file_type(), set_file_type(), is_amp_phase(), is_real_imag(), is_spectral()
    iftype: i32,                 /* RA type of file          */
    /// amp_type(), set_amp_type()
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

    /* End of Tradiitonal Header */
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


}


/*
/// String formatting of sac header data
fn strfmt(s: &Sac, fmt: &str) -> String {
    let mut out = String::new();
    let mut b = fmt.chars();
    let t = s.time();
    while let Some(p) = b.next() {
        if p == '%' {
            if let Some(p2) = b.next() {
                match p2 {
                    '%' => out.push('%'),
                    '+' => {
                        if s.nzyear == SAC_INT_UNDEF || s.nzjday == SAC_INT_UNDEF ||
                            s.nzhour == SAC_INT_UNDEF || s.nzmin == SAC_INT_UNDEF ||
                            s.nzsec == SAC_INT_UNDEF { continue; }
                        out += & strfmt(s, "%Y-%m-%dT%H:%M:%S.%f");
                    }
                    'Y' => vore!(s.nzyear, out, "{:04}", i),
                    'J' => vore!(s.nzjday, out, "{:02}", i),
                    'd' => vore!(t.ordinal(),   out, "{:02}", i),
                    'm' => vore!(t.month(), out, "{:02}", i),
                    'H' => vore!(s.nzhour, out, "{:02}",i),
                    'M' => vore!(s.nzmin, out, "{:02}", i),
                    'S' => vore!(s.nzsec, out, "{:02}", i),
                    'f' => vore!(s.nzmsec, out, "{:03}", i),

                    'n' => out += vore!(&s.knetwk, c),
                    's' => out += vore!(&s.kstnm, c),
                    'l' => out += vore!(&s.khole, c),
                    'c' => out += vore!(&s.kcmpnm, c),
                    'I' => out += &strfmt(s, "%n.%s.%l.%c"),

                    _ => {},
                }
            } else {
                out.push(p);
                break;
            }
        } else {
            out.push(p);
        }

    }
    out
}

    #[test]
    fn fmt() {
        println!("time");
        let mut s = Sac::from_amp(vec![1.,2.,3.], 0.0, 1.0);
        println!("time");
        let f = strfmt(&s, "thing");
        println!("time");
        assert_eq!(f, "thing");
        let f = strfmt(&s, "thing%Y-%m-%dT%H:%M:%S");
        assert_eq!(f, "thing--T::");
        println!("time");
        let f = strfmt(&s, "thing%+");
        assert_eq!(f, "thing");

        s.set_time(Time::new(1976, 27, 03, 23, 0,  23).unwrap());
        let f = strfmt(&s, "thing%Y-%m-%dT%H:%M:%S");
        assert_eq!(f, "thing1976-01-27T03:23:00");
        let f = strfmt(&s, "thing%+");
        assert_eq!(f, "thing1976-01-27T03:23:00.023");

        let f = strfmt(&s, "thing%n%s%l%c");
        assert_eq!(f, "thing");
        let f = strfmt(&s, "thing%I");
        assert_eq!(f, "thing...");

        s.kstnm = "PAS".to_string();
        s.knetwk = "CI".to_string();
        s.khole= "00".to_string();
        s.kcmpnm= "BHZ".to_string();
        let f = strfmt(&s, "thing%n%s%l%c");
        assert_eq!(f, "thingCIPAS00BHZ");
        let f = strfmt(&s, "thing%I");
        assert_eq!(f, "thingCI.PAS.00.BHZ");

        let f = strfmt(&s, "thing%x");
        assert_eq!(f, "thing");
        s.stlo = 40.1234;
    }
/// Value Or Empty (v_or_e)
macro_rules! vore {
    ($x: expr, $out: expr, $f: expr, i) => {
        if $x as i32 != SAC_INT_UNDEF {
            $out += & format!($f, $x);
        }
    };
    ($x: expr, $out: expr, $f: expr, f) => {
        if $x != SAC_FLOAT_UNDEF {
            $out += & format!($f, $x);
        }
    };
    ($x: expr, c) => { if $x == SAC_STRING_UNDEF { "" } else { $x } }
}

*/
