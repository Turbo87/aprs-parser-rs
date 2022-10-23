//! A Status Report announces the station's current mission or any other single
//! line status to everyone. The report starts with the '>' APRS Data Type Identifier.
//! The report may optionally contain a timestamp.
//!
//! Examples:
//! - ">12.6V 0.2A 22degC"              (report without timestamp)
//! - ">120503hFatal error"             (report with timestamp in HMS format)
//! - ">281205zSystem will shutdown"    (report with timestamp in DHM format)

use std::convert::TryFrom;
use std::io::Write;

use AprsError;
use EncodeError;
use Timestamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsStatus {
    pub timestamp: Option<Timestamp>,
    pub comment: Vec<u8>,
}

impl TryFrom<&[u8]> for AprsStatus {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        // Interpret the first 7 bytes as a timestamp, if valid.
        // Otherwise the whole field is the comment.
        let timestamp = b.get(..7).and_then(|b| Timestamp::try_from(b).ok());
        let comment = if timestamp.is_some() { &b[7..] } else { b };

        Ok(AprsStatus {
            timestamp,
            comment: comment.to_owned(),
        })
    }
}

impl AprsStatus {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, ">")?;

        if let Some(ts) = &self.timestamp {
            ts.encode(buf)?;
        }

        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_timestamp_or_comment() {
        let result = AprsStatus::try_from(&b""[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert_eq!(result.comment, []);
    }

    #[test]
    fn parse_with_timestamp_without_comment() {
        let result = AprsStatus::try_from(r"312359z".as_bytes()).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::DDHHMM(31, 23, 59)));
        assert_eq!(result.comment, b"");
    }

    #[test]
    fn parse_without_timestamp_with_comment() {
        let result = AprsStatus::try_from(&b"Hi there!"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert_eq!(result.comment, b"Hi there!");
    }

    #[test]
    fn parse_with_timestamp_and_comment() {
        let result = AprsStatus::try_from(r"235959hHi there!".as_bytes()).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(23, 59, 59)));
        assert_eq!(result.comment, b"Hi there!");
    }
}
