use std::io::Write;
use std::ops::{Deref, RangeInclusive};

use base91;
use bytes::parse_bytes;
use DecodeError;
use EncodeError;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Precision {
    TenDegree,
    OneDegree,
    TenMinute,
    OneMinute,
    TenthMinute,
    HundredthMinute,
}

impl Precision {
    /// Returns the width of the precision in degrees.
    /// For example, `Precision::OneDegree` would return 1.0.
    pub fn width(&self) -> f64 {
        match self {
            Precision::HundredthMinute => 1.0 / 6000.0,
            Precision::TenthMinute => 1.0 / 600.0,
            Precision::OneMinute => 1.0 / 60.0,
            Precision::TenMinute => 1.0 / 6.0,
            Precision::OneDegree => 1.0,
            Precision::TenDegree => 10.0,
        }
    }

    pub(crate) fn range(&self, center: f64) -> RangeInclusive<f64> {
        let width = self.width();

        (center - (width / 2.0))..=(center + (width / 2.0))
    }

    pub(crate) fn num_digits(&self) -> u8 {
        match self {
            Precision::HundredthMinute => 0,
            Precision::TenthMinute => 1,
            Precision::OneMinute => 2,
            Precision::TenMinute => 3,
            Precision::OneDegree => 4,
            Precision::TenDegree => 5,
        }
    }

    pub(crate) fn from_num_digits(digits: u8) -> Option<Self> {
        let res = match digits {
            0 => Precision::HundredthMinute,
            1 => Precision::TenthMinute,
            2 => Precision::OneMinute,
            3 => Precision::TenMinute,
            4 => Precision::OneDegree,
            5 => Precision::TenDegree,
            _ => return None,
        };

        Some(res)
    }
}

