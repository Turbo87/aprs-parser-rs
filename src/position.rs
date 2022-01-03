use std::str::FromStr;

use lonlat::{parse_latitude, parse_longitude};
use APRSError;
use Timestamp;

#[derive(PartialEq, Debug, Clone)]
pub struct APRSPosition {
    pub timestamp: Option<Timestamp>,
    pub latitude: f32,
    pub longitude: f32,
    pub comment: String,
}

impl FromStr for APRSPosition {
    type Err = APRSError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
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
            return Err(APRSError::UnsupportedPositionFormat(s.to_owned()));
        }

        if s.len() < 19 {
            return Err(APRSError::InvalidPosition(s.to_owned()));
        }

        // parse position
        let latitude = parse_latitude(&s[0..8])?;
        let longitude = parse_longitude(&s[9..18])?;

        let comment = &s[19..s.len()];

        Ok(APRSPosition {
            timestamp,
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
    fn parse() {
        let result = r"!4903.50N/07201.75W-".parse::<APRSPosition>().unwrap();
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(result.latitude, 49.05833);
        assert_relative_eq!(result.longitude, -72.02916);
        assert_eq!(result.comment, "");
    }

    #[test]
    fn parse_with_comment() {
        let result = r"!4903.50N/07201.75W-Hello/A=001000"
            .parse::<APRSPosition>()
            .unwrap();
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(result.latitude, 49.05833);
        assert_relative_eq!(result.longitude, -72.02916);
        assert_eq!(result.comment, "Hello/A=001000");
    }

    #[test]
    fn parse_with_timestamp() {
        let result = r"/074849h4821.61N\01224.49E^322/103/A=003054"
            .parse::<APRSPosition>()
            .unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert_relative_eq!(result.latitude, 48.360166);
        assert_relative_eq!(result.longitude, 12.408166);
        assert_eq!(result.comment, "322/103/A=003054");
    }
}
