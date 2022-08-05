use std::str::FromStr;

use lonlat::{Latitude, Longitude};
use AprsError;
use Timestamp;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsPosition {
    pub timestamp: Option<Timestamp>,
    pub messaging_supported: bool,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub comment: String,
}

impl FromStr for AprsPosition {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let messaging_supported = s.starts_with('=') || s.starts_with('@');

        // parse timestamp if necessary
        let has_timestamp = s.starts_with('@') || s.starts_with('/');
        let timestamp = if has_timestamp {
            Some(s[1..8].parse()?)
        } else {
            None
        };

        // strip leading type symbol and potential timestamp
        let s = if has_timestamp {
            &s[8..s.len()]
        } else {
            &s[1..s.len()]
        };

        // check for compressed position format
        let is_uncompressed_position = s.chars().take(1).all(|c| c.is_numeric());
        if !is_uncompressed_position {
            return Err(AprsError::UnsupportedPositionFormat(s.to_owned()));
        }

        if s.len() < 19 {
            return Err(AprsError::InvalidPosition(s.to_owned()));
        }

        // parse position
        let latitude = s[0..8].parse()?;
        let longitude = s[9..18].parse()?;

        let comment = &s[19..s.len()];

        Ok(AprsPosition {
            timestamp,
            messaging_supported,
            latitude,
            longitude,
            comment: comment.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_timestamp_or_messaging() {
        let result = r"!4903.50N/07201.75W-".parse::<AprsPosition>().unwrap();
        assert_eq!(result.timestamp, None);
        assert_eq!(result.messaging_supported, false);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.comment, "");
    }

    #[test]
    fn parse_with_comment() {
        let result = r"!4903.50N/07201.75W-Hello/A=001000"
            .parse::<AprsPosition>()
            .unwrap();
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.comment, "Hello/A=001000");
    }

    #[test]
    fn parse_with_timestamp_without_messaging() {
        let result = r"/074849h4821.61N\01224.49E^322/103/A=003054"
            .parse::<AprsPosition>()
            .unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert_eq!(result.messaging_supported, false);
        assert_relative_eq!(*result.latitude, 48.360166);
        assert_relative_eq!(*result.longitude, 12.408166);
        assert_eq!(result.comment, "322/103/A=003054");
    }

    #[test]
    fn parse_without_timestamp_with_messaging() {
        let result = r"=4903.50N/07201.75W-".parse::<AprsPosition>().unwrap();
        assert_eq!(result.timestamp, None);
        assert_eq!(result.messaging_supported, true);
        assert_relative_eq!(*result.latitude, 49.05833);
        assert_relative_eq!(*result.longitude, -72.02916);
        assert_eq!(result.comment, "");
    }

    #[test]
    fn parse_with_timestamp_and_messaging() {
        let result = r"@074849h4821.61N\01224.49E^322/103/A=003054"
            .parse::<AprsPosition>()
            .unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert_eq!(result.messaging_supported, true);
        assert_relative_eq!(*result.latitude, 48.360166);
        assert_relative_eq!(*result.longitude, 12.408166);
        assert_eq!(result.comment, "322/103/A=003054");
    }
}
