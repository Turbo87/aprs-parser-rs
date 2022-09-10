use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use bytes::parse_bytes;
use AprsError;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Timestamp {
    /// Day of month, Hour and Minute in UTC
    DDHHMM(u8, u8, u8),
    /// Hour, Minute and Second in UTC
    HHMMSS(u8, u8, u8),
    /// Unsupported timestamp format
    Unsupported(Vec<u8>),
}

impl TryFrom<&[u8]> for Timestamp {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        if b.len() != 7 {
            return Err(AprsError::InvalidTimestamp(b.to_owned()));
        }

        let one = parse_bytes(&b[0..2]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;
        let two = parse_bytes(&b[2..4]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;
        let three =
            parse_bytes(&b[4..6]).ok_or_else(|| AprsError::InvalidTimestamp(b.to_owned()))?;

        Ok(match b[6] {
            b'z' => Timestamp::DDHHMM(one, two, three),
            b'h' => Timestamp::HHMMSS(one, two, three),
            b'/' => Timestamp::Unsupported(b.to_owned()),
            _ => return Err(AprsError::InvalidTimestamp(b.to_owned())),
        })
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::DDHHMM(d, h, m) => write!(f, "{:02}{:02}{:02}z", d, h, m),
            Self::HHMMSS(h, m, s) => write!(f, "{:02}{:02}{:02}h", h, m, s),
            Self::Unsupported(s) => write!(f, "{}", String::from_utf8_lossy(s)),
        }
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
    }

    #[test]
    fn parse_hhmmss() {
        assert_eq!(
            Timestamp::try_from(&b"123456h"[..]),
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
}
