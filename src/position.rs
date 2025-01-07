use std::convert::TryFrom;
use std::io::Write;

use Callsign;
use DecodeError;
use EncodeError;
use Timestamp;

use Position;

use AprsCst;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsPosition {
    pub to: Callsign,

    pub timestamp: Option<Timestamp>,
    pub messaging_supported: bool,

    pub position: Position,

    pub comment: Vec<u8>,
}

impl AprsPosition {
    pub fn decode(b: &[u8], to: Callsign) -> Result<Self, DecodeError> {
        let first = *b
            .first()
            .ok_or_else(|| DecodeError::InvalidPosition(vec![]))?;
        let messaging_supported = first == b'=' || first == b'@';

        // parse timestamp if necessary
        let has_timestamp = first == b'@' || first == b'/';
        let timestamp = if has_timestamp {
            Some(Timestamp::try_from(
                b.get(1..8)
                    .ok_or_else(|| DecodeError::InvalidPosition(b.to_vec()))?,
            )?)
        } else {
            None
        };

        // strip leading type symbol and potential timestamp
        let b = if has_timestamp { &b[8..] } else { &b[1..] };

        // decode the position and symbol data
        let position = Position::decode(b)?;
        // decide where the comment comes from
        let comment = if matches!(position.cst, AprsCst::Uncompressed) {
            b[19..].to_vec()
        } else {
            b[13..].to_vec()
        };

        Ok(Self {
            to,
            timestamp,
            messaging_supported,
            position,
            comment,
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

        // if we have a compressed cst, we must use a compressed position
        if matches!(self.position.cst, AprsCst::Uncompressed) {
            self.position.encode_uncompressed(buf)?;
        } else {
            self.position.encode_compressed(buf)?;
        }
        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compression_type::{GpsFix, NmeaSource, Origin};
    use AprsAltitude;
    use AprsCompressedCs;
    use AprsCompressionType;
    use AprsCourseSpeed;
    use AprsRadioRange;

    fn default_callsign() -> Callsign {
        Callsign::new_no_ssid("VE9")
    }

    #[test]
    fn parse_compressed_without_timestamp_or_messaging() {
        let result = AprsPosition::decode(&b"!/ABCD#$%^- >C"[..], default_callsign()).unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 25.97004667573229);
        assert_relative_eq!(*result.position.longitude, -171.95429033460567);
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.position.cst, AprsCst::CompressedNone);
    }

    #[test]
    fn parse_compressed_with_comment() {
        let result =
            AprsPosition::decode(&b"!/ABCD#$%^-X>DHello/A=001000"[..], default_callsign()).unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert_relative_eq!(*result.position.latitude, 25.97004667573229);
        assert_relative_eq!(*result.position.longitude, -171.95429033460567);
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, b"Hello/A=001000");
        assert_eq!(
            result.position.cst,
            AprsCst::CompressedSome {
                cs: AprsCompressedCs::CourseSpeed(AprsCourseSpeed::new(220, 8.317274897290226,)),
                t: AprsCompressionType {
                    gps_fix: GpsFix::Current,
                    nmea_source: NmeaSource::Other,
                    origin: Origin::Tbd,
                }
            }
        );
    }

    #[test]
    fn parse_compressed_with_timestamp_without_messaging() {
        let result = AprsPosition::decode(
            &br"/074849h\ABCD#$%^^{?C322/103/A=003054"[..],
            default_callsign(),
        )
        .unwrap();

        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 25.97004667573229);
        assert_relative_eq!(*result.position.longitude, -171.95429033460567);
        assert_eq!(result.position.symbol_table, '\\');
        assert_eq!(result.position.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(
            result.position.cst,
            AprsCst::CompressedSome {
                cs: AprsCompressedCs::RadioRange(AprsRadioRange::new(20.12531377814689)),
                t: AprsCompressionType {
                    gps_fix: GpsFix::Current,
                    nmea_source: NmeaSource::Other,
                    origin: Origin::Software,
                }
            }
        );
    }

