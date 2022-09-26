use std::convert::TryFrom;
use std::io::Write;

use lonlat::{encode_latitude, encode_longitude, Latitude, Longitude};
use AprsCompressedCs;
use AprsCompressionType;
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
    pub extra: PositionExtra,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PositionExtra {
    Compressed(Option<(AprsCompressedCs, AprsCompressionType)>),
    Uncompressed,
}

impl TryFrom<&[u8]> for AprsPosition {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let first = *b.get(0).ok_or_else(|| AprsError::InvalidPosition(vec![]))?;
        let messaging_supported = first == b'=' || first == b'@';

        // parse timestamp if necessary
        let has_timestamp = first == b'@' || first == b'/';
        let timestamp = if has_timestamp {
            Some(Timestamp::try_from(
                b.get(1..8)
                    .ok_or_else(|| AprsError::InvalidPosition(b.to_vec()))?,
            )?)
        } else {
            None
        };

        // strip leading type symbol and potential timestamp
        let b = if has_timestamp { &b[8..] } else { &b[1..] };

        // check for compressed position format
        let is_uncompressed_position = (*b.get(0).unwrap_or(&0) as char).is_numeric();
        match is_uncompressed_position {
            true => Self::parse_uncompressed(b, timestamp, messaging_supported),
            false => Self::parse_compressed(b, timestamp, messaging_supported),
        }
    }
}

impl AprsPosition {
    fn parse_compressed(
        b: &[u8],
        timestamp: Option<Timestamp>,
        messaging_supported: bool,
    ) -> Result<Self, AprsError> {
        if b.len() < 13 {
            return Err(AprsError::InvalidPosition(b.to_owned()));
        }

        let symbol_table = b[0] as char;
        let comp_lat = &b[1..5];
        let comp_lon = &b[5..9];
        let symbol_code = b[9] as char;
        let course_speed = &b[10..12];
        let comp_type = b[12];

        let latitude = Latitude::from_compressed_ascii(comp_lat)?;
        let longitude = Longitude::from_compressed_ascii(comp_lon)?;

        // From the APRS spec - if the c value is a space,
        // the csT doesn't matter
        let cst = match course_speed[0] {
            b' ' => None,
            _ => {
                let t = comp_type
                    .checked_sub(33)
                    .ok_or_else(|| AprsError::InvalidPosition(b.to_owned()))?
                    .into();
                let cs = AprsCompressedCs::parse(course_speed[0], course_speed[1], t)?;
                Some((cs, t))
            }
        };

        let comment = b[13..].to_owned();

        Ok(Self {
            timestamp,
            messaging_supported,
            latitude,
            longitude,
            symbol_table,
            symbol_code,
            comment,
            extra: PositionExtra::Compressed(cst),
        })
    }

    fn parse_uncompressed(
        b: &[u8],
        timestamp: Option<Timestamp>,
        messaging_supported: bool,
    ) -> Result<Self, AprsError> {
        if b.len() < 19 {
            return Err(AprsError::InvalidPosition(b.to_owned()));
        }

        // parse position
        let latitude = Latitude::try_from(&b[0..8])?;
        let longitude = Longitude::try_from(&b[9..18])?;

        let symbol_table = b[8] as char;
        let symbol_code = b[18] as char;

        let comment = b[19..].to_owned();

        Ok(Self {
            timestamp,
            messaging_supported,
            latitude,
            longitude,
            symbol_table,
            symbol_code,
            comment,
            extra: PositionExtra::Uncompressed,
        })
    }

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

