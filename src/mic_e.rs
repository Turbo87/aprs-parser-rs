use std::io::Write;

use Callsign;
use DecodeError;
use EncodeError;
use Latitude;
use Precision;

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
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct AprsMicE {}

impl AprsMicE {
    pub fn decode(b: &[u8], to: Callsign, new: bool) -> Result<Self, DecodeError> {
        let dest = decode_callsign(to);
        dbg!(dest);

        Ok(Self {})
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        todo!()
    }

    pub fn encode_destination(&self) -> Callsign {
        todo!()
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

#[derive(Debug)]
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

#[derive(Debug)]
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

fn decode_callsign(c: Callsign) -> Option<(Latitude, Precision, Message, LongOffset, LongDir)> {
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
