use std::convert::TryFrom;
use std::ops::Deref;

use bytes::parse_bytes;
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

impl TryFrom<&[u8]> for Latitude {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        if b.len() != 8 || b[4] != b'.' {
            return Err(Self::Error::InvalidLatitude(b.to_owned()));
        }

        let north = match b[7] {
            b'N' => true,
            b'S' => false,
            _ => return Err(Self::Error::InvalidLatitude(b.to_owned())),
        };

        let deg = parse_bytes::<u32>(&b[0..2])
            .ok_or_else(|| Self::Error::InvalidLatitude(b.to_owned()))? as f32;
        let min = parse_bytes::<u32>(&b[2..4])
            .ok_or_else(|| Self::Error::InvalidLatitude(b.to_owned()))? as f32;
        let min_frac = parse_bytes::<u32>(&b[5..7])
            .ok_or_else(|| Self::Error::InvalidLatitude(b.to_owned()))?
            as f32;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if north { value } else { -value };

        if value > 90. || value < -90. {
            return Err(Self::Error::InvalidLatitude(b.to_owned()));
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

impl TryFrom<&[u8]> for Longitude {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        if b.len() != 9 || b[5] != b'.' {
            return Err(Self::Error::InvalidLongitude(b.to_owned()));
        }

        let east = match b[8] {
            b'E' => true,
            b'W' => false,
            _ => return Err(Self::Error::InvalidLongitude(b.to_owned())),
        };

        let deg = parse_bytes::<u32>(&b[0..3])
            .ok_or_else(|| Self::Error::InvalidLongitude(b.to_owned()))? as f32;
        let min = parse_bytes::<u32>(&b[3..5])
            .ok_or_else(|| Self::Error::InvalidLongitude(b.to_owned()))? as f32;
        let min_frac = parse_bytes::<u32>(&b[6..8])
            .ok_or_else(|| Self::Error::InvalidLongitude(b.to_owned()))?
            as f32;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if east { value } else { -value };

        if value > 180. || value < -180. {
            return Err(Self::Error::InvalidLongitude(b.to_owned()));
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
        assert_relative_eq!(
            *Latitude::try_from("4903.50N".as_bytes()).unwrap(),
            49.05833
        );
        assert_relative_eq!(
            *Latitude::try_from("4903.50S".as_bytes()).unwrap(),
            -49.05833
        );
        assert_eq!(
            Latitude::try_from("4903.50W".as_bytes()),
            Err(AprsError::InvalidLatitude(b"4903.50W".to_vec()))
        );
        assert_eq!(
            Latitude::try_from("4903.50E".as_bytes()),
            Err(AprsError::InvalidLatitude(b"4903.50E".to_vec()))
        );
        assert_eq!(
            Latitude::try_from("9903.50N".as_bytes()),
            Err(AprsError::InvalidLatitude(b"9903.50N".to_vec()))
        );
        assert_relative_eq!(*Latitude::try_from("0000.00N".as_bytes()).unwrap(), 0.0);
        assert_relative_eq!(*Latitude::try_from("0000.00S".as_bytes()).unwrap(), 0.0);
    }

    #[test]
    fn test_longitude() {
        assert_relative_eq!(
            *Longitude::try_from(b"12903.50E".as_slice()).unwrap(),
            129.05833
        );
        assert_relative_eq!(
            *Longitude::try_from(b"04903.50W".as_slice()).unwrap(),
            -49.05833
        );
        assert_eq!(
            Longitude::try_from(&b"04903.50N"[..]),
            Err(AprsError::InvalidLongitude(b"04903.50N".to_vec()))
        );
        assert_eq!(
            Longitude::try_from(&b"04903.50S"[..]),
            Err(AprsError::InvalidLongitude(b"04903.50S".to_vec()))
        );
        assert_eq!(
            Longitude::try_from(&b"18903.50E"[..]),
            Err(AprsError::InvalidLongitude(b"18903.50E".to_vec()))
        );
        assert_relative_eq!(*Longitude::try_from(&b"00000.00E"[..]).unwrap(), 0.0);
        assert_relative_eq!(*Longitude::try_from(&b"00000.00W"[..]).unwrap(), 0.0);
    }

    #[test]
    fn test_encode_latitude() {
        assert_eq!(
            encode_latitude(Latitude(49.05833)).unwrap(),
            "4903.50N".to_string()
        );
        assert_eq!(
            encode_latitude(Latitude(-49.05833)).unwrap(),
            "4903.50S".to_string()
        );
        assert!(matches!(
            encode_latitude(Latitude(-90.1)),
            Err(EncodeError::InvalidLatitude(x)) if x == -90.1
        ));
        assert!(matches!(
            encode_latitude(Latitude(90.1)),
            Err(EncodeError::InvalidLatitude(x)) if x == 90.1
        ));
        assert_eq!(
            encode_latitude(Latitude(0.0)).unwrap(),
            "0000.00N".to_string()
        );
    }

    #[test]
    fn test_encode_longitude() {
        assert_eq!(
            encode_longitude(Longitude(129.05833)).unwrap(),
            "12903.50E".to_string()
        );
        assert_eq!(
            encode_longitude(Longitude(-49.05833)).unwrap(),
            "04903.50W".to_string()
        );
        assert!(matches!(
            encode_longitude(Longitude(-180.1)),
            Err(EncodeError::InvalidLongitude(x)) if x == -180.1
        ));
        assert!(matches!(
            encode_longitude(Longitude(180.1)),
            Err(EncodeError::InvalidLongitude(x)) if x == 180.1
        ));
        assert_eq!(
            encode_longitude(Longitude(0.0)).unwrap(),
            "00000.00E".to_string()
        );
    }
}
