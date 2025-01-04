//! Any APRS station can manually report the position of an APRS entity (e.g.
//! another station or a weather phenomenon). This is intended for situations
//! where the entity is not capable of reporting its own position.
//! Object Reports specify an Object’s position, can have an optional timestamp,
//! and can include course/speed information or other Extended Data. Object
//! Reports are intended primarily for plotting the positions of moving objects
//! (e.g. spacecraft, storms, marathon runners without trackers).
//!
//! Examples:
//!

use std::convert::TryFrom;
use std::io::Write;

use Callsign;
use DecodeError;

use EncodeError;
use Timestamp;

use crate::{AprsCompressedCs, AprsCst, Latitude, Longitude, Precision};

#[derive(Clone, Debug, PartialEq)]
pub struct AprsObject {
    pub to: Callsign,
    pub name: String,
    pub live: bool,
    pub timestamp: Timestamp,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub precision: Precision,
    pub symbol_table: char,
    pub symbol_code: char,
    pub comment: Vec<u8>,
    pub cst: AprsCst,
}

impl AprsObject {
    pub fn decode(b: &[u8], to: Callsign) -> Result<Self, DecodeError> {
        // format for uncompressed is
        // N Bytes      Description/Value
        // -----------------------------
        // [9]          (object name, space-padded)
        // [1]          (object live =  '*' dead = '_')
        // [7]          (timestamp)
        // [8]          (latitude)
        // [1]          (sym table id)
        // [9]          (longitude)
        // [1]          (sym code)
        // [7/0]        (extended data [NOT PARSING])
        // [0-36/0-43]  (comment)
        //

        // format for compressed is
        // N Bytes      Description/Value
        // -----------------------------
        // [9]          (object name, space-padded)
        // [1]          (object live =  '*' dead = '_')
        // [7]          (timestamp)
        // [13]         (compressed position data)
        // [43]         (comment)

        // packet must have at least name + liveness + timestamp --> (9+1+7) so check that here

        if b.len() < 17 {
            return Err(DecodeError::InvalidObjectFormat(
                b.to_vec(),
                format!(
                    "packet length must be at least 17 bytes long, got {}",
                    b.len()
                ),
            ));
        }

        let name = b
            .get(..9)
            .map(|by| String::from_utf8_lossy(by).trim().to_string())
            .unwrap();
        let live = match *b.get(9).unwrap() as char {
            '*' => Ok(true),
            ' ' => Ok(false),
            others => Err(DecodeError::InvalidObjectFormat(
                vec![*b.get(9).unwrap()],
                format!("expected either '*' or '(space)' for object liveness, got '{others}'"),
            )),
        }?;

        let timestamp = Timestamp::try_from(b.get(10..17).unwrap())?;

        // ok, here we switch, it COULD be compressed, or it COULD be uncompressed
        // if the leading character is a numeric, it's an uncompressed
        // if it's alpha it's compressed
        let compressed = !(*b.get(17).unwrap_or(&0) as char).is_numeric();

        if compressed {
            let symbol_table = b[17] as char;
            let comp_lat = &b[18..22];
            let comp_lon = &b[22..26];
            let symbol_code = b[26] as char;
            let course_speed = &b[27..29];
            let comp_type = b[29];

            let latitude = Latitude::parse_compressed(comp_lat)?;
            let longitude = Longitude::parse_compressed(comp_lon)?;

            // From the APRS spec - if the c value is a space,
            // the csT doesn't matter
            let cst = match course_speed[0] {
                b' ' => AprsCst::CompressedNone,
                _ => {
                    let t = comp_type
                        .checked_sub(33)
                        .ok_or_else(|| DecodeError::InvalidPosition(b.to_owned()))?
                        .into();
                    let cs = AprsCompressedCs::parse(course_speed[0], course_speed[1], t)?;
                    AprsCst::CompressedSome { cs, t }
                }
            };

            let comment = b[30..].to_owned();

            Ok(Self {
                to,
                name,
                live,
                timestamp,
                latitude,
                longitude,
                precision: Precision::default(),
                symbol_table,
                symbol_code,
                comment,
                cst,
            })
        } else {
            let (latitude, precision) = Latitude::parse_uncompressed(&b[17..25])?;
            let symbol_table = b[25] as char;
            let longitude = Longitude::parse_uncompressed(&b[26..35], precision)?;

            let symbol_code = b[35] as char;

            let comment = b[36..].to_owned();

            Ok(Self {
                to,
                name,
                live,
                timestamp,
                latitude,
                longitude,
                precision,
                symbol_table,
                symbol_code,
                comment,
                cst: AprsCst::Uncompressed,
            })
        }
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        // format for uncompressed is
        // N Bytes      Description/Value
        // -----------------------------
        // [9]          (object name, space-padded)
        // [1]          (object live =  '*' dead = '_')
        // [7]          (timestamp)
        // [8]          (latitude)
        // [1]          (sym table id)
        // [9]          (longitude)
        // [1]          (sym code)
        // [7/0]        (extended data [NOT PARSING])
        // [0-36/0-43]  (comment)
        //

        // format for compressed is
        // N Bytes      Description/Value
        // -----------------------------
        // [9]          (object name, space-padded)
        // [1]          (object live =  '*' dead = '_')
        // [7]          (timestamp)
        // [13]         (compressed position data)
        // [43]         (comment)

        let mut name = self.name.clone();
        name.truncate(9);
        write!(buf, ";")?;
        write!(buf, "{name:9}")?;
        write!(buf, "{}", if self.live { '*' } else { ' ' })?;
        self.timestamp.encode(buf)?;
        match self.cst {
            AprsCst::CompressedSome { cs, t } => {
                write!(buf, "{}", self.symbol_table)?;

                self.latitude.encode_compressed(buf)?;
                self.longitude.encode_compressed(buf)?;

                write!(buf, "{}", self.symbol_code)?;

                cs.encode(buf, t)?;
            }
            AprsCst::CompressedNone => {
                write!(buf, "{}", self.symbol_table)?;

                self.latitude.encode_compressed(buf)?;
                self.longitude.encode_compressed(buf)?;

                write!(buf, "{}", self.symbol_code)?;

                write!(buf, " sT")?; // no cst
            }
            AprsCst::Uncompressed => {
                self.latitude.encode_uncompressed(buf, self.precision)?;
                write!(buf, "{}", self.symbol_table)?;
                self.longitude.encode_uncompressed(buf)?;
                write!(buf, "{}", self.symbol_code)?;
            }
        }

        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{AprsData, AprsPacket};

    use super::*;

    #[test]
    fn parse_uncompressed_live_object() -> Result<(), Box<dyn std::error::Error>> {
        let packet = AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\\08635.47Wh146.940MHz T100 Huntsville Hamfest")?;

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, "HFEST-18H");
            assert!(o.live);
            assert_eq!(o.symbol_table, '\\');
            assert_eq!(o.symbol_code, 'h');
            assert_eq!(o.comment, "146.940MHz T100 Huntsville Hamfest".as_bytes());
            assert_eq!(o.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.latitude, 34.725833333333334);
            assert_relative_eq!(*o.longitude, -86.59116666666667);
        }

