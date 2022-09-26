use std::io::Write;

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
        let c_lwr = c - 33;
        let s_lwr = s - 33;

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
                buf.write_all(&[c + 33, s + 33])?;
            }
            AprsCompressedCs::RadioRange(rr) => {
                let s = rr.to_s();
                buf.write_all(&[b'{', s + 33])?;
            }
            AprsCompressedCs::Altitude(a) => {
                if t.nmea_source != NmeaSource::Gga {
                    return Err(EncodeError::NonGgaAltitude);
                }

                let (c, s) = a.to_cs();
                buf.write_all(&[c + 33, s + 33])?;
            }
        }

        buf.write_all(&[u8::from(t) + 33])?;

        Ok(())
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct AprsCourseSpeed {
    pub course_degrees: i32,
    pub speed_knots: f64,
}

impl AprsCourseSpeed {
    fn from_cs(c: u8, s: u8) -> Self {
        Self {
            course_degrees: c as i32 * 4,
            speed_knots: ((1.08_f64).powi(s as i32) - 1.0) as f64,
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
    pub range_miles: f64,
}

impl AprsRadioRange {
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
    pub altitude_feet: f64,
}

impl AprsAltitude {
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
        for c in 0..=255 {
            for s in 0..=255 {
                let val = AprsCourseSpeed::from_cs(c, s);

                assert_eq!((c, s), val.to_cs());
            }
        }
    }

    #[test]
    fn radio_range_exhaustive() {
        for s in 0..=255 {
            let val = AprsRadioRange::from_s(s);

            assert_eq!(s, val.to_s());
        }
    }

    #[test]
    fn altitude_exhaustive() {
        for c in 0..=90 {
            for s in 0..=90 {
                let val = AprsAltitude::from_cs(c, s);

                assert_eq!((c, s), val.to_cs());
            }
        }
    }
}
