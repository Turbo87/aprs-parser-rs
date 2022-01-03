use std::str::FromStr;

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

impl FromStr for Callsign {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let delimiter = s.find('-'); //.ok_or_else(|| AprsError::EmptyCallsign(s.to_owned()))?;
        if delimiter.is_none() {
            return Ok(Callsign::new(s, None));
        }

        let delimiter = delimiter.unwrap();
        if delimiter == 0 {
            return Err(AprsError::EmptyCallsign(s.to_owned()));
        }

        let (call, rest) = s.split_at(delimiter);
        let ssid = &rest[1..rest.len()];

        if ssid.is_empty() {
            return Err(AprsError::EmptySSID(s.to_owned()));
        }

        Ok(Callsign::new(call, Some(ssid)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_callsign() {
        assert_eq!("ABCDEF".parse(), Ok(Callsign::new("ABCDEF", None)));
    }

    #[test]
    fn parse_with_ssid() {
        assert_eq!("ABCDEF-42".parse(), Ok(Callsign::new("ABCDEF", Some("42"))));
    }

    #[test]
    fn empty_callsign() {
        assert_eq!(
            "-42".parse::<Callsign>(),
            Err(AprsError::EmptyCallsign("-42".to_owned()))
        );
    }

    #[test]
    fn empty_ssid() {
        assert_eq!(
            "ABCDEF-".parse::<Callsign>(),
            Err(AprsError::EmptySSID("ABCDEF-".to_owned()))
        );
    }
}
