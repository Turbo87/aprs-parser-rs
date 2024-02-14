use std::borrow::Cow;
use std::io::Write;

use callsign::CallsignField;
use AprsMessage;
use AprsMicE;
use AprsPosition;
use AprsStatus;
use Callsign;
use DecodeError;
use EncodeError;
use Via;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsPacket {
    pub from: Callsign,
    pub via: Vec<Via>,
    pub data: AprsData,
}

impl AprsPacket {
    pub fn decode_textual(s: &[u8]) -> Result<Self, DecodeError> {
        let header_delimiter = s
            .iter()
            .position(|x| *x == b':')
            .ok_or_else(|| DecodeError::InvalidPacket(s.to_owned()))?;
        let (header, rest) = s.split_at(header_delimiter);
        let body = &rest[1..];

        let from_delimiter = header
            .iter()
            .position(|x| *x == b'>')
            .ok_or_else(|| DecodeError::InvalidPacket(s.to_owned()))?;
        let (from, rest) = header.split_at(from_delimiter);
        let (from, _) = Callsign::decode_textual(from)
            .ok_or_else(|| DecodeError::InvalidCallsign(from.to_owned()))?;

        let to_and_via = &rest[1..];
        let mut to_and_via = to_and_via.split(|x| *x == b',');

        let to = to_and_via
            .next()
            .ok_or_else(|| DecodeError::InvalidPacket(s.to_owned()))?;
        let (to, _) = Callsign::decode_textual(to)
            .ok_or_else(|| DecodeError::InvalidCallsign(to.to_owned()))?;

        let mut via = vec![];
        for v in to_and_via {
            via.push(Via::decode_textual(v).ok_or_else(|| DecodeError::InvalidVia(v.to_owned()))?);
        }

        // if our Via path looks like A,B,C*,D,E
        // this really means A*,B*,C*,D,E
        // so we need to propagate the `heard` flag backwards
        let mut heard = false;
        for v in via.iter_mut().rev() {
            if let Some((_, c_heard)) = v.callsign_mut() {
                if !heard {
                    heard = *c_heard;
                }
                *c_heard = heard;
            }
        }

        let data = AprsData::decode(body, to)?;

        Ok(AprsPacket { from, via, data })
    }

    pub fn to(&self) -> Option<&Callsign> {
        self.data.to()
    }

    /// Used for encoding a packet into ASCII for transmission on the internet (APRS-IS)
    pub fn encode_textual<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        // logic to clear extraneous asterisks
        let mut via = self.via.clone();
        let mut heard = false;
        for v in via.iter_mut().rev() {
            if let Some((_, c_heard)) = v.callsign_mut() {
                if !heard {
                    heard = *c_heard;
                } else {
                    *c_heard = false;
                }
            }
        }

        self.from.encode_textual(false, buf)?;
        write!(buf, ">")?;
        self.data.dest_field().encode_textual(false, buf)?;
        for v in &via {
            write!(buf, ",")?;
            v.encode_textual(buf)?;
        }
        write!(buf, ":")?;
        self.data.encode(buf)?;