impl Default for Precision {
    fn default() -> Self {
        Self::HundredthMinute
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Latitude(f64);

impl Deref for Latitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Latitude {
    /// Creates a new `Latitude`.
    /// Returns `None` if the given value is not a valid latitude.
    pub fn new(value: f64) -> Option<Self> {
        if value > 90.0 || value < -90.0 || value.is_nan() {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Creates a new `Latitude` from degrees, minutes, and hundredths of a minute,
    /// as well as direction
    pub fn from_dmh(deg: u32, min: u32, hundredths: u32, north: bool) -> Option<Self> {
        let value = f64::from(deg) + f64::from(min) / 60. + f64::from(hundredths) / 6_000.;
        let value = if north { value } else { -value };

        Self::new(value)
    }

    /// Returns the `Latitude`'s degrees, minutes, hundredths of a minute, and direction.
    /// `true` is north, `false` is south.
    pub fn dmh(&self) -> (u32, u32, u32, bool) {
        let lat = self.0;

        let (dir, lat) = if lat >= 0.0 {
            (true, lat)
        } else {
            (false, -lat)
        };

        let mut deg = lat as u32;
        let mut min = ((lat - (deg as f64)) * 60.0) as u32;
        let mut hundredths = ((lat - (deg as f64) - (min as f64 / 60.0)) * 6000.0).round() as u32;

        if hundredths == 100 {
            // overflow from the rounding. need to propagate it up
            hundredths = 0;
            min += 1;
        }

        if min == 60 {
            min = 0;
            deg += 1;
        }

        (deg, min, hundredths, dir)
    }

    /// The value of the latitude.
    pub fn value(&self) -> f64 {
        self.0
    }

    pub(crate) fn parse_uncompressed(b: &[u8]) -> Result<(Self, Precision), DecodeError> {
        if b.len() != 8 || b[4] != b'.' {
            return Err(DecodeError::InvalidLatitude(b.to_owned()));
        }

        let north = match b[7] {
            b'N' => true,
            b'S' => false,
            _ => return Err(DecodeError::InvalidLatitude(b.to_owned())),
        };

        // Some APRS lats have trailing spaces
        // This is used to convey ambiguity and is only valid in latitudes
        // Once we encounter a space, the remainder must be spaces
        let mut total_spaces = 0;
        let (deg, num_spaces) = parse_bytes_trailing_spaces(&[b[0], b[1]], false)
            .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?;
        total_spaces += num_spaces;
        let (min, num_spaces) = parse_bytes_trailing_spaces(&[b[2], b[3]], num_spaces > 0)
            .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?;
        total_spaces += num_spaces;
        let (min_frac, num_spaces) = parse_bytes_trailing_spaces(&[b[5], b[6]], num_spaces > 0)
            .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?;
        total_spaces += num_spaces;

        let precision = Precision::from_num_digits(total_spaces)
            .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?;

        let lat = Self::from_dmh(deg, min, min_frac, north)
            .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?;

        Ok((lat, precision))
    }

    pub(crate) fn parse_compressed(b: &[u8]) -> Result<Self, DecodeError> {
        let value = 90.0
            - (base91::decode_ascii(b)
                .ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))?
                / 380926.0);

        Self::new(value).ok_or_else(|| DecodeError::InvalidLatitude(b.to_owned()))
    }

    pub(crate) fn encode_compressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let value = (90.0 - self.0) * 380926.0;
        base91::encode_ascii(value, buf, 4)
    }

    pub(crate) fn encode_uncompressed<W: Write>(
        &self,
        buf: &mut W,
        precision: Precision,
    ) -> Result<(), EncodeError> {
        let (deg, min, min_frac, is_north) = self.dmh();
        let dir = if is_north { 'N' } else { 'S' };

        // zero out fields as required for precision
        // Ideally we would be doing some clever rounding here
        // E.g. if last 2 digits were blanked,
        // 4905.83 would become 4906.__
        let mut digit_buffer = [b' '; 6];
        let blank_index = 6 - precision.num_digits() as usize;

        // write will only fail if there isn't enough space
        // which is what we want (the remaining buffer should remain untouched)
        let _ = write!(
            &mut digit_buffer[..blank_index],
            "{:02}{:02}{:02}",
            deg,
            min,
            min_frac
        );
        buf.write_all(&digit_buffer[0..4])?;
        write!(buf, ".")?;
        buf.write_all(&digit_buffer[4..6])?;
        write!(buf, "{}", dir)?;
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
    /// Creates a new `Longitude`.
    /// Returns `None` if the given value is not a valid longitude
    pub fn new(value: f64) -> Option<Self> {
        if value > 180.0 || value < -180.0 || value.is_nan() {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Creates a new `Longitude` from degrees, minutes, and hundredths of a minute,
    /// as well as direction
    pub fn from_dmh(deg: u32, min: u32, hundredths: u32, east: bool) -> Option<Self> {
        let value = f64::from(deg) + f64::from(min) / 60. + f64::from(hundredths) / 6_000.;
        let value = if east { value } else { -value };

        Self::new(value)
    }

    /// Returns the `Longitude`'s degrees, minutes, hundredths of a minute, and direction.
    /// `true` is east, `false` is west.
    pub fn dmh(&self) -> (u32, u32, u32, bool) {
        let lon = self.0;

        let (dir, lon) = if lon >= 0.0 {
            (true, lon)
        } else {
            (false, -lon)
        };

        let mut deg = lon as u32;
        let mut min = ((lon - (deg as f64)) * 60.0) as u32;
        let mut hundredths = ((lon - (deg as f64) - (min as f64 / 60.0)) * 6000.0).round() as u32;

        if hundredths == 100 {
            // overflow from the rounding. need to propagate it up
            hundredths = 0;
            min += 1;
        }

        if min == 60 {
            min = 0;
            deg += 1;
        }

        (deg, min, hundredths, dir)
    }

    /// The value of the longitude.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Precision is needed so we know how many digits to ignore
    pub(crate) fn parse_uncompressed(b: &[u8], precision: Precision) -> Result<Self, DecodeError> {
        if b.len() != 9 || b[5] != b'.' {
            return Err(DecodeError::InvalidLongitude(b.to_owned()));
        }

        let east = match b[8] {
            b'E' => true,
            b'W' => false,
            _ => return Err(DecodeError::InvalidLongitude(b.to_owned())),
        };

        let mut digit_buffer = [0; 7];
        digit_buffer[0..5].copy_from_slice(&b[0..5]);
        digit_buffer[5..7].copy_from_slice(&b[6..8]);

        // zero out the digits we don't care about
        for i in (7 - precision.num_digits())..7 {
            digit_buffer[i as usize] = b'0';
        }

        let deg = parse_bytes::<u32>(&digit_buffer[0..3])
            .ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))?;
        let min = parse_bytes::<u32>(&digit_buffer[3..5])
            .ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))?;
        let min_frac = parse_bytes::<u32>(&digit_buffer[5..7])
            .ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))?;

        Self::from_dmh(deg, min, min_frac, east)
            .ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))
    }

    pub(crate) fn parse_compressed(b: &[u8]) -> Result<Self, DecodeError> {
        let value = (base91::decode_ascii(b)
            .ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))?
            / 190463.0)
            - 180.0;

        Self::new(value).ok_or_else(|| DecodeError::InvalidLongitude(b.to_owned()))
    }

    pub(crate) fn encode_compressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let value = (180.0 + self.0) * 190463.0;
        base91::encode_ascii(value, buf, 4)
    }

    pub(crate) fn encode_uncompressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        let (deg, min, min_frac, is_east) = self.dmh();
        let dir = if is_east { 'E' } else { 'W' };

        write!(buf, "{:03}{:02}.{:02}{}", deg, min, min_frac, dir)?;
        Ok(())
    }
}

