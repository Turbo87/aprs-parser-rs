use std::{
    io::{Read, Write},
    ops::RangeInclusive,
};

use crate::{AprsCompressedCs, AprsCompressionType, DecodeError, EncodeError};

use super::lonlat::{Latitude, Longitude, Precision};

#[derive(PartialEq, Debug, Clone)]
pub enum AprsCst {
    CompressedSome {
        cs: AprsCompressedCs,
        t: AprsCompressionType,
    },
    CompressedNone,
    Uncompressed,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Position {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub precision: Precision,
    pub symbol_table: char,
    pub symbol_code: char,
    pub cst: AprsCst,
}

impl Position {
    /// Latitudes in APRS aren't perfectly precise - they have a configurable level of ambiguity. This is stored in the `precision` field on the `Position` struct. This method returns a range of what the actual latitude value might be.
    pub fn latitude_bounding(&self) -> RangeInclusive<f64> {
        self.precision.range(self.latitude.value())
    }

    /// Longitudes in APRS aren't perfectly precise - they have a configurable level of ambiguity. This is stored in the `precision` field on the `Position` struct. This method returns a range of what the actual longitude value might be.
    pub fn longitude_bounding(&self) -> RangeInclusive<f64> {
        self.precision.range(self.longitude.value())
    }

    pub(crate) fn encode_uncompressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        self.latitude.encode_uncompressed(buf, self.precision)?;
        write!(buf, "{}", self.symbol_table)?;
        self.longitude.encode_uncompressed(buf)?;
        write!(buf, "{}", self.symbol_code)?;
        Ok(())
    }

    pub(crate) fn encode_compressed<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        write!(buf, "{}", self.symbol_table)?;

        self.latitude.encode_compressed(buf)?;
        self.longitude.encode_compressed(buf)?;

        write!(buf, "{}", self.symbol_code)?;

        match self.cst {
            AprsCst::CompressedSome { cs, t } => cs.encode(buf, t)?,
            AprsCst::CompressedNone => write!(buf, " sT")?,
            AprsCst::Uncompressed => unreachable!(),
        };

        Ok(())
    }
    /// this function assumes we are getting the head of a byte list
    /// representing a compressed or uncompressed position
    ///
    /// all position representations interleave the symbol table and code
    /// so we stuff it all in here
    pub(crate) fn decode(b: &[u8]) -> Result<Self, DecodeError> {
        let is_uncompressed_position = (*b.first().unwrap_or(&0) as char).is_numeric();
        if is_uncompressed_position {
            let (latitude, precision) = Latitude::parse_uncompressed(&b[0..8])?;
            let longitude = Longitude::parse_uncompressed(&b[9..18], precision)?;

            let symbol_table = b[8] as char;
            let symbol_code = b[18] as char;

            Ok(Self {
                latitude,
                longitude,
                precision,
                symbol_code,
                symbol_table,
                cst: AprsCst::Uncompressed,
            })
        } else {
            let symbol_table = b[0] as char;
            let comp_lat = &b[1..5];
            let comp_lon = &b[5..9];
            let symbol_code = b[9] as char;
            let course_speed = &b[10..12];
            let comp_type = b[12];

            b.take(12);

            let latitude = Latitude::parse_compressed(comp_lat)?;
            let longitude = Longitude::parse_compressed(comp_lon)?;

            // From the APRS spec - if the c value is a space,
            // the csT doesn't matter
            let cst = match course_speed[0] {
                b' ' => AprsCst::CompressedNone,
                _ => {
                    let t = comp_type
                        .checked_sub(33)
                        .ok_or_else(|| DecodeError::InvalidPosition(b.to_owned()))?
                        .into();
                    let cs = AprsCompressedCs::parse(course_speed[0], course_speed[1], t)?;
                    AprsCst::CompressedSome { cs, t }
                }
            };
            Ok(Self {
                latitude,
                longitude,
                precision: Precision::default(),
                symbol_code,
                symbol_table,
                cst,
            })
        }
    }
}
