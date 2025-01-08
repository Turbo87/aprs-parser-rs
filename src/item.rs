//! Any APRS station can manually report the position of an APRS entity (e.g.
//! another station or a weather phenomenon). This is intended for situations
//! where the entity is not capable of reporting its own position.
//! Item Reports specify an Item’s position, but cannot have a timestamp. While
//! Item reports may also include course/speed or other Extended Data, they are
//! really intended for inanimate things that are occasionally posted on a map
//! (e.g. marathon checkpoints or first-aid posts). Otherwise they are handled in
//! the same way as Item Reports.

use std::io::Write;

use Callsign;
use DecodeError;

use EncodeError;

use AprsCst;
use Extension;
use Position;

#[derive(Clone, Debug, PartialEq)]
pub struct AprsItem {
    pub to: Callsign,
    pub name: Vec<u8>,
    pub live: bool,
    pub position: Position,
    pub extension: Option<Extension>,
    pub comment: Vec<u8>,
}

impl AprsItem {
    pub fn decode(b: &[u8], to: Callsign) -> Result<Self, DecodeError> {
        // items are odd, name is 3..9 in length, any char except '!' or ' '
        // as that ends name parsing
        let mut name = b
            .get(..3)
            .ok_or_else(|| DecodeError::InvalidItemName(b.to_vec()))?
            .to_vec();

        let mut packet_offset_index = 3;

        for index in packet_offset_index..9.min(b.len()) {
            // dev note: we should use .is_some_and() here but MSRV is 1.70 for that feature
            if let Some(c) = b.get(index) {
                if [b' ', b'!'].contains(c) {
                    // we found the end of the name
                    packet_offset_index = index;
                    break;
                }
            }
        }
        // add these several bytes to name
        name.extend(
            b.get(3..packet_offset_index)
                .ok_or_else(|| DecodeError::InvalidItemName(b.to_vec()))?,
        );

        if name.contains(&b'!') || name.contains(&b' ') {
            // check that the name is not bad right away
            return Err(DecodeError::InvalidItemName(b.to_vec()));
        }

        let live = match *b.get(packet_offset_index).unwrap() as char {
            '!' => Ok(true),
            ' ' => Ok(false),
            others => Err(DecodeError::InvalidItemLiveness(others)),
        }?;

        let position = Position::decode(
            b.get(packet_offset_index + 1..)
                .ok_or_else(|| DecodeError::InvalidTimestamp(b.to_vec()))?,
        )?;

        // decide where the comment comes from
        let (extension, comment) = if matches!(position.cst, AprsCst::Uncompressed) {
            // opportunistically decode extensions if we can

            if let Some(ext) = b
                .get(packet_offset_index + 20..packet_offset_index + 27)
                .and_then(|ext| Extension::decode(ext).ok())
            {
                (Some(ext), b[packet_offset_index + 27..].to_vec())
            } else {
                (None, b[packet_offset_index + 20..].to_vec())
            }
        } else {
            (None, b[packet_offset_index + 14..].to_vec())
        };

        Ok(Self {
            to,
            name,
            live,
            position,
            comment,
            extension,
        })
    }

    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let mut name = self.name.clone();
        name.truncate(9);
        write!(buf, ")")?;
        buf.write_all(&self.name)?;

        write!(buf, "{}", if self.live { '!' } else { ' ' })?;

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
        let packet =
            AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:)AIDV#2!4903.50N/07201.75WA")
                .unwrap();

        assert!(matches!(packet.data, AprsData::Item(_)));

        if let AprsData::Item(o) = packet.data {
            assert_eq!(o.name, b"AIDV#2");
            assert!(o.live);
            assert_eq!(o.position.symbol_table, '/');
            assert_eq!(o.position.symbol_code, 'A');
            assert_eq!(o.comment, vec![]);
            assert_eq!(o.position.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.position.latitude, 49.05833333333333);
            assert_relative_eq!(*o.position.longitude, -72.02916666666667);
        }
    }

    #[test]
    fn parse_uncompressed_dead_object_short_name() {
        let packet =
            AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:)AID 4903.50N/07201.75WA").unwrap();

        assert!(matches!(packet.data, AprsData::Item(_)));

        if let AprsData::Item(o) = packet.data {
            assert_eq!(o.name, b"AID");
            assert!(!o.live);
            assert_eq!(o.position.symbol_table, '/');
            assert_eq!(o.position.symbol_code, 'A');
            assert_eq!(o.comment, vec![]);
            assert_eq!(o.position.cst, AprsCst::Uncompressed);
            assert_relative_eq!(*o.position.latitude, 49.05833333333333);
            assert_relative_eq!(*o.position.longitude, -72.02916666666667);
        }
    }

    #[test]
    fn parse_uncompressed_dead_object_short_name_extensions() {
        let packet =
            AprsPacket::decode_textual(b"N8DEU-7>APZWX,WIDE2-2:)AID 4903.50N/07201.75WAPHG5132")
                .unwrap();

        assert!(matches!(packet.data, AprsData::Item(_)));

        if let AprsData::Item(o) = packet.data {
            assert_eq!(o.name, b"AID");
            assert!(!o.live);
            assert_eq!(o.position.symbol_table, '/');
            assert_eq!(o.position.symbol_code, 'A');
            assert!(matches!(
                o.extension,
                Some(Extension::PowerHeightGainDirectivity {
                    power_watts: 25,
                    antenna_height_feet: 20,
                    antenna_gain_db: 3,
                    antenna_directivity: Directivity::DirectionDegrees(90)
                })
            ));
            assert!(o.comment.is_empty());
        }
    }

    #[test]
    fn parse_compressed_live_object() {
        let packet = AprsPacket::decode_textual(br"N0CALL>APRS:)MOBIL!\5L!!<*e79 sT").unwrap();

        assert!(matches!(packet.data, AprsData::Item(_)));

        if let AprsData::Item(o) = packet.data {
            assert_eq!(o.name, b"MOBIL");
            assert!(o.live);
            assert_relative_eq!(*o.position.latitude, 49.5);
            assert_relative_eq!(*o.position.longitude, -72.75000393777269);
            assert_eq!(o.position.symbol_code, '9');
            assert_eq!(o.position.symbol_table, '\\');
            assert!(matches!(o.position.cst, AprsCst::CompressedNone))
        }
    }

    #[test]
    fn decode_recode_uncompressed() {
        let textual_repr = b"N8DEU-7>APZWX,WIDE2-2:)AID 4903.50N/07201.75WAPHG5132";
        let packet = AprsPacket::decode_textual(textual_repr).unwrap();

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(buf, textual_repr);
    }

    #[test]
    fn decode_recode_compressed() {
        let textual_repr = br"N0CALL>APRS:)MOBIL!\5L!!<*e79 sT";
        let packet = AprsPacket::decode_textual(textual_repr).unwrap();

        let mut buf = Vec::new();

        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(buf, textual_repr);
    }
}
