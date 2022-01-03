use std::ops::Deref;
use std::str::FromStr;
use AprsError;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Latitude(f32);

impl Deref for Latitude {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Latitude {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();

        if b.len() != 8 || b[4] as char != '.' {
            return Err(Self::Err::InvalidLatitude(s.to_owned()));
        }

        let north = match b[7] as char {
            'N' => true,
            'S' => false,
            _ => return Err(Self::Err::InvalidLatitude(s.to_owned())),
        };

        let deg = s[0..2]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLatitude(s.to_owned()))? as f32;
        let min = s[2..4]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLatitude(s.to_owned()))? as f32;
        let min_frac = s[5..7]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLatitude(s.to_owned()))? as f32;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if north { value } else { -value };

        if value > 90. || value < -90. {
            return Err(Self::Err::InvalidLatitude(s.to_owned()));
        }

        Ok(Self(value))
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Longitude(f32);

impl Deref for Longitude {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Longitude {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();

        if b.len() != 9 || b[5] as char != '.' {
            return Err(Self::Err::InvalidLongitude(s.to_owned()));
        }

        let east = match b[8] as char {
            'E' => true,
            'W' => false,
            _ => return Err(Self::Err::InvalidLongitude(s.to_owned())),
        };

        let deg = s[0..3]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLongitude(s.to_owned()))? as f32;
        let min = s[3..5]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLongitude(s.to_owned()))? as f32;
        let min_frac = s[6..8]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidLongitude(s.to_owned()))? as f32;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if east { value } else { -value };

        if value > 180. || value < -180. {
            return Err(Self::Err::InvalidLongitude(s.to_owned()));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latitude() {
        assert_relative_eq!(*"4903.50N".parse::<Latitude>().unwrap(), 49.05833);
        assert_relative_eq!(*"4903.50S".parse::<Latitude>().unwrap(), -49.05833);
        assert_eq!(
            "4903.50W".parse::<Latitude>(),
            Err(AprsError::InvalidLatitude("4903.50W".to_owned()))
        );
        assert_eq!(
            "4903.50E".parse::<Latitude>(),
            Err(AprsError::InvalidLatitude("4903.50E".to_owned()))
        );
        assert_eq!(
            "9903.50N".parse::<Latitude>(),
            Err(AprsError::InvalidLatitude("9903.50N".to_owned()))
        );
        assert_relative_eq!(*"0000.00N".parse::<Latitude>().unwrap(), 0.0);
        assert_relative_eq!(*"0000.00S".parse::<Latitude>().unwrap(), 0.0);
    }

    #[test]
    fn test_longitude() {
        assert_relative_eq!(*"12903.50E".parse::<Longitude>().unwrap(), 129.05833);
        assert_relative_eq!(*"04903.50W".parse::<Longitude>().unwrap(), -49.05833);
        assert_eq!(
            "04903.50N".parse::<Longitude>(),
            Err(AprsError::InvalidLongitude("04903.50N".to_owned()))
        );
        assert_eq!(
            "04903.50S".parse::<Longitude>(),
            Err(AprsError::InvalidLongitude("04903.50S".to_owned()))
        );
        assert_eq!(
            "18903.50E".parse::<Longitude>(),
            Err(AprsError::InvalidLongitude("18903.50E".to_owned()))
        );
        assert_relative_eq!(*"00000.00E".parse::<Longitude>().unwrap(), 0.0);
        assert_relative_eq!(*"00000.00W".parse::<Longitude>().unwrap(), 0.0);
    }
}