        match self.extra {
            PositionExtra::Uncompressed => self.encode_uncompressed(buf),
            PositionExtra::Compressed(data) => self.encode_compressed(buf, data),
        }
    }

    pub fn encode_uncompressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
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

    pub fn encode_compressed<W: Write>(
        &self,
        buf: &mut W,
        extra: Option<(AprsCompressedCs, AprsCompressionType)>,
    ) -> Result<(), EncodeError> {
        write!(buf, "{}", self.symbol_table)?;

        self.latitude.to_compressed_ascii(buf)?;
        self.longitude.to_compressed_ascii(buf)?;

        write!(buf, "{}", self.symbol_code)?;

        match extra {
            Some((cs, t)) => {
                cs.encode(buf, t)?;
            }
            None => write!(buf, " sT")?,
        };

        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compression_type::{GpsFix, NmeaSource, Origin};
    use AprsAltitude;
    use AprsCourseSpeed;
    use AprsRadioRange;

    #[test]
    fn parse_compressed_without_timestamp_or_messaging() {
        let result = AprsPosition::try_from(&b"!/ABCD#$%^- >C"[..]).unwrap();

        assert_eq!(result.timestamp, None);
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 25.97004667573229);
        assert_relative_eq!(*result.longitude, -171.95429033460567);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.extra, PositionExtra::Compressed(None));
    }

    #[test]
    fn parse_compressed_with_comment() {
        let result = AprsPosition::try_from(&b"!/ABCD#$%^-X>DHello/A=001000"[..]).unwrap();

        assert_eq!(result.timestamp, None);
        assert_relative_eq!(*result.latitude, 25.97004667573229);
        assert_relative_eq!(*result.longitude, -171.95429033460567);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, b"Hello/A=001000");
        assert_eq!(
            result.extra,
            PositionExtra::Compressed(Some((
                AprsCompressedCs::CourseSpeed(AprsCourseSpeed::new(220, 8.317274897290226,)),
                AprsCompressionType {
                    gps_fix: GpsFix::Current,
                    nmea_source: NmeaSource::Other,
                    origin: Origin::Tbd,
                }
            )))
        );
    }

    #[test]
    fn parse_compressed_with_timestamp_without_messaging() {
        let result =
            AprsPosition::try_from(&br"/074849h\ABCD#$%^^{?C322/103/A=003054"[..]).unwrap();

        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 25.97004667573229);
        assert_relative_eq!(*result.longitude, -171.95429033460567);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(
            result.extra,
            PositionExtra::Compressed(Some((
                AprsCompressedCs::RadioRange(AprsRadioRange::new(20.12531377814689)),
                AprsCompressionType {
                    gps_fix: GpsFix::Current,
                    nmea_source: NmeaSource::Other,
                    origin: Origin::Software,
                }
            )))
        );
    }

    #[test]
    fn parse_compressed_without_timestamp_with_messaging() {
        let result = AprsPosition::try_from(&b"=/ABCD#$%^-S]1"[..]).unwrap();

        assert_eq!(result.timestamp, None);
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 25.97004667573229);
        assert_relative_eq!(*result.longitude, -171.95429033460567);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(
            result.extra,
            PositionExtra::Compressed(Some((
                AprsCompressedCs::Altitude(AprsAltitude::new(10004.520050700292)),
                AprsCompressionType {
                    gps_fix: GpsFix::Old,
                    nmea_source: NmeaSource::Gga,
                    origin: Origin::Compressed,
                }
            )))
        );
    }

    #[test]
    fn parse_compressed_with_timestamp_and_messaging() {
        let result =
            AprsPosition::try_from(&br"@074849h\ABCD#$%^^ >C322/103/A=003054"[..]).unwrap();

        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 25.97004667573229);
        assert_relative_eq!(*result.longitude, -171.95429033460567);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.extra, PositionExtra::Compressed(None));
    }

    #[test]
    fn parse_without_timestamp_or_messaging() {
        let result = AprsPosition::try_from(&b"!4903.50N/07201.75W-"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 49.05833333333333);
        assert_relative_eq!(*result.longitude, -72.02916666666667);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.extra, PositionExtra::Uncompressed);
    }

    #[test]
    fn parse_with_comment() {
        let result = AprsPosition::try_from(&b"!4903.50N/07201.75W-Hello/A=001000"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(*result.latitude, 49.05833333333333);
        assert_relative_eq!(*result.longitude, -72.02916666666667);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, b"Hello/A=001000");
        assert_eq!(result.extra, PositionExtra::Uncompressed);
    }

    #[test]
    fn parse_with_timestamp_without_messaging() {
        let result =
            AprsPosition::try_from(&br"/074849h4821.61N\01224.49E^322/103/A=003054"[..]).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.latitude, 48.36016666666667);
        assert_relative_eq!(*result.longitude, 12.408166666666666);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.extra, PositionExtra::Uncompressed);
    }

    #[test]
    fn parse_without_timestamp_with_messaging() {
        let result = AprsPosition::try_from(&b"=4903.50N/07201.75W-"[..]).unwrap();
        assert_eq!(result.timestamp, None);
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 49.05833333333333);
        assert_relative_eq!(*result.longitude, -72.02916666666667);
        assert_eq!(result.symbol_table, '/');
        assert_eq!(result.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.extra, PositionExtra::Uncompressed);
    }

    #[test]
    fn parse_with_timestamp_and_messaging() {
        let result =
            AprsPosition::try_from(&br"@074849h4821.61N\01224.49E^322/103/A=003054"[..]).unwrap();
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.latitude, 48.36016666666667);
        assert_relative_eq!(*result.longitude, 12.408166666666666);
        assert_eq!(result.symbol_table, '\\');
        assert_eq!(result.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.extra, PositionExtra::Uncompressed);
    }

    #[test]
    fn parse_and_reencode_positions() {
        let positions = vec![
            &b"!/ABCD#$%^- sT"[..],
            &b"!/ABCD#$%^-A>CHello/A=001000"[..],
            &b"/074849h/ABCD#$%^-{>C322/103/A=001000"[..],
            &b"=/ABCD#$%^-2>1"[..],
            &b"@074849h/ABCD#$%^- sT"[..],
            &b"!4903.50N/07201.75W-"[..],
            &b"!4903.50N/07201.75W-Hello/A=001000"[..],
            &br"/074849h4821.61N\01224.49E^322/103/A=003054"[..],
            &b"=4903.50N/07201.75W-"[..],
            &br"@074849h4821.61N\01224.49E^322/103/A=003054"[..],
        ];

        for p in positions {
            let pos = AprsPosition::try_from(p).unwrap();
            let mut buf = vec![];
            pos.encode(&mut buf).unwrap();

            assert_eq!(
                p,
                buf,
                "Expected '{}', got '{}'",
                String::from_utf8_lossy(p),
                String::from_utf8_lossy(&buf)
            );
        }
    }
}
