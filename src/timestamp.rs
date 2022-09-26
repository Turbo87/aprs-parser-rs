use bytes::parse_bytes;
use std::convert::TryFrom;
use std::io::Write;

use AprsError;
use EncodeError;

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
}
