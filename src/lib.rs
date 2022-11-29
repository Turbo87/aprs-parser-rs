//! [APRS] message parser for [Rust]
//!
//! [APRS]: http://www.aprs.org/
//! [Rust]: https://www.rust-lang.org/
//!
//! # Usage
//!
//! ```rust
//! extern crate aprs_parser;
//!
//! use aprs_parser::{AprsCst, AprsData, AprsPacket, AprsPosition, Callsign, Latitude, Longitude, Precision, Timestamp};
//!
//! fn main() {
//!     let result = aprs_parser::parse(
//!         br"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
//!     );
//!
//!     assert_eq!(
//!         result,
//!         Ok(
//!             AprsPacket {
//!                 from: Callsign {
//!                     call: "ICA3D17F2".to_string(),
//!                     ssid: None
//!                 },
//!                 to: Callsign {
//!                     call: "APRS".to_string(),
//!                     ssid: None
//!                 },
//!                 via: vec![
//!                     Callsign {
//!                         call: "qAS".to_string(),
//!                         ssid: None
//!                     },
//!                     Callsign {
//!                         call: "dl4mea".to_string(),
//!                         ssid: None
//!                     }
//!                 ],
//!                 data: AprsData::Position(
//!                     AprsPosition {
//!                         timestamp: Some(
//!                             Timestamp::HHMMSS(
//!                                 7,
//!                                 48,
//!                                 49
//!                             )
//!                         ),
//!                         messaging_supported: false,
//!                         latitude: Latitude::new(48.36016666666667).unwrap(),
//!                         longitude: Longitude::new(12.408166666666666).unwrap(),
//!                         precision: Precision::HundredthMinute,
//!                         symbol_table: '\\',
//!                         symbol_code: '^',
//!                         comment: b"322/103/A=003054".to_vec(),
//!                         cst: AprsCst::Uncompressed,
//!                     }
//!                 )
//!             }
//!        )
//!    );
//! }
//! ```

// `!(-90. ..=90.).contains(&value)` seems worse than `value > 90. || value < -90.`
#![allow(clippy::manual_range_contains)]

extern crate thiserror;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod base91;
mod bytes;
mod callsign;
mod compressed_cs;
mod compression_type;
mod error;
mod lonlat;
mod message;
mod packet;
mod position;
mod timestamp;

use std::convert::TryFrom;

pub use callsign::Callsign;
pub use compressed_cs::{AprsAltitude, AprsCompressedCs, AprsCourseSpeed, AprsRadioRange};
pub use compression_type::AprsCompressionType;
pub use error::{AprsError, EncodeError};
pub use lonlat::{Latitude, Longitude};
pub use message::AprsMessage;
pub use packet::{AprsData, AprsPacket};
pub use position::{AprsCst, AprsPosition, Precision};
pub use timestamp::Timestamp;

pub fn parse(b: &[u8]) -> Result<AprsPacket, AprsError> {
    AprsPacket::try_from(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overall() {
        let original =
            &b"ICA3D17F2>Aprs,qAS,dl4mea::DEST     :Hello World! This msg has a : colon {3a2B975"[..];

        let mut buf = vec![];
        parse(original).unwrap().encode(&mut buf).unwrap();
        assert_eq!(original, &buf);
    }
}
