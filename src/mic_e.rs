use std::io::Write;

use Callsign;
use DecodeError;
use EncodeError;
use Latitude;
use Precision;

use crate::Longitude;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Message {
    M0,
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    Emergency,
    Unknown,
}

impl Message {
    fn decode(a: MessageBit, b: MessageBit, c: MessageBit) -> Self {
        use self::Message::*;
        use self::MessageBit::{CustomOne, StandardOne, Zero};

        match (a, b, c) {
            (StandardOne, StandardOne, StandardOne) => M0,
            (CustomOne, CustomOne, CustomOne) => C0,

            (StandardOne, StandardOne, Zero) => M1,
            (CustomOne, CustomOne, Zero) => C1,

            (StandardOne, Zero, StandardOne) => M2,
            (CustomOne, Zero, CustomOne) => C2,

            (StandardOne, Zero, Zero) => M3,
            (CustomOne, Zero, Zero) => C3,

            (Zero, StandardOne, StandardOne) => M4,
            (Zero, CustomOne, CustomOne) => C4,

            (Zero, StandardOne, Zero) => M5,
            (Zero, CustomOne, Zero) => C5,

            (Zero, Zero, StandardOne) => M6,
            (Zero, Zero, CustomOne) => C6,

            (Zero, Zero, Zero) => Self::Emergency,
            _ => Self::Unknown,
        }
    }

    fn encode(&self) -> (MessageBit, MessageBit, MessageBit) {
        use self::Message::*;
        use self::MessageBit::{CustomOne, StandardOne, Zero};

        match self {
            M0 => (StandardOne, StandardOne, StandardOne),
            C0 => (CustomOne, CustomOne, CustomOne),

            M1 => (StandardOne, StandardOne, Zero),
            C1 => (CustomOne, CustomOne, Zero),

            M2 => (StandardOne, Zero, StandardOne),
            C2 => (CustomOne, Zero, CustomOne),

            M3 => (StandardOne, Zero, Zero),
            C3 => (CustomOne, Zero, Zero),

            M4 => (Zero, StandardOne, StandardOne),
            C4 => (Zero, CustomOne, CustomOne),

            M5 => (Zero, StandardOne, Zero),
            C5 => (Zero, CustomOne, Zero),

            M6 => (Zero, Zero, StandardOne),
            C6 => (Zero, Zero, CustomOne),

            Message::Emergency => (Zero, Zero, Zero),
            // any combination of standard and custom ones would work here
            Message::Unknown => (StandardOne, CustomOne, StandardOne),
        }
    }
}

/// A speed. Valid values range from 0 to 799 knots.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Speed(u32);

impl Speed {
    /// Creates a new `Speed` from knots.
    pub fn new(knots: u32) -> Option<Self> {
        if knots > 799 {
            return None;
        }

        Some(Self(knots))
    }

    pub fn knots(&self) -> u32 {
        self.0
    }
}

/// A course. Valid values range from 0 to 360 degrees.
/// 0 degrees represents an unknown course.
/// 360 degrees represents north.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Course(u32);

impl Course {
    pub const UNKNOWN: Self = Self(0);

    /// Creates a new `Course` from degrees.
    pub fn new(degrees: u32) -> Option<Self> {
        if degrees > 360 {
            return None;
        }

        Some(Self(degrees))
    }