    #[test]
    fn parse_compressed_without_timestamp_with_messaging() {
        let result = AprsPosition::decode(&b"=/ABCD#$%^-S]1"[..], default_callsign()).unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 25.97004667573229);
        assert_relative_eq!(*result.position.longitude, -171.95429033460567);
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(
            result.position.cst,
            AprsCst::CompressedSome {
                cs: AprsCompressedCs::Altitude(AprsAltitude::new(10004.520050700292)),
                t: AprsCompressionType {
                    gps_fix: GpsFix::Old,
                    nmea_source: NmeaSource::Gga,
                    origin: Origin::Compressed,
                }
            }
        );
    }

    #[test]
    fn parse_compressed_with_timestamp_and_messaging() {
        let result = AprsPosition::decode(
            &br"@074849h\ABCD#$%^^ >C322/103/A=003054"[..],
            default_callsign(),
        )
        .unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 25.97004667573229);
        assert_relative_eq!(*result.position.longitude, -171.95429033460567);
        assert_eq!(result.position.symbol_table, '\\');
        assert_eq!(result.position.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.position.cst, AprsCst::CompressedNone);
    }

    #[test]
    fn parse_without_timestamp_or_messaging() {
        let result =
            AprsPosition::decode(&b"!4903.50N/07201.75W-"[..], default_callsign()).unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 49.05833333333333);
        assert_relative_eq!(*result.position.longitude, -72.02916666666667);
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.position.cst, AprsCst::Uncompressed);
    }

    #[test]
    fn parse_with_comment() {
        let result = AprsPosition::decode(
            &b"!4903.5 N/07201.75W-Hello/A=001000"[..],
            default_callsign(),
        )
        .unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert_eq!(*result.position.latitude, 49.05833333333333);
        assert_eq!(*result.position.longitude, -72.02833333333334);
        assert_eq!(
            crate::components::lonlat::Precision::TenthMinute,
            result.position.precision
        );
        assert_eq!(
            49.0575..=49.05916666666666,
            result.position.latitude_bounding()
        );
        assert_eq!(
            -72.02916666666667..=-72.0275,
            result.position.longitude_bounding()
        );
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, b"Hello/A=001000");
        assert_eq!(result.position.cst, AprsCst::Uncompressed);
    }

    #[test]
    fn parse_with_timestamp_without_messaging() {
        let result = AprsPosition::decode(
            &br"/074849h4821.61N\01224.49E^322/103/A=003054"[..],
            default_callsign(),
        )
        .unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(!result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 48.36016666666667);
        assert_relative_eq!(*result.position.longitude, 12.408166666666666);
        assert_eq!(result.position.symbol_table, '\\');
        assert_eq!(result.position.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.position.cst, AprsCst::Uncompressed);
    }

    #[test]
    fn parse_without_timestamp_with_messaging() {
        let result =
            AprsPosition::decode(&b"=4903.50N/07201.75W-"[..], default_callsign()).unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, None);
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 49.05833333333333);
        assert_relative_eq!(*result.position.longitude, -72.02916666666667);
        assert_eq!(result.position.symbol_table, '/');
        assert_eq!(result.position.symbol_code, '-');
        assert_eq!(result.comment, []);
        assert_eq!(result.position.cst, AprsCst::Uncompressed);
    }

    #[test]
    fn parse_with_timestamp_and_messaging() {
        let result = AprsPosition::decode(
            &br"@074849h4821.61N\01224.49E^322/103/A=003054"[..],
            default_callsign(),
        )
        .unwrap();

        assert_eq!(result.to, default_callsign());
        assert_eq!(result.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
        assert!(result.messaging_supported);
        assert_relative_eq!(*result.position.latitude, 48.36016666666667);
        assert_relative_eq!(*result.position.longitude, 12.408166666666666);
        assert_eq!(result.position.symbol_table, '\\');
        assert_eq!(result.position.symbol_code, '^');
        assert_eq!(result.comment, b"322/103/A=003054");
        assert_eq!(result.position.cst, AprsCst::Uncompressed);
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
            &br"@074849h4821.  N\01224.00E^322/103/A=003054"[..],
        ];

        for p in positions {
            let pos = AprsPosition::decode(p, default_callsign()).unwrap();
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
