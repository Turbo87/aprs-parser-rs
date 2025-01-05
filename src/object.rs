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

use crate::{AprsCst, AprsPosition};

#[derive(Clone, Debug, PartialEq)]
pub struct AprsObject {
    pub to: Callsign,
    pub name: Vec<u8>,
    pub live: bool,
    pub timestamp: Timestamp,
    pub position: AprsPosition,
}

impl AprsObject {
    pub fn comment(&self) -> &[u8] {
        &self.position.comment
    }
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

        let mut name = b
            .get(..9)
            .ok_or_else(|| DecodeError::InvalidObjectName(b.to_vec()))?
            .to_vec();

        crate::utils::trim_spaces_end(&mut name);
        let live = match *b.get(9).unwrap() as char {
            '*' => Ok(true),
            ' ' => Ok(false),
            others => Err(DecodeError::InvalidObjectLiveness(others)),
        }?;

        let timestamp = Timestamp::try_from(b.get(10..17).unwrap())?;

        // ok, here we switch, it COULD be compressed, or it COULD be uncompressed
        // if the leading character is a numeric, it's an uncompressed
        // if it's alpha it's compressed
        let compressed = !(*b.get(17).unwrap_or(&0) as char).is_numeric();

        if compressed {
            let position = AprsPosition::parse_compressed(
                &b[17..],
                to.clone(),
                Some(timestamp.clone()),
                false,
            )?;

            Ok(Self {
                to,
                name,
                live,
                timestamp,
                position,
            })
        } else {
            let position = AprsPosition::parse_uncompressed(
                &b[17..],
                to.clone(),
                Some(timestamp.clone()),
                false,
            )?;

            Ok(Self {
                to,
                name,
                live,
                timestamp,
                position,
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
        buf.write_all(&self.name)?;
        for _ in self.name.len()..9 {
            buf.write_all(b" ")?; // pad out the remainder
        }

        write!(buf, "{}", if self.live { '*' } else { ' ' })?;
        self.timestamp.encode(buf)?;
        match self.position.cst {
            AprsCst::CompressedSome { cs, t } => {
                write!(buf, "{}", self.position.symbol_table)?;

                self.position.latitude.encode_compressed(buf)?;
                self.position.longitude.encode_compressed(buf)?;

                write!(buf, "{}", self.position.symbol_code)?;

                cs.encode(buf, t)?;
            }
            AprsCst::CompressedNone => {
                write!(buf, "{}", self.position.symbol_table)?;

                self.position.latitude.encode_compressed(buf)?;
                self.position.longitude.encode_compressed(buf)?;

                write!(buf, "{}", self.position.symbol_code)?;

                write!(buf, " sT")?; // no cst
            }
            AprsCst::Uncompressed => {
                self.position
                    .latitude
                    .encode_uncompressed(buf, self.position.precision)?;
                write!(buf, "{}", self.position.symbol_table)?;
                self.position.longitude.encode_uncompressed(buf)?;
                write!(buf, "{}", self.position.symbol_code)?;
            }
        }

        buf.write_all(&self.position.comment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{AprsData, AprsPacket};

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
            assert_eq!(
                o.position.comment,
                "146.940MHz T100 Huntsville Hamfest".as_bytes()
            );
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
            assert_eq!(o.comment(), []);
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
            assert_eq!(o.position.comment, "Moving to the north".as_bytes());
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