    pub fn degrees(&self) -> u32 {
        self.0
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct AprsMicE {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub precision: Precision,

    pub message: Message,
    pub speed: Speed,
    pub course: Course,
    pub symbol_table: char,
    pub symbol_code: char,
    pub comment: Vec<u8>,

    pub current: bool,
}

impl AprsMicE {
    pub fn decode(b: &[u8], to: Callsign, current: bool) -> Result<Self, DecodeError> {
        let (latitude, precision, message, long_offset, long_dir) =
            decode_callsign(&to).ok_or(DecodeError::InvalidMicEDestination(to))?;

        let info = b
            .get(0..8)
            .ok_or_else(|| DecodeError::InvalidMicEInformation(b.to_vec()))?;
        let comment = b.get(8..).unwrap_or(&[]).to_vec();

        let longitude = decode_longitude(&info[0..3], long_offset, long_dir)
            .ok_or_else(|| DecodeError::InvalidMicEInformation(b.to_vec()))?;
        let (speed, course) = decode_speed_and_course(&info[3..6])
            .ok_or_else(|| DecodeError::InvalidMicEInformation(b.to_vec()))?;
        let symbol_code = char::from(info[6]);
        let symbol_table = char::from(info[7]);

        Ok(Self {
            latitude,
            longitude,
            precision,

            message,
            speed,
            course,
            symbol_table,
            symbol_code,
            comment,

            current,
        })
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        todo!()
    }

    pub fn encode_destination(&self) -> Callsign {
        let mut encoded_lat = vec![];
        // safe to do
        // can only fail from a write error
        // which is impossible because we're writing to an array
        self.latitude
            .encode_uncompressed(&mut encoded_lat, self.precision)
            .unwrap();
        assert_eq!(8, encoded_lat.len());

        let lat_dir = if *self.latitude >= 0.0 {
            LatDir::North
        } else {
            LatDir::South
        };

        let long_dir = if *self.longitude >= 0.0 {
            LongDir::East
        } else {
            LongDir::West
        };

        let long_abs = self.longitude.abs();
        let long_offset = if (0.0..=9.0).contains(&long_abs) || long_abs >= 100.0 {
            LongOffset::Hundred
        } else {
            LongOffset::Zero
        };

        let (a, b, c) = self.message.encode();

        let bytes = vec![
            encode_bits_012(encoded_lat[0], a),
            encode_bits_012(encoded_lat[1], b),
            encode_bits_012(encoded_lat[2], c),
            encode_bit_3(encoded_lat[3], lat_dir),
            encode_bit_4(encoded_lat[5], long_offset),
            encode_bit_5(encoded_lat[6], long_dir),
        ];

        // Safe to unwrap because we know all bytes are valid ASCII
        Callsign::new_no_ssid(String::from_utf8(bytes).unwrap())
    }
}

enum MessageBit {
    Zero,
    CustomOne,
    StandardOne,
}

impl MessageBit {
    fn decode(c: u8) -> Option<Self> {
        match c {
            b'0'..=b'9' | b'L' => Some(MessageBit::Zero),
            b'A'..=b'K' => Some(MessageBit::CustomOne),
            b'P'..=b'Z' => Some(MessageBit::StandardOne),
            _ => None,
        }
    }
}

enum LatDir {
    North,
    South,
}

impl LatDir {
    fn decode(c: u8) -> Option<Self> {
        match c {
            b'0'..=b'9' | b'L' => Some(LatDir::South),
            b'P'..=b'Z' => Some(LatDir::North),
            _ => None,
        }
    }

    fn byte(self) -> u8 {
        match self {
            Self::North => b'N',
            Self::South => b'S',
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LongOffset {
    Zero,
    Hundred,
}

impl LongOffset {
    fn decode(c: u8) -> Option<Self> {
        match c {
            b'0'..=b'9' | b'L' => Some(LongOffset::Zero),
            b'P'..=b'Z' => Some(LongOffset::Hundred),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LongDir {
    East,
    West,
}

impl LongDir {
    fn decode(c: u8) -> Option<Self> {
        match c {
            b'0'..=b'9' | b'L' => Some(Self::East),
            b'P'..=b'Z' => Some(Self::West),
            _ => None,
        }
    }
}

// returns the ASCII value, since we can have spaces
fn decode_latitude_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c),
        b'A'..=b'J' => Some(c - 17),
        b'K' | b'L' | b'Z' => Some(b' '),
        b'P'..=b'Y' => Some(c - 32),
        _ => None,
    }
}

fn decode_callsign(c: &Callsign) -> Option<(Latitude, Precision, Message, LongOffset, LongDir)> {
    let data = c.call().as_bytes();
    if data.len() != 6 {
        return None;
    }

    let lat_bytes = [
        decode_latitude_digit(data[0])?,
        decode_latitude_digit(data[1])?,
        decode_latitude_digit(data[2])?,
        decode_latitude_digit(data[3])?,
        b'.',
        decode_latitude_digit(data[4])?,
        decode_latitude_digit(data[5])?,
        LatDir::decode(data[3])?.byte(),
    ];

    let (lat, precision) = Latitude::parse_uncompressed(&lat_bytes).ok()?;

    let a = MessageBit::decode(data[0])?;
    let b = MessageBit::decode(data[1])?;
    let c = MessageBit::decode(data[2])?;

    let msg = Message::decode(a, b, c);

    let long_offset = LongOffset::decode(data[4])?;

    let long_dir = LongDir::decode(data[5])?;

    Some((lat, precision, msg, long_offset, long_dir))
}

fn decode_longitude(b: &[u8], offset: LongOffset, dir: LongDir) -> Option<Longitude> {
    if b.len() != 3 {
        return None;
    }

    let mut d = b[0] - 28;

    if offset == LongOffset::Hundred {
        d += 100;
    }

    if d >= 180 && d <= 189 {
        d -= 80;
    } else if d >= 190 && d <= 199 {
        d -= 190;
    }

    let mut m = b[1] - 28;

    if m >= 60 {
        m -= 60;
    }

    let h = b[2] - 28;

    Longitude::from_dmh(d.into(), m.into(), h.into(), dir == LongDir::East)
}

fn decode_speed_and_course(b: &[u8]) -> Option<(Speed, Course)> {
    let sp = u32::from(b[0] - 28);

    let tens_knots = sp * 10;

    let dc = u32::from(b[1] - 28);

    let units_knots = dc / 10;
    let hundreds_course = (dc % 10) * 100;

    let units_course = u32::from(b[2] - 28);

    let mut speed_knots = tens_knots + units_knots;
    if speed_knots >= 800 {
        speed_knots -= 800;
    }

    let mut course_degrees = hundreds_course + units_course;
    if course_degrees >= 400 {
        course_degrees -= 400;
    }

    let speed = Speed::new(speed_knots)?;
    let course = Course::new(course_degrees)?;

    Some((speed, course))
}

// lat_digit must be an ASCII char to allow for spaces
fn encode_bits_012(lat_digit: u8, message_bit: MessageBit) -> u8 {
    match (message_bit, lat_digit == b' ') {
        (MessageBit::Zero, false) => lat_digit,
        (MessageBit::Zero, true) => b'L',

        (MessageBit::CustomOne, false) => lat_digit + 17,
        (MessageBit::CustomOne, true) => b'K',

        (MessageBit::StandardOne, false) => lat_digit + 32,
        (MessageBit::StandardOne, true) => b'Z',
    }
}

fn encode_bit_3(lat_digit: u8, lat_dir: LatDir) -> u8 {
    match (lat_dir, lat_digit == b' ') {
        (LatDir::North, false) => lat_digit + 32,
        (LatDir::North, true) => b'Z',

        (LatDir::South, false) => lat_digit,
        (LatDir::South, true) => b'L',
    }
}

fn encode_bit_4(lat_digit: u8, long_offset: LongOffset) -> u8 {
    match (long_offset, lat_digit == b' ') {
        (LongOffset::Zero, false) => lat_digit,
        (LongOffset::Zero, true) => b'L',

        (LongOffset::Hundred, false) => lat_digit + 32,
        (LongOffset::Hundred, true) => b'Z',
    }
}

fn encode_bit_5(lat_digit: u8, long_dir: LongDir) -> u8 {
    match (long_dir, lat_digit == b' ') {
        (LongDir::East, false) => lat_digit,
        (LongDir::East, true) => b'L',

        (LongDir::West, false) => lat_digit + 32,
        (LongDir::West, true) => b'Z',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_from_u32() {
        let course = Course::new(123).unwrap();
        assert_eq!(123, course.degrees());

        let course = Course::new(360).unwrap();
        assert_eq!(360, course.degrees());
    }

    #[test]
    fn course_from_u32_smallest() {
        let course = Course::new(0).unwrap();
        assert_eq!(0, course.degrees());
    }

    #[test]
    fn course_from_u32_too_big() {
        assert_eq!(None, Course::new(361));
    }

    #[test]
    fn speed_from_u32() {
        let speed = Speed::new(545).unwrap();
        assert_eq!(545, speed.knots());

        let speed = Speed::new(799).unwrap();
        assert_eq!(799, speed.knots());
    }

    #[test]
    fn speed_from_u32_smallest() {
        let speed = Speed::new(0).unwrap();
        assert_eq!(0, speed.knots());
    }

    #[test]
    fn speed_from_u32_too_big() {
        assert_eq!(None, Speed::new(800));
        assert_eq!(None, Speed::new(801));
        assert_eq!(None, Speed::new(5237));
    }

    #[test]
    fn decode_dest_test() {
        let (latitude, precision, message, offset, dir) =
            decode_callsign(&Callsign::new_no_ssid("S32U6T")).unwrap();

        assert_eq!(Latitude::new(33.42733333333333).unwrap(), latitude);
        assert_eq!(Precision::HundredthMinute, precision);
        assert_eq!(Message::M3, message);
        assert_eq!(LongOffset::Zero, offset);
        assert_eq!(LongDir::West, dir);
    }

    #[test]
    fn decode_test() {
        // example from the APRS spec doc
        let information = &br#"(_fn"Oj/Hello world!"#[..];
        let to = Callsign::new_no_ssid("PPPPPP");

        let data = AprsMicE::decode(information, to, true).unwrap();

        assert_eq!(
            AprsMicE {
                latitude: Latitude::new(0.0).unwrap(),
                longitude: Longitude::new(-112.12899999999999).unwrap(),
                precision: Precision::HundredthMinute,
                message: Message::M0,
                speed: Speed::new(20).unwrap(),
                course: Course::new(251).unwrap(),
                symbol_table: '/',
                symbol_code: 'j',
                comment: b"Hello world!".to_vec(),
                current: true
            },
            data
        );
    }

    #[test]
    fn encode_destination_test() {
        let information = &br#"(_fn"Oj/Hello world!"#[..];
        let to = Callsign::new_no_ssid("S5PPW4");

        let data = AprsMicE::decode(information, to.clone(), true).unwrap();

        assert_eq!(to, data.encode_destination());
    }
}
