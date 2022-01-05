use std::str::FromStr;

use AprsError;
use AprsPosition;
use Callsign;

#[derive(PartialEq, Debug, Clone)]
pub struct AprsMessage {
    pub from: Callsign,
    pub to: Callsign,
    pub via: Vec<Callsign>,
    pub data: AprsData,
}

impl FromStr for AprsMessage {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let header_delimiter = s
            .find(':')
            .ok_or_else(|| AprsError::InvalidMessage(s.to_owned()))?;
        let (header, rest) = s.split_at(header_delimiter);
        let body = &rest[1..];

        let from_delimiter = header
            .find('>')
            .ok_or_else(|| AprsError::InvalidMessage(s.to_owned()))?;
        let (from, rest) = header.split_at(from_delimiter);
        let from = Callsign::from_str(from)?;

        let to_and_via = &rest[1..];
        let to_and_via: Vec<_> = to_and_via.split(',').collect();

        let to = to_and_via
            .first()
            .ok_or_else(|| AprsError::InvalidMessage(s.to_owned()))?;
        let to = Callsign::from_str(to)?;

        let mut via = vec![];
        for v in to_and_via.iter().skip(1) {
            via.push(Callsign::from_str(v)?);
        }

        let data = AprsPosition::from_str(body)
            .map(AprsData::Position)
            .unwrap_or(AprsData::Unknown);

        Ok(AprsMessage {
            from,
            to,
            via,
            data,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AprsData {
    Position(AprsPosition),
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use Timestamp;

    #[test]
    fn parse() {
        let result = r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1".parse::<AprsMessage>().unwrap();
        assert_eq!(result.from, Callsign::new("ICA3D17F2", None));
        assert_eq!(result.to, Callsign::new("APRS", None));
        assert_eq!(
            result.via,
            vec![Callsign::new("qAS", None), Callsign::new("dl4mea", None),]
        );

        match result.data {
            AprsData::Position(position) => {
                assert_eq!(position.timestamp, Some(Timestamp::HHMMSS(7, 48, 49)));
                assert_relative_eq!(*position.latitude, 48.360166);
                assert_relative_eq!(*position.longitude, 12.408166);
                assert_eq!(position.symbol_table, '\\');
                assert_eq!(position.symbol_code, '^');
                assert_eq!(
                    position.comment,
                    "322/103/A=003054 !W09! id213D17F2 -039fpm +0.0rot 2.5dB 3e -0.0kHz gps1x1"
                );
            }
            _ => panic!("Unexpected data type"),
        }
    }
}
