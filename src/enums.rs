
const IUNDEF : i32 = -12345;

const ITIME : i32 = 1;
const IRLIM : i32 = 2;
const IAMPH : i32 = 3;
const IXY   : i32 = 4;
const IXYZ  : i32 = 51;

/// Type of file contents
#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SacFileType {
    //None   = IUNDEF,
    //Real   = 0,
    /// Time Series file
    Time     = ITIME,
    /// Complex data: Real + Imaginary
    RealImag = IRLIM,
    /// Complex data: Amplitude + Phase
    AmpPhase = IAMPH,
    /// 2D Data
    XY       = IXY,
    /// 3D Data
    XYZ      = IXYZ,
}

const IB   : i32 =  9;
const IDAY : i32 =  10;
const IO   : i32 =  11;
const IA   : i32 =  12;
const IT0  : i32 =  13;
const IT1  : i32 =  14;
const IT2  : i32 =  15;
const IT3  : i32 =  16;
const IT4  : i32 =  17;
const IT5  : i32 =  18;
const IT6  : i32 =  19;
const IT7  : i32 =  20;
const IT8  : i32 =  21;
const IT9  : i32 =  22;

/// Zero time equivalence
#[repr(i32)]
pub enum SacZeroTime {
    None = IUNDEF,
    /// Begin Time
    B   = IB,
    /// Start of the Day
    Day = IDAY,
    /// Origin time
    O   = IO,
    /// First Arrival
    A   = IA,
    /// Timing mark 0
    T0  = IT0,
    /// Timing mark 1
    T1  = IT1,
    /// Timing mark 2
    T2  = IT2,
    /// Timing mark 3
    T3  = IT3,
    /// Timing mark 4
    T4  = IT4,
    /// Timing mark 5
    T5  = IT5,
    /// Timing mark 6
    T6  = IT6,
    /// Timing mark 7
    T7  = IT7,
    /// Timing mark 8
    T8  = IT8,
    /// Timing mark 9
    T9  = IT9,
}

const INUKE : i32 = 37;
const INUKEPRE : i32 = 38;
const INUKEPOST : i32 = 39;
const IEQ : i32 = 40;
const IFORE : i32 = 41;
const IAFTER : i32 = 42;
const ICHEM : i32 = 43;
const IOTHER : i32 = 44;
const IQUARRY : i32 = 72;
const IQUARRY1 : i32 = 73;
const IQUARRY2 : i32 = 74;

/// Type of Event
#[repr(i32)]
pub enum SacEventType {
    None              = IUNDEF,
    NuclearShot       = INUKE,
    NuclearPreShot    = INUKEPRE,
    NuclearPostShot   = INUKEPOST,
    Earthquake        = IEQ,
    Foreshock         = IFORE,
    Aftershock        = IAFTER,
    ChemicalExplosion = ICHEM,
    Other             = IOTHER,
    QuarryBlast       = IQUARRY,
    QuarryBlast1      = IQUARRY1,
    QuarryBlast2      = IQUARRY2,
}

const IDIS : i32 = 6;
const IVEL : i32 = 7;
const IACC : i32 = 8;
const IVOLTS : i32 = 50;

/// Type of Data
#[repr(i32)]
pub enum SacDataType {
    None         = IUNDEF,
    Displacement = IDIS,
    Velocity     = IVEL,
    Acceleration = IACC,
    Volts        = IVOLTS,
}
impl From<SacZeroTime> for i32 {
    fn from(t: SacZeroTime) -> i32 {
        t as i32
    }
}
impl From<SacFileType> for i32 {
    fn from(t: SacFileType) -> i32 {
        t as i32
    }
}
impl From<i32> for SacFileType {
    fn from(t: i32) -> SacFileType {
        match t {
            //IUNDEF => SacFileType::None,
            ITIME  => SacFileType::Time,
            IRLIM  => SacFileType::RealImag,
            IAMPH  => SacFileType::AmpPhase,
            IXY    => SacFileType::XY,
            IXYZ   => SacFileType::XYZ,
            _ => panic!("Unknown Sac File Type: {}", t),
        }
    }
}
impl From<SacEventType> for i32 {
    fn from(t: SacEventType) -> i32 {
        t as i32
    }
}
impl From<SacDataType> for i32 {
    fn from(t: SacDataType) -> i32 {
        t as i32
    }
}

impl Default for SacFileType {
    fn default() -> SacFileType { SacFileType::Time }
}

impl From<i32> for SacDataType {
    fn from(t: i32) -> SacDataType {
        match t {
            -12345 => SacDataType::None,
            IDIS => SacDataType::Displacement,
            IVEL => SacDataType::Velocity,
            IACC => SacDataType::Acceleration,
            IVOLTS => SacDataType::Volts,
            _ => panic!("Unknown Data Type: {}", t),
        }
    }
}

impl From<i32> for SacZeroTime {
    fn from(t: i32) -> SacZeroTime {
        match t {
            -12345 => SacZeroTime::None,
            9  => SacZeroTime::B,
            10 => SacZeroTime::Day,
            11 => SacZeroTime::O,
            12 => SacZeroTime::A,
            13 => SacZeroTime::T0,
            14 => SacZeroTime::T1,
            15 => SacZeroTime::T2,
            16 => SacZeroTime::T3,
            17 => SacZeroTime::T4,
            18 => SacZeroTime::T5,
            19 => SacZeroTime::T6,
            20 => SacZeroTime::T7,
            21 => SacZeroTime::T8,
            22 => SacZeroTime::T9,
            _ => panic!("Unknown Zero Time: {}", t),
        }
    }
}

impl From<i32> for SacEventType {
    fn from(t: i32) -> SacEventType {
        match t {
            -12345    => SacEventType::None,
            INUKE     => SacEventType::NuclearShot,
            INUKEPRE  => SacEventType::NuclearPreShot,
            INUKEPOST => SacEventType::NuclearPostShot,
            IEQ       => SacEventType::Earthquake,
            IFORE     => SacEventType::Foreshock,
            IAFTER    => SacEventType::Aftershock,
            ICHEM     => SacEventType::ChemicalExplosion,
            IOTHER    => SacEventType::Other,
            IQUARRY   => SacEventType::QuarryBlast,
            IQUARRY1  => SacEventType::QuarryBlast1,
            IQUARRY2  => SacEventType::QuarryBlast2,
            _ => panic!("Unknown Event Type: {}", t),
        }
    }
}

