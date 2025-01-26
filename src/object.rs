//! Any APRS station can manually report the position of an APRS entity (e.g.
//! another station or a weather phenomenon). This is intended for situations
//! where the entity is not capable of reporting its own position.
//! Object Reports specify an Object’s position, can have an optional timestamp,
//! and can include course/speed information or other Extended Data. Object
//! Reports are intended primarily for plotting the positions of moving objects
//! (e.g. spacecraft, storms, marathon runners without trackers).

use std::convert::TryFrom;
use std::io::Write;

use Callsign;
use DecodeError;

use EncodeError;
use Timestamp;

use AprsCst;
use Extension;
use Position;

#[derive(Clone, Debug, PartialEq)]
pub struct AprsObject {
    pub to: Callsign,
    pub name: Vec<u8>,
    pub live: bool,
    pub timestamp: Timestamp,
    pub position: Position,
    pub extension: Option<Extension>,
    pub comment: Vec<u8>,
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
        // [7/0]        (extended data)
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

        let mut name = b
            .get(..9)
            .ok_or_else(|| DecodeError::InvalidObjectName(b.to_vec()))?
            .to_vec();

        crate::utils::trim_spaces_end(&mut name);
        let live = match *b.get(9).ok_or(DecodeError::InvalidObject)? as char {
            '*' => Ok(true),
            ' ' => Ok(false),
            others => Err(DecodeError::InvalidObjectLiveness(others)),
        }?;
        let timestamp_bytes = b
            .get(10..17)
            .ok_or_else(|| DecodeError::InvalidTimestamp(b.to_vec()))?;
        let timestamp = Timestamp::try_from(timestamp_bytes)?;

        let (remaining_buffer, position) = Position::decode(
            b.get(17..)
                .ok_or_else(|| DecodeError::InvalidTimestamp(b.to_vec()))?,
        )?;

        // decide where the comment comes from
        let (extension, comment) = if matches!(position.cst, AprsCst::Uncompressed) {
            // opportunistically decode extensions if we can
            if let Some(comment_bytes) = remaining_buffer {
                if let Some(ext) = comment_bytes
                    .get(..7)
                    .and_then(|ext| Extension::decode(ext).ok())
                {
                    (
                        Some(ext),
                        comment_bytes.get(7..).unwrap_or_default().to_vec(),
                    )
                } else {
                    (None, comment_bytes.to_vec())
                }
            } else {
                (None, vec![])
            }
        } else {
            (None, remaining_buffer.unwrap_or_default().to_vec())
        };

        Ok(Self {
            to,
            name,
            live,
            timestamp,
            position,
            comment,
            extension,
        })
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
        buf.write_all(&self.name)?;
        for _ in self.name.len()..9 {
            buf.write_all(b" ")?; // pad out the remainder
        }

        write!(buf, "{}", if self.live { '*' } else { ' ' })?;
        self.timestamp.encode(buf)?;

        // if we have extensions, we have to do an uncompressed encoding to support it
        if let Some(ext) = &self.extension {
            self.position.encode_uncompressed(buf)?;
            ext.encode(buf)?;
        } else if matches!(self.position.cst, AprsCst::Uncompressed) {
            self.position.encode_uncompressed(buf)?; // just uncompressed, no extensions
        } else {
            self.position.encode_compressed(buf)?;
        }

        buf.write_all(&self.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{AprsData, AprsPacket, Directivity};

    use super::*;

    #[test]
    fn parse_uncompressed_live_object() {
        let packet = AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\\08635.47Wh146.940MHz T100 Huntsville Hamfest").unwrap();

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, b"HFEST-18H");
            assert!(o.live);
            assert_eq!(o.position.symbol_table, '\\');
            assert_eq!(o.position.symbol_code, 'h');
            assert_eq!(o.comment, "146.940MHz T100 Huntsville Hamfest".as_bytes());
            assert_eq!(o.position.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.position.latitude, 34.725833333333334);
            assert_relative_eq!(*o.position.longitude, -86.59116666666667);
        }
    }

    #[test]
    fn parse_uncompressed_dead_object_short_name() {
        let packet = AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:;HFEST     170403z3443.55N\\08635.47Wh146.940MHz T100 Huntsville Hamfest").unwrap();

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, b"HFEST");
            assert!(!o.live);
            assert_eq!(o.position.symbol_table, '\\');
        }
    }

    #[test]
    fn parse_uncompressed_live_object_short_name_extensions() {
        let packet = AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:;HFEST     170403z3443.55N\\08635.47WhPHG5132146.940MHz T100 Huntsville Hamfest").unwrap();

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, b"HFEST");
            assert!(!o.live);
            assert_eq!(o.position.symbol_table, '\\');
            assert!(matches!(
                o.extension,
                Some(Extension::PowerHeightGainDirectivity {
                    power_watts: 25,
                    antenna_height_feet: 20,
                    antenna_gain_db: 3,
                    antenna_directivity: Directivity::DirectionDegrees(90)
                })
            ));
        }
    }

    #[test]
    fn parse_uncompressed_object_no_comment() {
        let packet = AprsPacket::decode_textual(
            b"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\\08635.47Wh",
        )
        .unwrap();

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, b"HFEST-18H");
            assert!(o.live);
            assert_eq!(o.position.symbol_table, '\\');
            assert_eq!(o.position.symbol_code, 'h');
            assert_eq!(o.comment, []);
            assert_eq!(o.position.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.position.latitude, 34.725833333333334);
            assert_relative_eq!(*o.position.longitude, -86.59116666666667);
        }
    }

    #[test]
    fn parse_compressed_dead_object() {
        let packet = AprsPacket::decode_textual(
            b"N0CALL>APRS:;CAR       092345z/5L!!<*e7>7P[Moving to the north",
        )
        .unwrap();

        assert!(matches!(packet.data, AprsData::Object(_)));

        if let AprsData::Object(o) = packet.data {
            assert_eq!(o.name, b"CAR");
            assert!(!o.live);
            assert_relative_eq!(*o.position.latitude, 49.5);
            assert_relative_eq!(*o.position.longitude, -72.75000393777269);
            assert_eq!(o.position.symbol_code, '>');
            assert_eq!(o.comment, "Moving to the north".as_bytes());
        }
    }

    #[test]
    fn decode_recode_uncompressed() {
        let textual_repr = br"N8DEU-7>APZWX,WIDE2-2:;HFEST-18H*170403z3443.55N\08635.47Wh146.940MHz T100 Huntsville Hamfest";
        let packet = AprsPacket::decode_textual(textual_repr).unwrap();

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(buf, textual_repr);
    }

    #[test]
    fn decode_recode_compressed() {
        let textual_repr = b"N0CALL>APRS:;CAR       092345z/5L!!<*e7>7P[Moving to the north";
        let packet = AprsPacket::decode_textual(textual_repr).unwrap();

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(buf, textual_repr);
    }
}
