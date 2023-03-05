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
//! use aprs_parser::{AprsCst, AprsData, AprsPacket, AprsPosition, Callsign, Latitude, Longitude, Precision, Timestamp, Via, QConstruct};
//!
//! fn main() {
//!     let result = AprsPacket::decode_textual(
//!         br"ICA3D2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
//!     );
//!
//!     assert_eq!(
//!         result,
//!         Ok(
//!             AprsPacket {
//!                 from: Callsign::new_no_ssid("ICA3D2"),
//!                 via: vec![
//!                     Via::QConstruct(QConstruct::AS),
//!                     Via::Callsign(Callsign::new_no_ssid("dl4mea"), false),
//!                 ],
//!                 data: AprsData::Position(
//!                     AprsPosition {
//!                         to: Callsign::new_no_ssid("APRS"),
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
pub mod mic_e;
mod packet;
mod position;
mod status;
mod timestamp;
mod via;

pub use callsign::Callsign;
pub use compressed_cs::{AprsAltitude, AprsCompressedCs, AprsCourseSpeed, AprsRadioRange};
pub use compression_type::AprsCompressionType;
pub use error::{DecodeError, EncodeError};
pub use lonlat::{Latitude, Longitude};
pub use message::AprsMessage;
pub use mic_e::AprsMicE;
pub use packet::{AprsData, AprsPacket};
pub use position::{AprsCst, AprsPosition, Precision};
pub use status::AprsStatus;
pub use timestamp::{DhmTimestamp, Timestamp};
pub use via::{QConstruct, Via};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overall() {
        let original =
            &b"ICA3D2>Aprs,qAO,dl4mea::DEST     :Hello World! This msg has a : colon {3a2B975"[..];

        let mut buf = vec![];
        AprsPacket::decode_textual(original)
            .unwrap()
            .encode_textual(&mut buf)
            .unwrap();
        assert_eq!(original, &buf);
    }
}