        Ok(())
    }

    #[test]
    fn parse_uncompressed_dead_object_short_name() -> Result<(), Box<dyn std::error::Error>> {
        let packet = AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:;HFEST     170403z3443.55N\\08635.47Wh146.940MHz T100 Huntsville Hamfest")?;

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, "HFEST");
            assert!(!o.live);
            assert_eq!(o.symbol_table, '\\');
        }

        Ok(())
    }

    #[test]
    fn parse_uncompressed_object_no_comment() -> Result<(), Box<dyn std::error::Error>> {
        let packet = AprsPacket::decode_textual(
            b"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\\08635.47Wh",
        )?;

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, "HFEST-18H");
            assert!(o.live);
            assert_eq!(o.symbol_table, '\\');
            assert_eq!(o.symbol_code, 'h');
            assert_eq!(o.comment, []);
            assert_eq!(o.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.latitude, 34.725833333333334);
            assert_relative_eq!(*o.longitude, -86.59116666666667);
        }

        Ok(())
    }

    #[test]
    fn parse_compressed_dead_object() -> Result<(), Box<dyn std::error::Error>> {
        let packet = AprsPacket::decode_textual(
            b"N0CALL>APRS:;CAR       092345z/5L!!<*e7>7P[Moving to the north",
        )?;

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, "CAR");
            assert!(!o.live);
            assert_relative_eq!(*o.latitude, 49.5);
            assert_relative_eq!(*o.longitude, -72.75000393777269);
            assert_eq!(o.symbol_code, '>');
            assert_eq!(o.comment, "Moving to the north".as_bytes());
        }

        Ok(())
    }

    #[test]
    fn decode_recode_uncompressed() -> Result<(), Box<dyn std::error::Error>> {
        let textual_repr = br"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\08635.47Wh146.940MHz T100 Huntsville Hamfest";
        let packet = AprsPacket::decode_textual(textual_repr)?;

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf)?;
        assert_eq!(buf, textual_repr);

        Ok(())
    }

    #[test]
    fn decode_recode_compressed() -> Result<(), Box<dyn std::error::Error>> {
        let textual_repr = b"N0CALL>APRS:;CAR       092345z/5L!!<*e7>7P[Moving to the north";
        let packet = AprsPacket::decode_textual(textual_repr)?;

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf)?;
        assert_eq!(buf, textual_repr);

        Ok(())
    }
}
