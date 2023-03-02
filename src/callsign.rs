use std::fmt::{Display, Formatter};
use std::io::{self, Write};

use EncodeError;

pub enum CallsignField {
    Destination,
    Source,
    Via(bool),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Callsign {
    call: String,
    ssid: Option<String>,
}

impl Callsign {
    /// Create a new callsign.
    /// SSID is parsed out.
    pub fn new(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref().split_once('-') {
            Some((call, ssid)) => {
                if call.is_empty() || ssid.is_empty() {
                    None
                } else {
                    Some(Callsign::new_with_ssid(call.to_owned(), ssid))
                }
            }

            None => Some(Callsign::new_no_ssid(s.as_ref())),
        }
    }

    /// Create a new callsign
    /// Note: If you need a callsign with an SSID, use `new_with_ssid`.
    pub fn new_no_ssid(call: impl Into<String>) -> Callsign {
        let call = call.into();
        let ssid = None;
        Callsign { call, ssid }
    }

    pub fn new_with_ssid(call: impl Into<String>, ssid: impl Into<String>) -> Callsign {
        let call = call.into();
        let ssid = Some(ssid.into());
        Callsign { call, ssid }
    }

    pub fn encode_textual<W: Write>(&self, heard: bool, w: &mut W) -> io::Result<()> {
        write!(w, "{}", self)?;

        if heard {
            write!(w, "*")?;
        }

        Ok(())
    }

    pub fn decode_textual(bytes: &[u8]) -> Option<(Self, bool)> {
        let (bytes, heard) = if bytes.last() == Some(&b'*') {
            (&bytes[0..(bytes.len() - 1)], true)
        } else {
            (bytes, false)
        };

        let s = std::str::from_utf8(bytes).ok()?;

        Self::new(s).map(|c| (c, heard))
    }

    pub fn encode_ax25<W: Write>(
        &self,
        buf: &mut W,
        field: CallsignField,
        has_more: bool,
    ) -> Result<(), EncodeError> {
        // callsign requirements:
        // <= 6 bytes long
        // all alphanumeric uppercase
        // ssid missing or a number between 0 and 15

        let call = self.call.as_bytes();
        if call.len() > 6 {
            return Err(EncodeError::InvalidCallsign(self.clone()));
        }

        let ssid: u8 = self
            .ssid
            .clone()
            .map(|x| x.parse().ok())
            .unwrap_or(Some(0))
            .ok_or_else(|| EncodeError::InvalidCallsign(self.clone()))?;

        if ssid > 15 {
            return Err(EncodeError::InvalidCallsign(self.clone()));
        }

        let has_more = if has_more { 0 } else { 1 };

        for c in call {
            if !c.is_ascii_alphanumeric() {
                return Err(EncodeError::InvalidCallsign(self.clone()));
            }

            buf.write_all(&[c.to_ascii_uppercase() << 1])?;
        }

        for _ in call.len()..6 {
            buf.write_all(&[b' ' << 1])?;
        }

        match field {
            CallsignField::Destination => {
                buf.write_all(&[0b11100000 | (ssid << 1) | has_more])?;
            }
            CallsignField::Source => {
                buf.write_all(&[0b01100000 | (ssid << 1) | has_more])?;
            }
            CallsignField::Via(heard) => {
                let heard = if heard { 1 } else { 0 };

                buf.write_all(&[0b01100000 | (ssid << 1) | (heard << 7) | has_more])?;
            }
        }

        Ok(())
    }