// if only_spaces is true, requires that b is only spaces
// returns the parsed value as well as the number of spaces we found
fn parse_bytes_trailing_spaces(b: &[u8; 2], only_spaces: bool) -> Option<(u32, u8)> {
    if only_spaces {
        if b == b"  " {
            return Some((0, 2));
        } else {
            return None;
        }
    }
    match (b[0], b[1]) {
        (b' ', b' ') => Some((0, 2)),
        (_, b' ') => parse_bytes::<u32>(&b[0..1]).map(|v| (v * 10, 1)),
        (_, _) => parse_bytes::<u32>(&b[..]).map(|v| (v, 0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precision_e2e() {
        for i in 0..6 {
            assert_eq!(i, Precision::from_num_digits(i).unwrap().num_digits());
        }
    }

    #[test]
    fn test_latitude_out_of_bounds() {
        assert_eq!(None, Latitude::new(90.1));
        assert_eq!(None, Latitude::new(-90.1));
    }

    #[test]
    fn test_longitude_out_of_bounds() {
        assert_eq!(None, Latitude::new(180.1));
        assert_eq!(None, Latitude::new(-180.1));
    }

    #[test]
    fn test_parse_bytes_trailing_spaces() {
        assert_eq!(Some((12, 0)), parse_bytes_trailing_spaces(b"12", false));
        assert_eq!(Some((10, 1)), parse_bytes_trailing_spaces(b"1 ", false));
        assert_eq!(Some((0, 2)), parse_bytes_trailing_spaces(b"  ", false));

        assert_eq!(None, parse_bytes_trailing_spaces(b" 2", false));

        assert_eq!(None, parse_bytes_trailing_spaces(b"12", true));
        assert_eq!(None, parse_bytes_trailing_spaces(b"1 ", true));
        assert_eq!(None, parse_bytes_trailing_spaces(b" 1", true));
        assert_eq!(Some((0, 2)), parse_bytes_trailing_spaces(b"  ", true));
    }

    #[test]
    fn test_parse_uncompressed_latitude() {
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50N"[..]).unwrap(),
            (
                Latitude::new(49.05833333333333).unwrap(),
                Precision::HundredthMinute
            )
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50S"[..]).unwrap(),
            (
                Latitude::new(-49.05833333333333).unwrap(),
                Precision::HundredthMinute
            )
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.5 S"[..]).unwrap(),
            (
                Latitude::new(-49.05833333333333).unwrap(),
                Precision::TenthMinute
            )
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.  S"[..]).unwrap(),
            (Latitude::new(-49.05).unwrap(), Precision::OneMinute)
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"490 .  S"[..]).unwrap(),
            (Latitude::new(-49.0).unwrap(), Precision::TenMinute)
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4   .  S"[..]).unwrap(),
            (Latitude::new(-40.0).unwrap(), Precision::TenDegree)
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"    .  S"[..]),
            Err(DecodeError::InvalidLatitude(b"    .  S".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"49 3.50W"[..]),
            Err(DecodeError::InvalidLatitude(b"49 3.50W".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"490 .50W"[..]),
            Err(DecodeError::InvalidLatitude(b"490 .50W".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"49  . 0W"[..]),
            Err(DecodeError::InvalidLatitude(b"49  . 0W".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50W"[..]),
            Err(DecodeError::InvalidLatitude(b"4903.50W".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"4903.50E"[..]),
            Err(DecodeError::InvalidLatitude(b"4903.50E".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"9903.50N"[..]),
            Err(DecodeError::InvalidLatitude(b"9903.50N".to_vec()))
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"0000.00N"[..]).unwrap(),
            (Latitude::new(0.0).unwrap(), Precision::HundredthMinute)
        );
        assert_eq!(
            Latitude::parse_uncompressed(&b"0000.00S"[..]).unwrap(),
            (Latitude::new(0.0).unwrap(), Precision::HundredthMinute)
        );
    }

    #[test]
    fn test_parse_uncompressed_longitude() {
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"12903.50E"[..], Precision::default()).unwrap(),
            129.05833333333333
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"04903.50W"[..], Precision::default()).unwrap(),
            -49.05833333333333
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"04903.50N"[..], Precision::default()),
            Err(DecodeError::InvalidLongitude(b"04903.50N".to_vec()))
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"04903.50S"[..], Precision::default()),
            Err(DecodeError::InvalidLongitude(b"04903.50S".to_vec()))
        );
        assert_eq!(
            Longitude::parse_uncompressed(&b"18903.50E"[..], Precision::default()),
            Err(DecodeError::InvalidLongitude(b"18903.50E".to_vec()))
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.00E"[..], Precision::default()).unwrap(),
            0.0
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.00W"[..], Precision::default()).unwrap(),
            0.0
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.ZZW"[..], Precision::OneMinute).unwrap(),
            0.0
        );
        assert_relative_eq!(
            *Longitude::parse_uncompressed(&b"00000.98W"[..], Precision::OneMinute).unwrap(),
            0.0
        );
    }

    #[test]
    fn test_encode_uncompressed_latitude() {
        let mut buf = vec![];
        Latitude::new(49.05833)
            .unwrap()
            .encode_uncompressed(&mut buf, Precision::default())
            .unwrap();
        assert_eq!(buf, &b"4903.50N"[..]);

        let mut buf = vec![];
        Latitude::new(-49.05833)
            .unwrap()
            .encode_uncompressed(&mut buf, Precision::default())
            .unwrap();
        assert_eq!(buf, &b"4903.50S"[..]);

        let mut buf = vec![];
        Latitude::new(0.0)
            .unwrap()
            .encode_uncompressed(&mut buf, Precision::default())
            .unwrap();
        assert_eq!(buf, &b"0000.00N"[..]);

        let mut buf = vec![];
        Latitude::new(-49.05833)
            .unwrap()
            .encode_uncompressed(&mut buf, Precision::OneMinute)
            .unwrap();
        assert_eq!(buf, &b"4903.  S"[..]);
    }

    #[test]
    fn test_dmh_lat() {
        let lat = Latitude::new(11.99999999).unwrap();
        assert_eq!((12, 0, 0, true), lat.dmh());

        let lat = Latitude::new(-11.99999999).unwrap();
        assert_eq!((12, 0, 0, false), lat.dmh());

        let lat = Latitude::new(89.9999999).unwrap();
        assert_eq!((90, 0, 0, true), lat.dmh());
    }

    #[test]
    fn test_dmh_lon() {
        let lon = Longitude::new(33.9999999999).unwrap();
        assert_eq!((34, 0, 0, true), lon.dmh());

        let lon = Longitude::new(-33.9999999999).unwrap();
        assert_eq!((34, 0, 0, false), lon.dmh());

        let lon = Longitude::new(179.9999999).unwrap();
        assert_eq!((180, 0, 0, true), lon.dmh());
    }

    #[test]
    fn test_encode_uncompressed_longitude() {
        let mut buf = vec![];
        Longitude::new(129.05833)
            .unwrap()
            .encode_uncompressed(&mut buf)
            .unwrap();
        assert_eq!(buf, &b"12903.50E"[..]);

        let mut buf = vec![];
        Longitude::new(-49.0583)
            .unwrap()
            .encode_uncompressed(&mut buf)
            .unwrap();
        assert_eq!(buf, &b"04903.50W"[..]);

        let mut buf = vec![];
        Longitude(0.0).encode_uncompressed(&mut buf).unwrap();
        assert_eq!(buf, &b"00000.00E"[..]);
    }
}
