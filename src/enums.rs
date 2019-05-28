
const IUNDEF : i32 = -12345;

const ITIME : i32 = 1;
const IRLIM : i32 = 2;
const IAMPH : i32 = 3;
const IXY   : i32 = 4;
const IXYZ  : i32 = 51;

const IDIS : i32 = 6;
const IVEL : i32 = 7;
const IACC : i32 = 8;
const IVOLTS : i32 = 50;

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


/// Type of Data
///
/// Present in idep 
#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SacDataType {
    None         = IUNDEF,
    Displacement = IDIS,
    Velocity     = IVEL,
    Acceleration = IACC,
    Volts        = IVOLTS,
}

/// Type of file contents
///
/// Present in the iftype value
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

/// Zero time equivalence
///
/// Present in iztype
#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone)]
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

/// Type of Event
///
/// Present in ievtyp
#[repr(i32)]
pub enum SacEventType {
    /// No Event Type
    None              = IUNDEF,
    /// Nuclear Event
    NuclearShot       = INUKE,
    /// Nuclear Preshot Event
    NuclearPreShot    = INUKEPRE,
    /// Nuclear Post Shot Event
    NuclearPostShot   = INUKEPOST,
    /// Earthquake
    Earthquake        = IEQ,
    /// Foreshock
    Foreshock         = IFORE,
    /// Aftershock
    Aftershock        = IAFTER,
    /// Chemicla Explosion
    ChemicalExplosion = ICHEM,
    /// Other
    Other             = IOTHER,
    /// Quarry or mine Blast, confirmed by Quarry
    QuarryBlast       = IQUARRY,
    /// Quarry or mine blast with designed shot information-ripple fired
    QuarryBlast1      = IQUARRY1,
    /// Quarry or mine blast with observed shot information-ripple fired
    QuarryBlast2      = IQUARRY2,
    /// Quarry or mine blast - single shot
    QuarryBlastSingle = 75,
    /// Quarry or mining-induced events: tremors and rockbursts
    QuarryTremor = 76,
    // Earthquake Unsed
    //_Earthquake = 77,
    EarthquakeSwarm = 78,
    /// Felt Earthquake
    FeltEarthquake = 79,
    /// Marine Explosion
    MarineExplosion = 80,
    /// Explosion
    Explosion = 81,
    // Nuclear Explosion (
    //_NuclearExplosion = 82,
    /// Nuclear Cavity Collapse
    NuclearCavityCollapse = 83,
    /// Unknown
    Unknown = 84,
    /// Local Event
    LocalEvent = 85,
    /// Regional Event
    RegionalEvent = 86,
    /// Teleseismic Event
    TeleseismicEvent = 87,
    /// Undetermined or Conflicting Information
    Undetermined = 88,
    /// Damaging Earthquake
    DamagingEarthquake = 89,
    /// Probable Earthquake
    ProbableEarthquake = 90,
    /// Probable Explosion
    ProbableExplosion = 91,
    /// Mine Collapse
    MineCollapse = 92,
    /// Probable Mine Blast
    ProbableMineBlast = 93,
    /// Geyser
    Geyser = 94,
    /// Light
    Light = 95,
    /// Meteoric Event
    MeteoricEvent = 96,
    /// Odor Event 
    Odors = 97,
    /// Other Unknown
    OtherUnknown = 103,
}

/// Instrument Type
///
/// Present in iinst
#[repr(i32)]
pub enum SacInstrument {
    /// Radial NTS
    RadNV = 23,
    /// Tangential NTS
    TanNV = 24,
    /// Radianl Event
    RadEV = 25,
    /// Tangential Event
    TanEV = 26,
    /// North Positive
    North = 27,
    /// East Positive
    East = 28,
    /// Horizontal 
    Horizontal = 29,
    /// Down Positive
    Down = 30,
    /// Up Positive
    Up = 31,
    /// LLL Broadband
    LLLBB = 32,
    /// WWSSN 15 - 100
    WWSSN1 = 33,
    /// WWSSN 30 - 100
    WWSSN2 = 34,
    /// High Gain, Intermediate Period
    HighGainLP = 35,
    /// SRO
    SRO = 36,
}

/// Magnitude type
///
/// Present in imagtyp
#[repr(i32)]
pub enum SacMagnitudeType {
    BodyWave     = 52,
    SurfaceWave  = 53,
    Local = 54,
    Moment = 55,
    Duration = 56,
    UserDefined = 57,
}
/// Magnitude Source
#[repr(i32)]
pub enum SacMagnitudeSource {
    NEIC = 58,
    PDEQ = 59,
    PDEW = 60,
    PDE = 61,
    ISC = 62,
    REB = 63,
    USGS = 64,
    Berkeley = 65,
    Caltech = 66,
    LLNL = 67,
    EVLOC = 68,
    JSOP = 69,
    User = 70,
    Unknown = 71,
}