    // Returns self, heard, and the flag for if there's another callsign after
    pub fn decode_ax25(data: &[u8]) -> Option<(Self, bool, bool)> {
        if data.len() != 7 {
            return None;
        }

        let mut call = String::new();
        let mut found_space = false;
        for d in &data[0..6] {
            // LSB must be cleared
            if (d & 0x01) != 0 {
                return None;
            }
            let d = d >> 1;

            if d == b' ' {
                found_space = true;
                continue;
            }

            // non-space char after a space
            if found_space {
                return None;
            }

            if !d.is_ascii_alphanumeric() {
                return None;
            }

            call.push(d.to_ascii_uppercase().into());
        }

        let s = data[6];
        let heard = (s & 0x80) != 0;
        let has_more = (s & 0x01) == 0;
        let ssid = (s & 0x1E) >> 1;

        let ssid = if ssid == 0 {
            None
        } else {
            Some(format!("{}", ssid))
        };

        Some((Callsign { call, ssid }, heard, has_more))
    }
}

impl Display for Callsign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.call)?;

        if let Some(ssid) = &self.ssid {
            if !ssid.is_empty() {
                write!(f, "-{}", ssid)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_callsign() {
        assert_eq!(
            Callsign::decode_textual(&b"ABCDEF"[..]),
            Some((Callsign::new("ABCDEF").unwrap(), false))
        );
    }

    #[test]
    fn parse_heard_callsign() {
        assert_eq!(
            Callsign::decode_textual(&b"ABCDEF*"[..]),
            Some((Callsign::new("ABCDEF").unwrap(), true))
        );
    }

    #[test]
    fn parse_with_ssid() {
        assert_eq!(
            Callsign::decode_textual(&b"ABCDEF-2"[..]),
            Some((Callsign::new_with_ssid("ABCDEF", "2"), false))
        );
    }

    #[test]
    fn omit_end_spaces() {
        assert_eq!(
            Callsign::decode_ax25(&[172, 138, 114, 64, 64, 64, 1]),
            Some((Callsign::new_no_ssid("VE9"), false, false))
        )
    }

    #[test]
    fn spaces_in_middle() {
        assert_eq!(Callsign::decode_ax25(&[172, 64, 114, 64, 64, 64, 1]), None)
    }

    #[test]
    fn uppercase_callsign() {
        assert_eq!(
            Callsign::decode_ax25(&[236, 202, 114, 64, 64, 64, 1]),
            Some((Callsign::new_no_ssid("VE9"), false, false))
        );

        let mut buf = vec![];
        let c = Callsign::new_no_ssid("ve9");
        c.encode_ax25(&mut buf, CallsignField::Destination, false)
            .unwrap();
        assert_eq!(&[172, 138, 114, 64, 64, 64, 225][..], buf);

        // sanity check with callsign that's already uppercased
        let mut buf = vec![];
        let c = Callsign::new_no_ssid("VE9");
        c.encode_ax25(&mut buf, CallsignField::Destination, false)
            .unwrap();
        assert_eq!(&[172, 138, 114, 64, 64, 64, 225][..], buf);
    }

    #[test]
    fn non_alphanumeric() {
        let mut buf = vec![];
        assert!(matches!(
        Callsign::new_no_ssid("VE9---").encode_ax25(
            &mut buf,
            CallsignField::Destination,
            false
        ),
        Err(EncodeError::InvalidCallsign(c)) if c == Callsign::new_no_ssid(
            "VE9---"
        )));
    }

    #[test]
    fn callsign_too_long() {
        let mut buf = vec![];
        assert!(matches!(
            Callsign::new_no_ssid("VE9ABCD").encode_ax25(
                &mut buf,
                CallsignField::Source,
                false
            ),
            Err(EncodeError::InvalidCallsign(c)) if c == Callsign::new_no_ssid(
                "VE9ABCD"
            )
        ));
    }

    #[test]
    fn empty_callsign() {
        assert_eq!(Callsign::decode_textual("-3".as_bytes()), None);
    }

    #[test]
    fn empty_ssid() {
        assert_eq!(Callsign::decode_textual("ABCDEF-".as_bytes()), None);
    }

    #[test]
    fn non_utf8() {
        assert_eq!(Callsign::decode_textual(&b"ABCDEF\xF0\xA4\xAD"[..]), None);
    }

    #[test]
    fn textual_no_ssid() {
        let c = Callsign::new_no_ssid("ABCDEF");

        assert_eq!("ABCDEF", format!("{}", c));

        let mut buf = vec![];
        c.encode_textual(true, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF*"[..], buf);

        buf.clear();
        c.encode_textual(false, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF"[..], buf);
    }

    #[test]
    fn textual_with_ssid() {
        let c = Callsign::new_with_ssid("ABCDEF", "XF");

        assert_eq!("ABCDEF-XF", format!("{}", c));

        let mut buf = vec![];
        c.encode_textual(true, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF-XF*"[..], buf);

        buf.clear();
        c.encode_textual(false, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF-XF"[..], buf);
    }

    #[test]
    fn textual_non_ascii_characters() {
        let c = Callsign::new_with_ssid("ABCDEF\x001\x002", "XF\x002\x003");

        assert_eq!("ABCDEF\x001\x002-XF\x002\x003", format!("{}", c));

        let mut buf = vec![];
        c.encode_textual(true, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF\x001\x002-XF\x002\x003*"[..], buf);

        buf.clear();
        c.encode_textual(false, &mut buf).unwrap();
        assert_eq!(&b"ABCDEF\x001\x002-XF\x002\x003"[..], buf);
    }

    #[test]
    fn display_no_ssid() {
        assert_eq!("ABCDEF", format!("{}", Callsign::new_no_ssid("ABCDEF")));
    }

    #[test]
    fn display_with_ssid() {
        assert_eq!(
            "ABCDEF-12",
            format!("{}", Callsign::new_with_ssid("ABCDEF", "12"))
        );
    }
}
