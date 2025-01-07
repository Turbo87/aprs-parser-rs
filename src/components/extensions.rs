use std::{convert::TryFrom, io::Write};

use crate::{bytes::parse_bytes, DecodeError, EncodeError};
#[derive(Clone, Debug, PartialEq)]

pub enum Directivity {
    Omni,
    DirectionDegrees(u16),
}

impl TryFrom<u8> for Directivity {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 10 {
            return match value {
                0 => Ok(Self::Omni),
                others => Ok(Self::DirectionDegrees(45 * others as u16)),
            };
        }
        Err(())
    }
}

impl From<Directivity> for u8 {
    fn from(val: Directivity) -> Self {
        match val {
            Directivity::Omni => 0,
            Directivity::DirectionDegrees(deg) => {
                let code = (deg % 360) / 45; // wrap around in case someone did a funny thing

                code as u8
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq)]

pub enum Extension {
    // this is a single defn for both
    // course/speed and direction/speed
    // because the only differentiating factor is
    // the type of packet it comes in on...
    DirectionSpeed {
        direction_degrees: u16,
        speed_knots: u16,
    },
    PowerHeightGainDirectivity {
        power_watts: u16,
        antenna_height_feet: u16,
        antenna_gain_db: u8,
        antenna_directivity: Directivity,
    },
    RadioRange {
        radio_range_miles: u16,
    },
    DFStrengthHeightGainDirectivity {
        s_points: u8,
        antenna_height_feet: u16,
        antenna_gain_db: u8,
        antenna_directivity: Directivity,
    },
    AreaObjectDescriptor {
        r#type: u8,
        color: u8,
    },
}

impl Extension {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        match self {
            Extension::DirectionSpeed {
                direction_degrees,
                speed_knots,
            } => {
                write!(buf, "{:3}/{:3}", direction_degrees, speed_knots)?;
            }
            Extension::PowerHeightGainDirectivity {
                power_watts,
                antenna_height_feet,
                antenna_gain_db,
                antenna_directivity,
            } => {
                let power_value = { ((*power_watts as f64).sqrt() as u8).to_string() };

                let height_value = {
                    ((((antenna_height_feet / 10) as f64).log2()) as u8)
                        .checked_add(48)
                        .ok_or_else(|| EncodeError::InvalidExtension(self.clone()))?
                } as char; // sqrt(H/10)

                let gain_value = if (0..9).contains(antenna_gain_db) {
                    Ok(antenna_gain_db.to_string())
                } else {
                    Err(EncodeError::InvalidExtension(self.clone()))
                }?;

                let directivitity = u8::from(antenna_directivity.to_owned()).to_string();

                write!(
                    buf,
                    "PHG{power_value}{height_value}{gain_value}{directivitity}"
                )?;
            }
            Extension::RadioRange { radio_range_miles } => {
                write!(buf, "RNG{:4}", radio_range_miles)?;
            }
            Extension::DFStrengthHeightGainDirectivity {
                s_points,
                antenna_height_feet,
                antenna_gain_db,
                antenna_directivity,
            } => {
                let s_value = if (0..9).contains(s_points) {
                    Ok(s_points.to_string())
                } else {
                    Err(EncodeError::InvalidExtension(self.clone()))
                }?;

                let height_value = {
                    ((((antenna_height_feet / 10) as f64).log2()) as u8)
                        .checked_add(48)
                        .ok_or_else(|| EncodeError::InvalidExtension(self.clone()))?
                } as char; // sqrt(H/10)

                let gain_value = if (0..9).contains(antenna_gain_db) {
                    Ok(antenna_gain_db.to_string())
                } else {
                    Err(EncodeError::InvalidExtension(self.clone()))
                }?;

                let directivitity = u8::from(antenna_directivity.to_owned()).to_string();

                write!(buf, "DFS{s_value}{height_value}{gain_value}{directivitity}")?;
            }
            Extension::AreaObjectDescriptor { r#type, color } => {
                write!(buf, "T{:2}/C{:2}", r#type, color)?;
            }
        }

        Ok(())
    }

    pub fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        let bytes = b
            .get(..7)
            .ok_or_else(|| DecodeError::InvalidExtension(b.to_vec()))?;

        const AREA_TYPE_T: u8 = b'T';
        match &bytes[..3] {
            b"RNG" => Ok(Self::RadioRange {
                radio_range_miles: parse_bytes(&bytes[3..])
                    .ok_or_else(|| DecodeError::InvalidExtensionRange(b.to_vec()))?,
            }),
            b"PHG" => {
                let power_code = (bytes[3] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionPhg(b.to_vec()))?;

                /*
                The height code may in fact be any ASCII character 0–9 and above. This is
                so that larger heights for balloons, aircraft or satellites may be specified.
                For example:
                : is the height code for 10240 feet (approximately 1.9 miles).
                ; is the height code for 20480 feet (approximately 3.9 miles), and so on.
                                */
                let height_code = bytes[4]
                    .checked_sub(48)
                    .ok_or_else(|| DecodeError::InvalidExtensionPhg(b.to_vec()))?;

                let gain_code = (bytes[5] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionPhg(b.to_vec()))?;
                let directivity_code = (bytes[6] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionPhg(b.to_vec()))?;

                let directivtity = Directivity::try_from(directivity_code as u8)
                    .map_err(|_| DecodeError::InvalidExtensionPhg(b.to_vec()))?;

                Ok(Self::PowerHeightGainDirectivity {
                    power_watts: power_code.pow(2) as u16,
                    antenna_height_feet: 2u16.pow(height_code as u32) * 10,
                    antenna_gain_db: gain_code as u8,
                    antenna_directivity: directivtity,
                })
            }
            b"DFS" => {
                let s_code = (bytes[3] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionDfs(b.to_vec()))?;

                /*
                The height code may in fact be any ASCII character 0–9 and above. This is
                so that larger heights for balloons, aircraft or satellites may be specified.
                For example:
                : is the height code for 10240 feet (approximately 1.9 miles).
                ; is the height code for 20480 feet (approximately 3.9 miles), and so on.
                                */
                let height_code = bytes[4]
                    .checked_sub(48)
                    .ok_or_else(|| DecodeError::InvalidExtensionDfs(b.to_vec()))?;

                let gain_code = (bytes[5] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionDfs(b.to_vec()))?;
                let directivity_code = (bytes[6] as char)
                    .to_digit(10)
                    .ok_or_else(|| DecodeError::InvalidExtensionDfs(b.to_vec()))?;

                let directivtity = Directivity::try_from(directivity_code as u8)
                    .map_err(|_| DecodeError::InvalidExtensionDfs(b.to_vec()))?;

                Ok(Self::DFStrengthHeightGainDirectivity {
                    s_points: s_code as u8,
                    antenna_height_feet: 2u16.pow(height_code as u32) * 10,
                    antenna_gain_db: gain_code as u8,
                    antenna_directivity: directivtity,
                })
            }
            // 'Txx'
            &[AREA_TYPE_T, _, _] => Ok(Self::AreaObjectDescriptor {
                r#type: parse_bytes(&bytes[1..3])
                    .ok_or_else(|| DecodeError::InvalidExtensionArea(b.to_vec()))?,
                color: parse_bytes(&bytes[5..])
                    .ok_or_else(|| DecodeError::InvalidExtensionArea(b.to_vec()))?,
            }),
            // course speed
            _ => {
                if b[3] == b'/' {
                    Ok(Self::DirectionSpeed {
                        direction_degrees: parse_bytes(&bytes[..3]).ok_or_else(|| {
                            DecodeError::InvalidExtensionDirectionSpeed(b.to_vec())
                        })?,
                        speed_knots: parse_bytes(&bytes[5..]).ok_or_else(|| {
                            DecodeError::InvalidExtensionDirectionSpeed(b.to_vec())
                        })?,
                    })
                } else {
                    Err(DecodeError::InvalidExtensionDirectionSpeed(b.to_vec()))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_course_speed() {
        let cse_speed = b"080/043";

        let ext = Extension::decode(cse_speed).unwrap();

        assert!(matches!(
            ext,
            Extension::DirectionSpeed {
                direction_degrees: 80,
                speed_knots: 43
            }
        ));
    }

    #[test]
    fn test_parse_phg() {
        let cse_speed = b"PHG5132";

        let ext = Extension::decode(cse_speed).unwrap();

        assert!(matches!(
            ext,
            Extension::PowerHeightGainDirectivity {
                power_watts: 25,
                antenna_height_feet: 20,
                antenna_gain_db: 3,
                antenna_directivity: Directivity::DirectionDegrees(90)
            }
        ));
        let mut buf = Vec::new();

        ext.encode(&mut buf).unwrap();

        assert_eq!(buf, cse_speed);
    }

    #[test]
    fn test_parse_phg_extended_height() {
        let cse_speed = b"PHG5;32";

        let ext = Extension::decode(cse_speed).unwrap();

        assert!(matches!(
            ext,
            Extension::PowerHeightGainDirectivity {
                power_watts: 25,
                antenna_height_feet: 20480,
                antenna_gain_db: 3,
                antenna_directivity: Directivity::DirectionDegrees(90)
            }
        ));

        let mut buf = Vec::new();

        ext.encode(&mut buf).unwrap();

        assert_eq!(buf, cse_speed);
    }

    #[test]
    fn test_parse_rng() {
        let cse_speed = b"RNG2345";

        let ext = Extension::decode(cse_speed).unwrap();

        assert!(matches!(
            ext,
            Extension::RadioRange {
                radio_range_miles: 2345
            }
        ));

        let mut buf = Vec::new();

        ext.encode(&mut buf).unwrap();

        assert_eq!(buf, cse_speed);
    }

    #[test]
    fn test_parse_bad_phg() {
        let ext = Extension::PowerHeightGainDirectivity {
            power_watts: 1000,
            antenna_height_feet: 0,
            antenna_gain_db: 255,
            antenna_directivity: Directivity::Omni,
        };
        let mut buf = Vec::new();

        assert!(ext.encode(&mut buf).is_err())
    }
}