/// Data Quality
///
/// Present in iqual
#[repr(i32)]
pub enum SacQuality {
    Good = 45,
    Glitches = 46,
    Dropouts = 47,
    LowSNR = 48,
}

impl From<SacZeroTime> for i32 {
    fn from(t: SacZeroTime) -> i32 {
        t as i32
    }
}
impl From<SacMagnitudeType> for i32 {
    fn from(t: SacMagnitudeType) -> i32 {
        t as i32
    }
}
impl From<SacMagnitudeSource> for i32 {
    fn from(t: SacMagnitudeSource) -> i32 {
        t as i32
    }
}
impl From<SacQuality> for i32 {
    fn from(t: SacQuality) -> i32 {
        t as i32
    }
}
impl From<SacFileType> for i32 {
    fn from(t: SacFileType) -> i32 {
        t as i32
    }
}
impl From<i32> for SacMagnitudeType {
    fn from(t: i32) -> Self {
        match t {
            52 => SacMagnitudeType::BodyWave,
            53 => SacMagnitudeType::SurfaceWave,
            54 => SacMagnitudeType::Local,
            55 => SacMagnitudeType::Moment,
            56 => SacMagnitudeType::Duration,
            57 => SacMagnitudeType::UserDefined,
            _ => panic!("Unknown Sac Magnitude Type: {}", t),
        }
    }
}
impl From<i32> for SacMagnitudeSource {
    fn from(t: i32) -> Self {
        match t {
            58 => SacMagnitudeSource::NEIC,
            59 => SacMagnitudeSource::PDEQ,
            60 => SacMagnitudeSource::PDEW,
            61 => SacMagnitudeSource::PDE,
            62 => SacMagnitudeSource::ISC,
            63 => SacMagnitudeSource::REB,
            64 => SacMagnitudeSource::USGS,
            65 => SacMagnitudeSource::Berkeley,
            66 => SacMagnitudeSource::Caltech,
            67 => SacMagnitudeSource::LLNL,
            68 => SacMagnitudeSource::EVLOC,
            69 => SacMagnitudeSource::JSOP,
            70 => SacMagnitudeSource::User,
            71 => SacMagnitudeSource::Unknown,
            _ => panic!("Unknown Sac Magnitude Source: {}", t),
        }
    }
}

impl From<i32> for SacQuality {
    fn from(t: i32) -> SacQuality {
        match t {
            45 => SacQuality::Good,
            46 => SacQuality::Glitches,
            47 => SacQuality::Dropouts,
            48 => SacQuality::LowSNR,
            _ => panic!("Unknown Sac Data Quality: {}", t),
        }
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
impl From<SacInstrument> for i32 {
    fn from(t: SacInstrument) -> i32 {
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

impl From<i32> for SacInstrument {
    fn from(t: i32) -> Self {
        match t {
            23 => SacInstrument::RadNV,
            24 => SacInstrument::TanNV,
            25 => SacInstrument::RadEV,
            26 => SacInstrument::TanEV,
            27 => SacInstrument::North,
            28 => SacInstrument::East,
            29 => SacInstrument::Horizontal,
            30 => SacInstrument::Down,
            31 => SacInstrument::Up,
            32 => SacInstrument::LLLBB,
            33 => SacInstrument::WWSSN1,
            34 => SacInstrument::WWSSN2,
            35 => SacInstrument::HighGainLP,
            36 => SacInstrument::SRO,
            _ => panic!("Unknown Instrument Type: {}", t),
        }
    }
}

pub enum SacInt {
    /// Origin ID
    OriginID,
    /// Event ID
    EventID,
    /// Waveform ID
    WaveformID,
}

/// Available String Meta Data
pub enum SacString {
    /// Station Name
    Station,
    /// Event Name
    EventName,
    /// Instrument Location at Site
    Hole,
    /// Instrument Location at Site
    Location,
    /// Origin Identifier
    O,
    /// First Arrival Identifier
    A,
    /// Time Pick Identifier
    T0,
    /// Time Pick Identifier
    T1,
    /// Time Pick Identifier
    T2,
    /// Time Pick Identifier
    T3,
    /// Time Pick Identifier
    T4,
    /// Time Pick Identifire
    T5,
    /// Time Pick Identifier
    T6,
    /// Time Pick Identifier
    T7,
    /// Time Pick Identifier
    T8,
    /// Time Pick Identifier
    T9,
    /// Event End Identifier
    EventEnd,
    /// User available String
    User0,
    /// User available String 
    User1,
    /// User available String 
    User2,
    /// Component Name
    Component,
    /// Network Name
    Network,
    /// Date Data was Read
    DateRead,
    /// Instrument Name
    Instrument,
}
