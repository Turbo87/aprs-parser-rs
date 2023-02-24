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

use DecodeError;
use DhmTimestamp;
use EncodeError;
use Timestamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsStatus {
    timestamp: Option<Timestamp>,
    comment: Vec<u8>,
}

impl AprsStatus {
    pub fn new(timestamp: Option<DhmTimestamp>, comment: Vec<u8>) -> Self {
        let timestamp = timestamp.map(|t| t.into());
        Self { timestamp, comment }
    }

    /// According to APRS spec, an AprsStatus should only allow the DDHHMM timestamp. (See page 80 of APRS101.PDF)
    /// In practice, many encoders don't adhere to this.
    /// Use this function to create an AprsStatus with any timestamp type
    pub fn new_noncompliant(timestamp: Option<Timestamp>, comment: Vec<u8>) -> Self {
        Self { timestamp, comment }
    }

    pub fn is_timestamp_compliant(&self) -> bool {
        self.timestamp
            .as_ref()
            .map(|t| matches!(t, Timestamp::DDHHMM(_, _, _)))
            .unwrap_or(true)
    }

    pub fn timestamp(&self) -> Option<&Timestamp> {
        self.timestamp.as_ref()
    }

    pub fn comment(&self) -> &[u8] {
        &self.comment
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, ">")?;

        if let Some(ts) = &self.timestamp {
            ts.encode(buf)?;
        }

        buf.write_all(&self.comment)?;

        Ok(())
    }
}

impl TryFrom<&[u8]> for AprsStatus {
    type Error = DecodeError;

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

    #[test]
    fn compliant_time_is_compliant() {
        let result = AprsStatus::try_from(r"312359z".as_bytes()).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::DDHHMM(31, 23, 59)));
        assert!(result.is_timestamp_compliant());
    }

    #[test]
    fn uncompliant_time_is_not_compliant() {
        let result = AprsStatus::try_from(r"235959hHi there!".as_bytes()).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(23, 59, 59)));
        assert!(!result.is_timestamp_compliant());
    }

    #[test]
    fn missing_time_is_compliant() {
        let result = AprsStatus::try_from(&b"Hi there!"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert!(result.is_timestamp_compliant());
    }
}
