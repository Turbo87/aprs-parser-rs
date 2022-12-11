use bytes::parse_bytes;
use std::convert::TryFrom;
use std::io::Write;

use AprsError;
use EncodeError;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct DhmTimestamp(u8, u8, u8);

/// Day of month, Hour and Minute in UTC
impl DhmTimestamp {
    pub fn new(d: u8, h: u8, m: u8) -> Option<Self> {
        // We could theoretically be more restrictive here
        // Is (0, 24, 0) invalid or is it the same as
        // (1, 0, 0)?
        if d <= 99 && h <= 99 && m <= 99 {
            Some(Self(d, h, m))
        } else {
            None
        }
    }
}

impl TryFrom<Timestamp> for DhmTimestamp {
    type Error = ();

    fn try_from(t: Timestamp) -> Result<Self, ()> {
        if let Timestamp::DDHHMM(d, h, m) = t {
            Ok(Self(d, h, m))
        } else {
            Err(())
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Timestamp {
    /// Day of month, Hour and Minute in UTC
    DDHHMM(u8, u8, u8),
    /// Hour, Minute and Second in UTC
    HHMMSS(u8, u8, u8),
    /// Unsupported timestamp format
    Unsupported(Vec<u8>),
}

impl Timestamp {
    /// Day of month, Hour and Minute in UTC
    pub fn new_dhm(d: u8, h: u8, m: u8) -> Option<Self> {
        if d <= 99 && h <= 99 && m <= 99 {
            Some(Self::DDHHMM(d, h, m))
        } else {
            None
        }
    }

    /// Hour, Minute and Second in UTC
    pub fn new_hms(h: u8, m: u8, s: u8) -> Option<Self> {
        if h <= 99 && m <= 99 && s <= 99 {
            Some(Self::HHMMSS(h, m, s))
        } else {
            None
        }
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        match self {
            Self::DDHHMM(d, h, m) => write!(buf, "{:02}{:02}{:02}z", d, h, m)?,
            Self::HHMMSS(h, m, s) => write!(buf, "{:02}{:02}{:02}h", h, m, s)?,
            Self::Unsupported(s) => buf.write_all(s)?,
        };

        Ok(())
    }
}

impl TryFrom<&[u8]> for Timestamp {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        if b.len() != 7 {
            return Err(AprsError::InvalidTimestamp(b.to_owned()));
        }

        if b[6] == b'/' {
            return Ok(Timestamp::Unsupported(b.to_owned()));
        }

        let one = parse_bytes(&b[0..2]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;
        let two = parse_bytes(&b[2..4]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;
        let three =
            parse_bytes(&b[4..6]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;

        Ok(match b[6] {
            b'z' | b'Z' => Timestamp::DDHHMM(one, two, three),
            b'h' | b'H' => Timestamp::HHMMSS(one, two, three),
            _ => return Err(AprsError::InvalidTimestamp(b.to_owned())),
        })
    }
}

impl From<DhmTimestamp> for Timestamp {
    fn from(t: DhmTimestamp) -> Self {
        Self::DDHHMM(t.0, t.1, t.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ddhhmm() {
        assert_eq!(
            Timestamp::try_from(&b"123456z"[..]),
            Ok(Timestamp::DDHHMM(12, 34, 56))
        );

        assert_eq!(
            Timestamp::try_from(&b"123456Z"[..]),
            Ok(Timestamp::DDHHMM(12, 34, 56))
        );
    }

    #[test]
    fn parse_hhmmss() {
        assert_eq!(
            Timestamp::try_from(&b"123456h"[..]),
            Ok(Timestamp::HHMMSS(12, 34, 56))
        );

        assert_eq!(
            Timestamp::try_from(&b"123456H"[..]),
            Ok(Timestamp::HHMMSS(12, 34, 56))
        );
    }

    #[test]
    fn parse_local_time() {
        assert_eq!(
            Timestamp::try_from(&b"123456/"[..]),
            Ok(Timestamp::Unsupported(b"123456/".to_vec()))
        );
    }

    #[test]
    fn invalid_timestamp() {
        assert_eq!(
            Timestamp::try_from(&b"1234567"[..]),
            Err(AprsError::InvalidTimestamp(b"1234567".to_vec()))
        );
    }

    #[test]
    fn invalid_timestamp2() {
        assert_eq!(
            Timestamp::try_from(&b"123a56z"[..]),
            Err(AprsError::InvalidTimestamp(b"123a56z".to_vec()))
        );
    }

    #[test]
    fn encode_ddhhmm() {
        let mut buf = vec![];
        Timestamp::DDHHMM(65, 43, 21).encode(&mut buf).unwrap();
        assert_eq!(b"654321z"[..], buf);
    }

    #[test]
    fn encode_hhmmss() {
        let mut buf = vec![];
        Timestamp::HHMMSS(65, 43, 21).encode(&mut buf).unwrap();
        assert_eq!(b"654321h"[..], buf);
    }

    #[test]
    fn encode_local_time() {
        let mut buf = vec![];
        Timestamp::Unsupported(b"135a67z".to_vec())
            .encode(&mut buf)
            .unwrap();
        assert_eq!(b"135a67z"[..], buf);
    }

    #[test]
    fn convert_dhm_timestamp_to_normal_timestamp() {
        let timestamp: Timestamp = DhmTimestamp::new(12, 34, 56).unwrap().into();
        assert_eq!(Timestamp::new_dhm(12, 34, 56).unwrap(), timestamp);
    }

    #[test]
    fn convert_timestamp_to_dhm_timestamp_success() {
        use std::convert::TryInto;

        let timestamp = Timestamp::new_dhm(65, 43, 21).unwrap();
        assert_eq!(
            DhmTimestamp::new(65, 43, 21).unwrap(),
            timestamp.try_into().unwrap()
        );
    }

    #[test]
    fn convert_timestamp_to_dhm_timestamp_failure() {
        use std::convert::TryInto;

        let timestamp = Timestamp::new_hms(65, 43, 21).unwrap();
        let dhm: Result<DhmTimestamp, ()> = timestamp.try_into();
        assert_eq!(Err(()), dhm);
    }
}
