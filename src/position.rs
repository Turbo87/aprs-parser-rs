use std::convert::TryFrom;
use std::io::Write;

use lonlat::{encode_latitude, encode_longitude, Latitude, Longitude};
use AprsError;
use EncodeError;
use Timestamp;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsPosition {
    pub timestamp: Option<Timestamp>,
    pub messaging_supported: bool,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub symbol_table: char,
    pub symbol_code: char,
    pub comment: Vec<u8>,
}

impl TryFrom<&[u8]> for AprsPosition {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let first = *b.get(0).ok_or_else(|| AprsError::InvalidPosition(vec![]))?;
        let messaging_supported = first == b'=' || first == b'@';

        // parse timestamp if necessary
        let has_timestamp = first == b'@' || first == b'/';
        let timestamp = if has_timestamp {
            Some(Timestamp::try_from(&b[1..8])?)
        } else {
            None
        };

        // strip leading type symbol and potential timestamp
        let b = if has_timestamp { &b[8..] } else { &b[1..] };

        // check for compressed position format
        let is_uncompressed_position = (*b.get(0).unwrap_or(&0) as char).is_numeric();
        if !is_uncompressed_position {
            return Err(AprsError::UnsupportedPositionFormat(b.to_owned()));
        }

        if b.len() < 19 {
            return Err(AprsError::InvalidPosition(b.to_owned()));
        }

        // parse position
        let latitude = Latitude::try_from(&b[0..8])?;
        let longitude = Longitude::try_from(&b[9..18])?;

        let symbol_table = b[8] as char;
        let symbol_code = b[18] as char;

        let comment = &b[19..];

        Ok(AprsPosition {
            timestamp,
            messaging_supported,
            latitude,
            longitude,
            symbol_table,
            symbol_code,
            comment: comment.to_owned(),
        })
    }
}

impl AprsPosition {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let sym = match (self.timestamp.is_some(), self.messaging_supported) {
            (true, true) => '@',
            (true, false) => '/',
            (false, true) => '=',
            (false, false) => '!',
        };

        write!(buf, "{}", sym)?;

        if let Some(ts) = &self.timestamp {
            ts.encode(buf)?;
        }

        write!(
            buf,
            "{}{}{}{}",
            encode_latitude(self.latitude)?,
            self.symbol_table,
            encode_longitude(self.longitude)?,
            self.symbol_code,
        )?;
        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_timestamp_or_messaging() {
        let result = AprsPosition::try_from(&b"!4903.50N/07201.75W-"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
    }

    #[test]
    fn parse_with_comment() {
        let result = AprsPosition::try_from(&b"!4903.50N/07201.75W-Hello/A=001000"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, b"Hello/A=001000");
    }

    #[test]
    fn parse_with_timestamp_without_messaging() {
        let result =
            AprsPosition::try_from(r"/074849h4821.61N\01224.49E^322/103/A=003054".as_bytes())
                .unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 48.360166);
        assert_relative_eq!(*result.longitude, 12.408166);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
    }

    #[test]
    fn parse_without_timestamp_with_messaging() {
        let result = AprsPosition::try_from(&b"=4903.50N/07201.75W-"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
    }

    #[test]
    fn parse_with_timestamp_and_messaging() {
        let result =
            AprsPosition::try_from(r"@074849h4821.61N\01224.49E^322/103/A=003054".as_bytes())
                .unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 48.360166);
        assert_relative_eq!(*result.longitude, 12.408166);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
    }
}
