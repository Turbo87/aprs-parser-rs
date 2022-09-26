use std::convert::TryFrom;
use std::io::Write;

use AprsError;
use AprsMessage;
use AprsPosition;
use Callsign;
use EncodeError;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsPacket {
    pub from: Callsign,
    pub to: Callsign,
    pub via: Vec<Callsign>,
    pub data: AprsData,
}

impl TryFrom<&[u8]> for AprsPacket {
    type Error = AprsError;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let header_delimiter = s
            .iter()
            .position(|x| *x == b':')
            .ok_or_else(|| AprsError::InvalidPacket(s.to_owned()))?;
        let (header, rest) = s.split_at(header_delimiter);
        let body = &rest[1..];

        let from_delimiter = header
            .iter()
            .position(|x| *x == b'>')
            .ok_or_else(|| AprsError::InvalidPacket(s.to_owned()))?;
        let (from, rest) = header.split_at(from_delimiter);
        let from = Callsign::try_from(from)?;

        let to_and_via = &rest[1..];
        let mut to_and_via = to_and_via.split(|x| *x == b',');

        let to = to_and_via
            .next()
            .ok_or_else(|| AprsError::InvalidPacket(s.to_owned()))?;
        let to = Callsign::try_from(to)?;

        let mut via = vec![];
        for v in to_and_via {
            via.push(Callsign::try_from(v)?);
        }

        let data = AprsData::try_from(body)?;

        Ok(AprsPacket {
            from,
            to,
            via,
            data,
        })
    }
}

impl AprsPacket {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, "{}>{}", self.from, self.to)?;
        for v in &self.via {
            write!(buf, ",{}", v)?;
        }
        write!(buf, ":")?;
        self.data.encode(buf)?;

        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AprsData {
    Position(AprsPosition),
    Message(AprsMessage),
    Unknown,
}

impl TryFrom<&[u8]> for AprsData {
    type Error = AprsError;

    fn try_from(s: &[u8]) -> Result<Self, AprsError> {
        Ok(match *s.first().unwrap_or(&0) {
            b':' => AprsData::Message(AprsMessage::try_from(&s[1..])?),
            b'!' | b'/' | b'=' | b'@' => AprsData::Position(AprsPosition::try_from(s)?),
            _ => AprsData::Unknown,
        })
    }
}

impl AprsData {
    fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        match self {
            Self::Position(p) => {
                p.encode(buf)?;
            }
            Self::Message(m) => {
                m.encode(buf)?;
            }
            Self::Unknown => return Err(EncodeError::InvalidData),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Timestamp;

    #[test]
    fn parse() {
        let result = AprsPacket::try_from(r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1".as_bytes()).unwrap();
        assert_eq!(result.from, Callsign::new("ICA3D17F2", None));
        assert_eq!(result.to, Callsign::new("APRS", None));
        assert_eq!(
            result.via,
            vec![Callsign::new("qAS", None), Callsign::new("dl4mea", None),]
        );

        match result.data {
            AprsData::Position(position) => {
                assert_eq!(position.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
                assert_relative_eq!(*position.latitude, 48.36016666666667);
                assert_relative_eq!(*position.longitude, 12.408166666666666);
                assert_eq!(
                    position.comment,
                    b"322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1"
                );
            }
            _ => panic!("Unexpected data type"),
        }
    }

    #[test]
    fn parse_message() {
        let result = AprsPacket::try_from(
            &b"ICA3D17F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has a : colon {3a2B975"[..],
        )
        .unwrap();
        assert_eq!(result.from, Callsign::new("ICA3D17F2", None));
        assert_eq!(result.to, Callsign::new("Aprs", None));
        assert_eq!(
            result.via,
            vec![Callsign::new("qAS", None), Callsign::new("dl4mea", None),]
        );

        match result.data {
            AprsData::Message(msg) => {
                assert_eq!(msg.addressee, b"DEST");
                assert_eq!(msg.text, b"Hello World! This msg has a : colon ");
                assert_eq!(msg.id, Some(b"3a2B975".to_vec()));
            }
            _ => panic!("Unexpected data type"),
        }
    }

    #[test]
    fn e2e_serialize_deserialize() {
        let valids = vec![
            r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ICA3D17F2>APRS,qAS,dl4mea:@074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ICA3D17F2>APRS,qAS,dl4mea:!4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ICA3D17F2>APRS,qAS,dl4mea:=4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ICA3D17F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has a : colon {32975",
            r"ICA3D17F2>Aprs,qAS,dl4mea::DESTINATI:Hello World! This msg has a : colon ",
        ];

        for v in valids {
            let mut buf = vec![];
            AprsPacket::try_from(v.as_bytes())
                .unwrap()
                .encode(&mut buf)
                .unwrap();
            assert_eq!(buf, v.as_bytes())
        }
    }

    #[test]
    fn e2e_invalid_string_msg() {
        let original = b"ICA3D17F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has raw bytes that are invalid utf8! \xc3\x28 {32975";

        let mut buf = vec![];
        let decoded = AprsPacket::try_from(&original[..]).unwrap();
        decoded.encode(&mut buf).unwrap();
        assert_eq!(buf, original);
    }
}