        Ok(())
    }

    /// Used for decoding a packet received over the air (via KISS or otherwise)
    pub fn decode_ax25(data: &[u8]) -> Result<Self, DecodeError> {
        let dest_bytes = data
            .get(0..7)
            .ok_or_else(|| DecodeError::InvalidPacket(data.to_owned()))?;
        let (to, _, has_more) = Callsign::decode_ax25(dest_bytes)
            .ok_or_else(|| DecodeError::InvalidCallsign(dest_bytes.to_owned()))?;

        if !has_more {
            return Err(DecodeError::InvalidPacket(data.to_owned()));
        }

        let src_bytes = data
            .get(7..14)
            .ok_or_else(|| DecodeError::InvalidPacket(data.to_owned()))?;
        let (from, _, mut has_more) = Callsign::decode_ax25(src_bytes)
            .ok_or_else(|| DecodeError::InvalidCallsign(src_bytes.to_owned()))?;

        let mut i = 14;
        let mut via = vec![];
        while has_more {
            let v_bytes = data
                .get(i..(i + 7))
                .ok_or_else(|| DecodeError::InvalidPacket(data.to_owned()))?;

            // vias received over AX.25 are going to be callsigns only
            // no Q-constructs
            let (v, heard, more) = Callsign::decode_ax25(v_bytes)
                .ok_or_else(|| DecodeError::InvalidCallsign(v_bytes.to_owned()))?;

            via.push(Via::Callsign(v, heard));
            has_more = more;
            i += 7;
        }

        // verify control field and protocol id
        if data.get(i..(i + 2)) != Some(&[0x03, 0xf0]) {
            return Err(DecodeError::InvalidPacket(data.to_owned()));
        }
        i += 2;

        // remainder is the information field
        let data = AprsData::decode(data.get(i..).unwrap_or(&[]), to)?;

        Ok(Self { data, from, via })
    }

    /// Used for encoding a packet for transmission on the air (via KISS or otherwise)
    pub fn encode_ax25<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        // Destination address
        self.data
            .dest_field()
            .encode_ax25(buf, CallsignField::Destination, true)?;

        let via_calls: Vec<_> = self.via.iter().filter_map(|v| v.callsign()).collect();

        // Source address
        let has_more = !via_calls.is_empty();
        self.from
            .encode_ax25(buf, CallsignField::Source, has_more)?;

        // Digipeater addresses
        if let Some(((last_v, last_heard), vs)) = via_calls.split_last() {
            for (v, heard) in vs {
                v.encode_ax25(buf, CallsignField::Via(*heard), true)?;
            }

            last_v.encode_ax25(buf, CallsignField::Via(*last_heard), false)?;
        }

        // Control field - hardcoded to UI
        // Protocol ID - hardcoded to no layer 3
        buf.write_all(&[0x03, 0xf0])?;

        // Information field
        self.data.encode(buf)?;

        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AprsData {
    Position(AprsPosition),
    Message(AprsMessage),
    Status(AprsStatus),
    MicE(AprsMicE),
    Unknown(Callsign),
}

impl AprsData {
    pub fn to(&self) -> Option<&Callsign> {
        match self {
            AprsData::Position(p) => Some(&p.to),
            AprsData::Message(m) => Some(&m.to),
            AprsData::Status(s) => Some(&s.to),
            AprsData::MicE(_) => None,
            AprsData::Unknown(to) => Some(to),
        }
    }

    fn dest_field(&self) -> Cow<Callsign> {
        match self {
            AprsData::Position(p) => Cow::Borrowed(&p.to),
            AprsData::Message(m) => Cow::Borrowed(&m.to),
            AprsData::Status(s) => Cow::Borrowed(&s.to),
            AprsData::MicE(m) => Cow::Owned(m.encode_destination()),
            AprsData::Unknown(to) => Cow::Borrowed(to),
        }
    }

    fn decode(s: &[u8], to: Callsign) -> Result<Self, DecodeError> {
        Ok(match *s.first().unwrap_or(&0) {
            b':' => AprsData::Message(AprsMessage::decode(&s[1..], to)?),
            b'!' | b'/' | b'=' | b'@' => AprsData::Position(AprsPosition::decode(s, to)?),
            b'>' => AprsData::Status(AprsStatus::decode(&s[1..], to)?),
            0x1c | b'`' => AprsData::MicE(AprsMicE::decode(&s[1..], to, true)?),
            0x1d | b'\'' => AprsData::MicE(AprsMicE::decode(&s[1..], to, false)?),
            _ => AprsData::Unknown(to),
        })
    }

    fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        match self {
            Self::Position(p) => {
                p.encode(buf)?;
            }
            Self::Message(m) => {
                m.encode(buf)?;
            }
            Self::Status(st) => {
                st.encode(buf)?;
            }
            Self::MicE(m) => {
                m.encode(buf)?;
            }
            Self::Unknown(_) => return Err(EncodeError::InvalidData),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mic_e::{Course, Message, Speed};
    use AprsCst;
    use Latitude;
    use Longitude;
    use Precision;
    use QConstruct;
    use Timestamp;

    #[test]
    fn parse() {
        let result = AprsPacket::decode_textual(r"ID17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1".as_bytes()).unwrap();
        assert_eq!(result.from, Callsign::new_no_ssid("ID17F2"));
        assert_eq!(result.to(), Some(&Callsign::new_no_ssid("APRS")));
        assert_eq!(
            result.via,
            vec![
                Via::QConstruct(QConstruct::AS),
                Via::Callsign(Callsign::new_no_ssid("dl4mea"), false),
            ]
        );

        match result.data {
            AprsData::Position(position) => {
                assert_eq!(position.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
                assert_eq!(position.latitude.value(), 48.36016666666667);
                assert_eq!(position.longitude.value(), 12.408166666666666);
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
        let result = AprsPacket::decode_textual(
            &b"IC17F2>Aprs,qAX,dl4mea::DEST     :Hello World! This msg has a : colon {3a2B975"[..],
        )
        .unwrap();
        assert_eq!(result.from, Callsign::new_no_ssid("IC17F2"));
        assert_eq!(result.to(), Some(&Callsign::new_no_ssid("Aprs")));
        assert_eq!(
            result.via,
            vec![
                Via::QConstruct(QConstruct::AX),
                Via::Callsign(Callsign::new_no_ssid("dl4mea"), false),
            ]
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
    fn parse_status() {
        let result =
            AprsPacket::decode_textual(&b"3D17F2>APRS,qAU,dl4mea:>312359zStatus seems okay!"[..])
                .unwrap();
        assert_eq!(result.from, Callsign::new_no_ssid("3D17F2"));
        assert_eq!(result.to(), Some(&Callsign::new_no_ssid("APRS")));
        assert_eq!(
            result.via,
            vec![
                Via::QConstruct(QConstruct::AU),
                Via::Callsign(Callsign::new_no_ssid("dl4mea"), false),
            ]
        );

        match result.data {
            AprsData::Status(msg) => {
                assert_eq!(msg.timestamp(), Some(&Timestamp::DDHHMM(31, 23, 59)));
                assert_eq!(msg.comment(), b"Status seems okay!");
            }
            _ => panic!("Unexpected data type"),
        }
    }

    #[test]
    fn encode_ax25_basic() {
        let encoded_ax25 = vec![
            0x82, 0xa0, 0x9c, 0xaa, 0x62, 0x72, 0xe0, 0xac, 0x8a, 0x72, 0x84, 0x86, 0xa2, 0x60,
            0xac, 0x8a, 0x72, 0x88, 0x8e, 0xa0, 0xe0, 0xac, 0x8a, 0x72, 0x8e, 0x8c, 0x92, 0xe4,
            0xac, 0x8a, 0x72, 0x8c, 0xa0, 0x8e, 0xe0, 0xae, 0x92, 0x88, 0x8a, 0x66, 0x40, 0x61,
            0x03, 0xf0, 0x21, 0x34, 0x36, 0x32, 0x37, 0x2e, 0x32, 0x30, 0x4e, 0x53, 0x30, 0x36,
            0x36, 0x33, 0x31, 0x2e, 0x31, 0x39, 0x57, 0x23, 0x50, 0x48, 0x47, 0x35, 0x34, 0x36,
            0x30, 0x2f, 0x57, 0x33, 0x20, 0x4d, 0x41, 0x52, 0x43, 0x41, 0x4e, 0x20, 0x55, 0x49,
            0x44, 0x49, 0x47, 0x49, 0x20, 0x42, 0x4f, 0x49, 0x45, 0x53, 0x54, 0x4f, 0x57, 0x4e,
            0x2c, 0x20, 0x4e, 0x42,
        ];

        let encoded_ascii = b"VE9BCQ>APNU19,VE9DGP,VE9GFI-2,VE9FPG*,WIDE3:!4627.20NS06631.19W#PHG5460/W3 MARCAN UIDIGI BOIESTOWN, NB";

        // ascii -> ax25
        let decoded_from_ascii = AprsPacket::decode_textual(&encoded_ascii[..]).unwrap();
        let mut actual_ax25 = vec![];
        decoded_from_ascii.encode_ax25(&mut actual_ax25).unwrap();
        assert_eq!(encoded_ax25, actual_ax25);

        // ax25 -> ascii
        let decoded_from_ax25 = AprsPacket::decode_ax25(&encoded_ax25).unwrap();
        let mut actual_ascii = vec![];
        decoded_from_ax25.encode_textual(&mut actual_ascii).unwrap();
        assert_eq!(encoded_ascii[..], actual_ascii);

        // both -> packet
        assert_eq!(decoded_from_ascii, decoded_from_ax25);
    }

    #[test]
    fn parse_packet_mic_e() {
        let result = AprsPacket::decode_textual(
            &br#"DF1CHB-9>UQ0RT6,ARISS,APRSAT,WIDE1-1,qAU,DB0KOE-12:`|9g"H?>/>"4z}="#[..],
        )
        .unwrap();

        assert_eq!(
            AprsPacket {
                from: Callsign::new_with_ssid("DF1CHB", "9"),
                via: vec![
                    Via::Callsign(Callsign::new_no_ssid("ARISS"), false),
                    Via::Callsign(Callsign::new_no_ssid("APRSAT"), false),
                    Via::Callsign(Callsign::new_with_ssid("WIDE1", "1"), false),
                    Via::QConstruct(QConstruct::AU),
                    Via::Callsign(Callsign::new_with_ssid("DB0KOE", "12"), false)
                ],
                data: AprsData::MicE(AprsMicE {
                    latitude: Latitude::new(51.041).unwrap(),
                    longitude: Longitude::new(6.495833333333334).unwrap(),
                    precision: Precision::HundredthMinute,
                    message: Message::M1,
                    speed: Speed::new(64).unwrap(),
                    course: Course::new(35).unwrap(),
                    symbol_table: b'/',
                    symbol_code: b'>',
                    comment: br#">"4z}="#.to_vec(),
                    current: true
                })
            },
            result
        );
    }

    #[test]
    fn encode_edge_case() {
        let packet = AprsPacket {
            from: Callsign::new_no_ssid("D9KS3"),
            via: vec![],
            data: AprsData::Position(AprsPosition {
                to: Callsign::new_no_ssid("NOBODY"),
                timestamp: None,
                messaging_supported: true,
                latitude: Latitude::new(33.999999999999).unwrap(),
                longitude: Longitude::new(33.999999999999).unwrap(),
                precision: Precision::HundredthMinute,
                symbol_table: '/',
                symbol_code: 'c',
                comment: b"Hello world".to_vec(),
                cst: AprsCst::Uncompressed,
            }),
        };

        let mut buf = vec![];
        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(
            "D9KS3>NOBODY:=3400.00N/03400.00EcHello world",
            String::from_utf8(buf).unwrap()
        );
    }

    #[test]
    fn encode_with_ssid_0() {
        let packet = AprsPacket {
            from: Callsign::new_with_ssid("D9KS3", "0"),
            via: vec![Via::Callsign(Callsign::new("D9KS7-0").unwrap(), false)],
            data: AprsData::Position(AprsPosition {
                to: Callsign::new_with_ssid("NOBODY", "0"),
                timestamp: None,
                messaging_supported: true,
                latitude: Latitude::new(3.95).unwrap(),
                longitude: Longitude::new(-4.58).unwrap(),
                precision: Precision::HundredthMinute,
                symbol_table: '/',
                symbol_code: 'c',
                comment: b"Hello world".to_vec(),
                cst: AprsCst::Uncompressed,
            }),
        };

        let mut buf = vec![];
        packet.encode_textual(&mut buf).unwrap();
        assert_eq!(
            "D9KS3>NOBODY,D9KS7:=0357.00N/00434.80WcHello world",
            String::from_utf8(buf).unwrap()
        );
    }

    #[test]
    fn e2e_serialize_deserialize() {
        let valids = vec![
            r"3D17F2>APRS,qAS,DL4MEA:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:@074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,qAS,DL4MEA:!4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:!48  .  N\01200.00E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:=4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,qAS,DL4MEA::DEST     :Hello World! This msg has a : colon {32975",
            r"IC17F2>APRS,qAS,DL4MEA::DESTINATI:Hello World! This msg has a : colon ",
            r"ICA7F2>APRS,qAS,DL4MEA:>312359zStatus seems okay!",
            r"ICA3F2>APRS,qAS,DL4MEA:>184050hAlso with HMS format...",
            "VE9MP-12>T5RX8P,VE9GFI-2,WIDE1*,WIDE2-1,qAR,VE9QLE-10:`]Q\x1cl|ok/'\"4<}Nick - Monitoring IRG|!\"&7'M|!wTD!|3",
            r#"DF1CHB-9>UQ0RT6,ARISS,APRSAT,WIDE1-1,qAU,DB0KOE-1:`|9g\"H?>/>\"4z}="#,
        ];

        for v in valids {
            let mut buf = vec![];
            let packet = AprsPacket::decode_textual(v.as_bytes()).unwrap();
            packet.encode_textual(&mut buf).unwrap();
            assert_eq!(buf, v.as_bytes(), "\n{}\n{}", buf.escape_ascii(), v);
        }
    }

    #[test]
    fn e2e_serialize_deserialize_ax25() {
        let originals = vec![
            r"3D17F2>APRS,qAS,DL4MEA*:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:@074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,qAS,dl4mea:!4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:!48  .  N\01200.00E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,qAS,DL4MEA:=4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,qAS,DL4MEA::DEST     :Hello World! This msg has a : colon {32975",
            r"IC17F2>APRS,qAS,DL4MEA::DESTINATI:Hello World! This msg has a : colon ",
            r"ICA7F2>APRS,qAS,DL4MEA:>312359zStatus seems okay!",
            r"ICA3F2>APRS,qAS,DL4MEA:>184050hAlso with HMS format...",
            "VE9MP-12>T5RX8P,VE9GFI-2,WIDE1*,WIDE2-1,qAR,VE9QLE-10:`]Q\x1cl|ok/'\"4<}Nick - Monitoring IRG|!\"&7'M|!wTD!|3",
            r#"DF1CHB-9>UQ0RT6,ARISS,APRSAT,WIDE1-1,qAU,DB0KOE-1:`|9g\"H?>/>\"4z}="#,
            // 0 to 8 via callsigns
            r"ICA3F2>APRS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIJ,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIJ,KLM,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,NIJ,KLM,QRZ,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ,SDKKA,qAS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ,SDKKA,ABC,qAS:>184050hAlso with HMS format...",
        ];

        // capitalized and q-codes removed
        let expected = vec![
            r"3D17F2>APRS,DL4MEA*:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,DL4MEA:@074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,DL4MEA:!4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,DL4MEA:!48  .  N\01200.00E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"3D17F2>APRS,DL4MEA:=4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1",
            r"ID17F2>APRS,DL4MEA::DEST     :Hello World! This msg has a : colon {32975",
            r"IC17F2>APRS,DL4MEA::DESTINATI:Hello World! This msg has a : colon ",
            r"ICA7F2>APRS,DL4MEA:>312359zStatus seems okay!",
            r"ICA3F2>APRS,DL4MEA:>184050hAlso with HMS format...",
            "VE9MP-12>T5RX8P,VE9GFI-2,WIDE1*,WIDE2-1,VE9QLE-10:`]Q\x1cl|ok/'\"4<}Nick - Monitoring IRG|!\"&7'M|!wTD!|3",
            r#"DF1CHB-9>UQ0RT6,ARISS,APRSAT,WIDE1-1,DB0KOE-1:`|9g\"H?>/>\"4z}="#,
            // 0 to 8 via callsigns
            r"ICA3F2>APRS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIJ:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIJ,KLM:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,NIJ,KLM,QRZ:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ,SDKKA:>184050hAlso with HMS format...",
            r"ICA3F2>APRS,ABC,DEF,HIK,ASD,NADL,ASKJ,SDKKA,ABC:>184050hAlso with HMS format...",
        ];

        for (o, e) in originals.iter().zip(expected.iter()) {
            let o_packet = AprsPacket::decode_textual(o.as_bytes()).unwrap();

            let mut o_ax25 = vec![];
            o_packet.encode_ax25(&mut o_ax25).unwrap();
            let o_pkt_from_ax25 = AprsPacket::decode_ax25(&o_ax25).unwrap();

            let mut o_re_encoded = vec![];
            o_pkt_from_ax25.encode_textual(&mut o_re_encoded).unwrap();

            // text -> packet -> ax25 -> packet -> text
            assert_eq!(
                e.as_bytes(),
                o_re_encoded,
                "\n{}\n{}",
                e,
                String::from_utf8_lossy(&o_re_encoded)
            );

            // o(text) -> packet -> ax25
            // VS.
            // e(text) -> packet -> ax25
            let e_packet = AprsPacket::decode_textual(e.as_bytes()).unwrap();
            let mut e_ax25 = vec![];
            e_packet.encode_ax25(&mut e_ax25).unwrap();

            assert_eq!(e_ax25, o_ax25);
        }
    }

    #[test]
    fn e2e_invalid_string_msg() {
        let original = b"ICA7F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has raw bytes that are invalid utf8! \xc3\x28 {32975";

        let mut buf = vec![];
        let decoded = AprsPacket::decode_textual(&original[..]).unwrap();
        decoded.encode_textual(&mut buf).unwrap();
        assert_eq!(buf, original);
    }
}
