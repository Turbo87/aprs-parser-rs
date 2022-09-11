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
//! fn main() {
//!     let result = aprs_parser::parse(
//!         br"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
//!     );
//!
//!     println!("{:#?}", result);
//!
//!     // Ok(
//!     //     AprsPacket {
//!     //         from: Callsign {
//!     //             call: "ICA3D17F2",
//!     //             ssid: None
//!     //         },
//!     //         to: Callsign {
//!     //             call: "APRS",
//!     //             ssid: None
//!     //         },
//!     //         via: [
//!     //             Callsign {
//!     //                 call: "qAS",
//!     //                 ssid: None
//!     //             },
//!     //             Callsign {
//!     //                 call: "dl4mea",
//!     //                 ssid: None
//!     //             }
//!     //         ],
//!     //         data: Position(
//!     //             AprsPosition {
//!     //                 timestamp: Some(
//!     //                     HHMMSS(
//!     //                         7,
//!     //                         48,
//!     //                         49
//!     //                     )
//!     //                 ),
//!     //                 messaging_supported: false,
//!     //                 latitude: Latitude(48.360165),
//!     //                 longitude: Longitude(12.408166),
//!     //                 symbol_table: '\\',
//!     //                 symbol_code: '^',
//!     //                 comment: "322/103/A=003054"
//!     //             }
//!     //         )
//!     //     }
//!     // )
//! }
//! ```

// `!(-90. ..=90.).contains(&value)` seems worse than `value > 90. || value < -90.`
#![allow(clippy::manual_range_contains)]

extern crate thiserror;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod bytes;
mod callsign;
mod error;
mod lonlat;
mod message;
mod packet;
mod position;
mod timestamp;

use std::convert::TryFrom;

pub use callsign::Callsign;
pub use error::{AprsError, EncodeError};
pub use lonlat::{Latitude, Longitude};
pub use message::AprsMessage;
pub use packet::{AprsData, AprsPacket};
pub use position::AprsPosition;
pub use timestamp::Timestamp;

pub fn parse(b: &[u8]) -> Result<AprsPacket, AprsError> {
    AprsPacket::try_from(b)
}
