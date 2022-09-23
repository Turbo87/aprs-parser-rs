use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use AprsError;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Callsign {
    pub call: String,
    pub ssid: Option<String>,
}

impl Callsign {
    pub fn new<T: Into<String>>(call: T, ssid: Option<T>) -> Callsign {
        Callsign {
            call: call.into(),
            ssid: ssid.map(|ssid| ssid.into()),
        }
    }
}

impl TryFrom<&[u8]> for Callsign {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(b).map_err(|_| AprsError::NonUtf8Callsign(b.to_owned()))?;

        match s.split_once('-') {
            Some((call, ssid)) => {
                if call.is_empty() {
                    Err(AprsError::EmptyCallsign(s.to_owned()))
                } else if ssid.is_empty() {
                    Err(AprsError::EmptySSID(s.to_owned()))
                } else {
                    Ok(Callsign::new(call, Some(ssid)))
                }
            }

            None => Ok(Callsign::new(s, None)),
        }
    }
}

impl Display for Callsign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.call)?;

        if let Some(ssid) = &self.ssid {
            write!(f, "-{}", ssid)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn parse_callsign() {
        assert_eq!(
            "ABCDEF".as_bytes().try_into(),
            Ok(Callsign::new("ABCDEF", None))
        );
    }

    #[test]
    fn parse_with_ssid() {
        assert_eq!(
            "ABCDEF-42".as_bytes().try_into(),
            Ok(Callsign::new("ABCDEF", Some("42")))
        );
    }

    #[test]
    fn empty_callsign() {
        assert_eq!(
            Callsign::try_from("-42".as_bytes()),
            Err(AprsError::EmptyCallsign("-42".to_owned()))
        );
    }

    #[test]
    fn empty_ssid() {
        assert_eq!(
            Callsign::try_from("ABCDEF-".as_bytes()),
            Err(AprsError::EmptySSID("ABCDEF-".to_owned()))
        );
    }

    #[test]
    fn non_utf8() {
        assert_eq!(
            Callsign::try_from(&b"ABCDEF\xF0\xA4\xAD"[..]),
            Err(AprsError::NonUtf8Callsign(b"ABCDEF\xF0\xA4\xAD".to_vec()))
        );
    }

    #[test]
    fn display_no_ssid() {
        assert_eq!("ABCDEF", format!("{}", Callsign::new("ABCDEF", None)));
    }

    #[test]
    fn display_with_ssid() {
        assert_eq!(
            "ABCDEF-12",
            format!("{}", Callsign::new("ABCDEF", Some("12")))
        );
    }
}
