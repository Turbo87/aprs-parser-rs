use std::ops::Deref;
use AprsError;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Latitude(f32);

impl Deref for Latitude {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_latitude(s: &str) -> Result<Latitude, AprsError> {
    let b = s.as_bytes();

    if b.len() != 8 || b[4] as char != '.' {
        return Err(AprsError::InvalidLatitude(s.to_owned()));
    }

    let north = match b[7] as char {
        'N' => true,
        'S' => false,
        _ => return Err(AprsError::InvalidLatitude(s.to_owned())),
    };

    let deg = s[0..2]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLatitude(s.to_owned()))? as f32;
    let min = s[2..4]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLatitude(s.to_owned()))? as f32;
    let min_frac = s[5..7]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLatitude(s.to_owned()))? as f32;

    let value = deg + min / 60. + min_frac / 6_000.;
    let value = if north { value } else { -value };

    if value > 90. || value < -90. {
        return Err(AprsError::InvalidLatitude(s.to_owned()));
    }

    Ok(Latitude(value))
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Longitude(f32);

impl Deref for Longitude {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_longitude(s: &str) -> Result<Longitude, AprsError> {
    let b = s.as_bytes();

    if b.len() != 9 || b[5] as char != '.' {
        return Err(AprsError::InvalidLongitude(s.to_owned()));
    }

    let east = match b[8] as char {
        'E' => true,
        'W' => false,
        _ => return Err(AprsError::InvalidLongitude(s.to_owned())),
    };

    let deg = s[0..3]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLongitude(s.to_owned()))? as f32;
    let min = s[3..5]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLongitude(s.to_owned()))? as f32;
    let min_frac = s[6..8]
        .parse::<u32>()
        .map_err(|_| AprsError::InvalidLongitude(s.to_owned()))? as f32;

    let value = deg + min / 60. + min_frac / 6_000.;
    let value = if east { value } else { -value };

    if value > 180. || value < -180. {
        return Err(AprsError::InvalidLongitude(s.to_owned()));
    }

    Ok(Longitude(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latitude() {
        assert_relative_eq!(*parse_latitude("4903.50N").unwrap(), 49.05833);
        assert_relative_eq!(*parse_latitude("4903.50S").unwrap(), -49.05833);
        assert_eq!(
            parse_latitude("4903.50W"),
            Err(AprsError::InvalidLatitude("4903.50W".to_owned()))
        );
        assert_eq!(
            parse_latitude("4903.50E"),
            Err(AprsError::InvalidLatitude("4903.50E".to_owned()))
        );
        assert_relative_eq!(*parse_latitude("0000.00N").unwrap(), 0.0);
        assert_relative_eq!(*parse_latitude("0000.00S").unwrap(), 0.0);
    }

    #[test]
    fn test_longitude() {
        assert_relative_eq!(*parse_longitude("12903.50E").unwrap(), 129.05833);
        assert_relative_eq!(*parse_longitude("04903.50W").unwrap(), -49.05833);
        assert_eq!(
            parse_longitude("04903.50N"),
            Err(AprsError::InvalidLongitude("04903.50N".to_owned()))
        );
        assert_eq!(
            parse_longitude("04903.50S"),
            Err(AprsError::InvalidLongitude("04903.50S".to_owned()))
        );
        assert_relative_eq!(*parse_longitude("00000.00E").unwrap(), 0.0);
        assert_relative_eq!(*parse_longitude("00000.00W").unwrap(), 0.0);
    }
}
