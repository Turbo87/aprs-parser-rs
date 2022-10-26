use std::io::Write;

use base91;
use compression_type::NmeaSource;
use AprsCompressionType;
use AprsError;
use EncodeError;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum AprsCompressedCs {
    CourseSpeed(AprsCourseSpeed),
    RadioRange(AprsRadioRange),
    Altitude(AprsAltitude),
}

impl AprsCompressedCs {
    pub(crate) fn parse(c: u8, s: u8, t: AprsCompressionType) -> Result<Self, AprsError> {
        let c_lwr = base91::digit_from_ascii(c).ok_or(AprsError::InvalidCs([c, s]))?;
        let s_lwr = base91::digit_from_ascii(s).ok_or(AprsError::InvalidCs([c, s]))?;

        if t.nmea_source == NmeaSource::Gga {
            Ok(AprsCompressedCs::Altitude(AprsAltitude::from_cs(
                c_lwr, s_lwr,
            )))
        } else {
            let val = match c_lwr {
                0..=89 => AprsCompressedCs::CourseSpeed(AprsCourseSpeed::from_cs(c_lwr, s_lwr)),
                90 => AprsCompressedCs::RadioRange(AprsRadioRange::from_s(s_lwr)),
                _ => return Err(AprsError::InvalidCs([c, s])),
            };

            Ok(val)
        }
    }

    pub(crate) fn encode<W: Write>(
        self,
        buf: &mut W,
        t: AprsCompressionType,
    ) -> Result<(), EncodeError> {
        match self {
            AprsCompressedCs::CourseSpeed(cs) => {
                let (c, s) = cs.to_cs();
                buf.write_all(&[base91::digit_to_ascii(c), base91::digit_to_ascii(s)])?;
            }
            AprsCompressedCs::RadioRange(rr) => {
                let s = rr.to_s();
                buf.write_all(&[b'{', base91::digit_to_ascii(s)])?;
            }
            AprsCompressedCs::Altitude(a) => {
                if t.nmea_source != NmeaSource::Gga {
                    return Err(EncodeError::NonGgaAltitude);
                }

                let (c, s) = a.to_cs();
                buf.write_all(&[base91::digit_to_ascii(c), base91::digit_to_ascii(s)])?;
            }
        }

        buf.write_all(&[base91::digit_to_ascii(u8::from(t))])?;

        Ok(())
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct AprsCourseSpeed {
    course_degrees: u16,
    speed_knots: f64,
}

impl AprsCourseSpeed {
    pub fn new(course_degrees: u16, speed_knots: f64) -> Self {
        assert!(course_degrees <= 360);
        assert!(speed_knots < (1.08_f64).powi(255));

        Self {
            course_degrees,
            speed_knots,
        }
    }

    pub fn course_degrees(&self) -> u16 {
        self.course_degrees
    }

    pub fn speed_knots(&self) -> f64 {
        self.speed_knots
    }

    fn from_cs(c: u8, s: u8) -> Self {
        let course_degrees = c as u16 * 4;
        let speed_knots = ((1.08_f64).powi(s as i32) - 1.0) as f64;

        debug_assert!(course_degrees <= 360);
        debug_assert!(speed_knots < (1.08_f64).powi(255));

        Self {
            course_degrees,
            speed_knots,
        }
    }

    fn to_cs(self) -> (u8, u8) {
        let c = self.course_degrees / 4;
        let s = ((self.speed_knots + 1.0).ln() / (1.08_f64).ln()).round();

        (c as u8, s as u8)
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct AprsRadioRange {
    range_miles: f64,
}

impl AprsRadioRange {
    pub fn new(range_miles: f64) -> Self {
        assert!(range_miles < (1.08_f64).powi(255));

        Self { range_miles }
    }

    pub fn range_miles(&self) -> f64 {
        self.range_miles
    }

    fn from_s(s: u8) -> Self {
        Self {
            range_miles: (2.0 * (1.08_f64).powi(s as i32)) as f64,
        }
    }

    fn to_s(self) -> u8 {
        let s = ((self.range_miles / 2.0).ln() / (1.08_f64).ln()).round();

        s as u8
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct AprsAltitude {
    altitude_feet: f64,
}

impl AprsAltitude {
    pub fn new(altitude_feet: f64) -> Self {
        assert!(altitude_feet < (1.002_f64).powi(255 * 91 + 255));

        Self { altitude_feet }
    }

    pub fn altitude_feet(&self) -> f64 {
        self.altitude_feet
    }

    fn from_cs(c: u8, s: u8) -> Self {
        Self {
            altitude_feet: ((1.002_f64).powi(c as i32 * 91 + s as i32)) as f64,
        }
    }

    fn to_cs(self) -> (u8, u8) {
        let alt = (self.altitude_feet.ln() / (1.002_f64).ln()).round() as i32;

        let c = alt / 91;
        let s = alt % 91;

        (c as u8, s as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_speed_exhaustive() {
        for c in 0..91 {
            for s in 0..91 {
                let val = AprsCourseSpeed::from_cs(c, s);

                assert_eq!((c, s), val.to_cs());
            }
        }
    }

    #[test]
    fn radio_range_exhaustive() {
        for s in 0..91 {
            let val = AprsRadioRange::from_s(s);

            assert_eq!(s, val.to_s());
        }
    }

    #[test]
    fn altitude_exhaustive() {
        for c in 0..91 {
            for s in 0..91 {
                let val = AprsAltitude::from_cs(c, s);

                assert_eq!((c, s), val.to_cs());
            }
        }
    }
}
