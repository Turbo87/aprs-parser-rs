use std::io::Write;
use std::ops::Deref;

use base91;
use bytes::parse_bytes;
use AprsError;
use EncodeError;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Latitude(f64);

impl Deref for Latitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Latitude {
    pub(crate) fn parse_uncompressed(b: &[u8]) -> Result<Self, AprsError> {
        if b.len() != 8 || b[4] != b'.' {
            return Err(AprsError::InvalidLatitude(b.to_owned()));
        }

        let north = match b[7] {
            b'N' => true,
            b'S' => false,
            _ => return Err(AprsError::InvalidLatitude(b.to_owned())),
        };

        let deg = parse_bytes::<u32>(&b[0..2])
            .ok_or_else(|| AprsError::InvalidLatitude(b.to_owned()))? as f64;
        let min = parse_bytes::<u32>(&b[2..4])
            .ok_or_else(|| AprsError::InvalidLatitude(b.to_owned()))? as f64;
        let min_frac = parse_bytes::<u32>(&b[5..7])
            .ok_or_else(|| AprsError::InvalidLatitude(b.to_owned()))? as f64;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if north { value } else { -value };

        if value > 90. || value < -90. {
            return Err(AprsError::InvalidLatitude(b.to_owned()));
        }

        Ok(Self(value))
    }

    pub(crate) fn parse_compressed(b: &[u8]) -> Result<Self, AprsError> {
        let value = 90.0
            - (base91::decode_ascii(b).ok_or_else(|| AprsError::InvalidLatitude(b.to_owned()))?
                / 380926.0);
        if value > 90. || value < -90. {
            return Err(AprsError::InvalidLatitude(b.to_owned()));
        }

        Ok(Self(value))
    }

    pub(crate) fn encode_compressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let value = (90.0 - self.0) * 380926.0;
        base91::encode_ascii(value, buf, 4)
    }

    pub(crate) fn encode_uncompressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let lat = self.0;

        if !(-90.0..=90.0).contains(&lat) {
            return Err(EncodeError::InvalidLatitude(lat));
        }

        let (dir, lat) = if lat >= 0.0 { ('N', lat) } else { ('S', -lat) };

        let deg = lat as u32;
        let min = ((lat - (deg as f64)) * 60.0) as u32;
        let min_frac = ((lat - (deg as f64) - (min as f64 / 60.0)) * 6000.0).round() as u32;

        write!(buf, "{:02}{:02}.{:02}{}", deg, min, min_frac, dir)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Longitude(f64);

impl Deref for Longitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Longitude {
    pub(crate) fn parse_uncompressed(b: &[u8]) -> Result<Self, AprsError> {
        if b.len() != 9 || b[5] != b'.' {
            return Err(AprsError::InvalidLongitude(b.to_owned()));
        }

        let east = match b[8] {
            b'E' => true,
            b'W' => false,
            _ => return Err(AprsError::InvalidLongitude(b.to_owned())),
        };

        let deg = parse_bytes::<u32>(&b[0..3])
            .ok_or_else(|| AprsError::InvalidLongitude(b.to_owned()))? as f64;
        let min = parse_bytes::<u32>(&b[3..5])
            .ok_or_else(|| AprsError::InvalidLongitude(b.to_owned()))? as f64;
        let min_frac = parse_bytes::<u32>(&b[6..8])
            .ok_or_else(|| AprsError::InvalidLongitude(b.to_owned()))?
            as f64;

        let value = deg + min / 60. + min_frac / 6_000.;
        let value = if east { value } else { -value };

        if value > 180. || value < -180. {
            return Err(AprsError::InvalidLongitude(b.to_owned()));
        }

        Ok(Self(value))
    }

    pub(crate) fn parse_compressed(b: &[u8]) -> Result<Self, AprsError> {
        let value = (base91::decode_ascii(b)
            .ok_or_else(|| AprsError::InvalidLongitude(b.to_owned()))?
            / 190463.0)
            - 180.0;
        if value > 180. || value < -180. {
            return Err(AprsError::InvalidLongitude(b.to_owned()));
        }

        Ok(Self(value))
    }

