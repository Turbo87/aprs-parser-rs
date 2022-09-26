#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum GpsFix {
    Old,
    Current,
}

impl From<bool> for GpsFix {
    fn from(bit: bool) -> Self {
        match bit {
            false => GpsFix::Old,
            true => GpsFix::Current,
        }
    }
}

impl From<GpsFix> for bool {
    fn from(fix: GpsFix) -> bool {
        match fix {
            GpsFix::Old => false,
            GpsFix::Current => true,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum NmeaSource {
    Other,
    Gll,
    Gga,
    Rmc,
}

impl From<(bool, bool)> for NmeaSource {
    fn from(bits: (bool, bool)) -> Self {
        match bits {
            (false, false) => NmeaSource::Other,
            (false, true) => NmeaSource::Gll,
            (true, false) => NmeaSource::Gga,
            (true, true) => NmeaSource::Rmc,
        }
    }
}

impl From<NmeaSource> for (bool, bool) {
    fn from(source: NmeaSource) -> Self {
        match source {
            NmeaSource::Other => (false, false),
            NmeaSource::Gll => (false, true),
            NmeaSource::Gga => (true, false),
            NmeaSource::Rmc => (true, true),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Origin {
    Compressed,
    TncBText,
    Software,
    Tbd,
    Kpc3,
    Pico,
    Other,
    Digipeater,
}

impl From<(bool, bool, bool)> for Origin {
    fn from(bits: (bool, bool, bool)) -> Self {
        match bits {
            (false, false, false) => Origin::Compressed,
            (false, false, true) => Origin::TncBText,
            (false, true, false) => Origin::Software,
            (false, true, true) => Origin::Tbd,
            (true, false, false) => Origin::Kpc3,
            (true, false, true) => Origin::Pico,
            (true, true, false) => Origin::Other,
            (true, true, true) => Origin::Digipeater,
        }
    }
}

impl From<Origin> for (bool, bool, bool) {
    fn from(origin: Origin) -> Self {
        match origin {
            Origin::Compressed => (false, false, false),
            Origin::TncBText => (false, false, true),
            Origin::Software => (false, true, false),
            Origin::Tbd => (false, true, true),
            Origin::Kpc3 => (true, false, false),
            Origin::Pico => (true, false, true),
            Origin::Other => (true, true, false),
            Origin::Digipeater => (true, true, true),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct AprsCompressionType {
    pub gps_fix: GpsFix,
    pub nmea_source: NmeaSource,
    pub origin: Origin,
}

impl From<u8> for AprsCompressionType {
    fn from(byte: u8) -> Self {
        let gps_fix = (byte & (1 << 5)) != 0;
        let nmea_source = (byte & (1 << 4) != 0, byte & (1 << 3) != 0);
        let origin = (
            byte & (1 << 2) != 0,
            byte & (1 << 1) != 0,
            byte & (1 << 0) != 0,
        );

        Self {
            gps_fix: gps_fix.into(),
            nmea_source: nmea_source.into(),
            origin: origin.into(),
        }
    }
}

impl From<AprsCompressionType> for u8 {
    fn from(t: AprsCompressionType) -> u8 {
        let b5: bool = t.gps_fix.into();
        let (b4, b3) = t.nmea_source.into();
        let (b2, b1, b0) = t.origin.into();

        (u8::from(b5) << 5)
            + (u8::from(b4) << 4)
            + (u8::from(b3) << 3)
            + (u8::from(b2) << 2)
            + (u8::from(b1) << 1)
            + (u8::from(b0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let byte: u8 = 0b00111010;
        let expected = AprsCompressionType {
            gps_fix: GpsFix::Current,
            nmea_source: NmeaSource::Rmc,
            origin: Origin::Software,
        };

        assert_eq!(expected, byte.into());
    }

    #[test]
    fn decode_works() {
        let expected: u8 = 0b00111010;
        let ctype = AprsCompressionType {
            gps_fix: GpsFix::Current,
            nmea_source: NmeaSource::Rmc,
            origin: Origin::Software,
        };

        assert_eq!(expected, ctype.into());
    }
}
