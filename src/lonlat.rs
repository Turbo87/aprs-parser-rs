use std::ops::Deref;
use std::str::FromStr;
use AprsError;
use EncodeError;

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

pub fn encode_latitude(lat: Latitude) -> Result<String, EncodeError> {
    let lat = lat.0;

    if !(-90.0..=90.0).contains(&lat) {
        return Err(EncodeError::InvalidLatitude(lat));
    }

    let (dir, lat) = if lat >= 0.0 { ('N', lat) } else { ('S', -lat) };

    let deg = lat as u32;
    let min = ((lat - (deg as f32)) * 60.0) as u32;
    let min_frac = ((lat - (deg as f32) - (min as f32 / 60.0)) * 6000.0).round() as u32;

    Ok(format!("{:02}{:02}.{:02}{}", deg, min, min_frac, dir))
}

pub fn encode_longitude(lon: Longitude) -> Result<String, EncodeError> {
    let lon = lon.0;

    if !(-180.0..=180.0).contains(&lon) {
        return Err(EncodeError::InvalidLongitude(lon));
    }

    let (dir, lon) = if lon >= 0.0 { ('E', lon) } else { ('W', -lon) };

    let deg = lon as u32;
    let min = ((lon - (deg as f32)) * 60.0) as u32;
    let min_frac = ((lon - (deg as f32) - (min as f32 / 60.0)) * 6000.0).round() as u32;

    Ok(format!("{:03}{:02}.{:02}{}", deg, min, min_frac, dir))
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

    #[test]
    fn test_encode_latitude() {
        assert_eq!(
            encode_latitude(Latitude(49.05833)),
            Ok("4903.50N".to_string())
        );
        assert_eq!(
            encode_latitude(Latitude(-49.05833)),
            Ok("4903.50S".to_string())
        );
        assert_eq!(
            encode_latitude(Latitude(-90.1)),
            Err(EncodeError::InvalidLatitude(-90.1))
        );
        assert_eq!(
            encode_latitude(Latitude(90.1)),
            Err(EncodeError::InvalidLatitude(90.1))
        );
        assert_eq!(encode_latitude(Latitude(0.0)), Ok("0000.00N".to_string()));
    }

    #[test]
    fn test_encode_longitude() {
        assert_eq!(
            encode_longitude(Longitude(129.05833)),
            Ok("12903.50E".to_string())
        );
        assert_eq!(
            encode_longitude(Longitude(-49.05833)),
            Ok("04903.50W".to_string())
        );
        assert_eq!(
            encode_longitude(Longitude(-180.1)),
            Err(EncodeError::InvalidLongitude(-180.1))
        );
        assert_eq!(
            encode_longitude(Longitude(180.1)),
            Err(EncodeError::InvalidLongitude(180.1))
        );
        assert_eq!(
            encode_longitude(Longitude(0.0)),
            Ok("00000.00E".to_string())
        );
    }
}