    pub(crate) fn encode_compressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let value = (180.0 + self.0) * 190463.0;
        base91::encode_ascii(value, buf, 4)
    }

    pub(crate) fn encode_uncompressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let lon = self.0;

        if !(-180.0..=180.0).contains(&lon) {
            return Err(EncodeError::InvalidLongitude(lon));
        }

        let (dir, lon) = if lon >= 0.0 { ('E', lon) } else { ('W', -lon) };

        let deg = lon as u32;
        let min = ((lon - (deg as f64)) * 60.0) as u32;
        let min_frac = ((lon - (deg as f64) - (min as f64 / 60.0)) * 6000.0).round() as u32;

        write!(buf, "{:03}{:02}.{:02}{}", deg, min, min_frac, dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uncompressed_latitude() {
        assert_relative_eq!(
            *Latitude::parse_uncompressed(&b"4903.50N"[..]).unwrap(),
            49.05833333333333
        );
        assert_relative_eq!(
            *Latitude::parse_uncompressed(&b"4903.50S"[..]).unwrap(),
            -49.05833333333333
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50W"[..]),
            Err(AprsError::InvalidLatitude(b"4903.50W".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50E"[..]),
            Err(AprsError::InvalidLatitude(b"4903.50E".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"9903.50N"[..]),
            Err(AprsError::InvalidLatitude(b"9903.50N".to_vec()))
        );
        assert_relative_eq!(
            *Latitude::parse_uncompressed(&b"0000.00N"[..]).unwrap(),
            0.0
        );
        assert_relative_eq!(
            *Latitude::parse_uncompressed(&b"0000.00S"[..]).unwrap(),
            0.0
        );
    }

    #[test]
    fn test_parse_uncompressed_longitude() {
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"12903.50E"[..]).unwrap(),
            129.05833333333333
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"04903.50W"[..]).unwrap(),
            -49.05833333333333
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"04903.50N"[..]),
            Err(AprsError::InvalidLongitude(b"04903.50N".to_vec()))
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"04903.50S"[..]),
            Err(AprsError::InvalidLongitude(b"04903.50S".to_vec()))
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"18903.50E"[..]),
            Err(AprsError::InvalidLongitude(b"18903.50E".to_vec()))
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.00E"[..]).unwrap(),
            0.0
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.00W"[..]).unwrap(),
            0.0
        );
    }

    #[test]
    fn test_encode_uncompressed_latitude() {
        let mut buf = vec![];
        Latitude(49.05833).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"4903.50N"[..]);

        let mut buf = vec![];
        Latitude(-49.05833).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"4903.50S"[..]);

        let mut buf = vec![];
        assert!(matches!(
            Latitude(-90.1).encode_uncompressed(&mut buf),
            Err(EncodeError::InvalidLatitude(x)) if x == -90.1
        ));
        assert!(matches!(
            Latitude(90.1).encode_uncompressed(&mut buf),
            Err(EncodeError::InvalidLatitude(x)) if x == 90.1
        ));

        let mut buf = vec![];
        Latitude(0.0).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"0000.00N"[..]);
    }

    #[test]
    fn test_encode_uncompressed_longitude() {
        let mut buf = vec![];
        Longitude(129.05833).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"12903.50E"[..]);

        let mut buf = vec![];
        Longitude(-49.0583).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"04903.50W"[..]);

        let mut buf = vec![];
        assert!(matches!(
            Longitude(-180.1).encode_uncompressed(&mut buf),
            Err(EncodeError::InvalidLongitude(x)) if x == -180.1
        ));
        assert!(matches!(
            Longitude(180.1).encode_uncompressed(&mut buf),
            Err(EncodeError::InvalidLongitude(x)) if x == 180.1
        ));

        let mut buf = vec![];
        Longitude(0.0).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"00000.00E"[..]);
    }
}
