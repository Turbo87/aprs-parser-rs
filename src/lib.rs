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
//!         r"ICA3D17F2>APRS,qAS,dl4mea:/074849h4821.61N\01224.49E^322/103/A=003054"
//!     );
//!
//!     println!("{:#?}", result);
//!
//!     // Ok(
//!     //     AprsMessage {
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
//!     //                 latitude: 48.360165,
//!     //                 longitude: 12.408166,
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
extern crate lazy_static;

mod callsign;
mod error;
mod lonlat;
mod message;
mod position;
mod symbol;
mod timestamp;

use std::str::FromStr;

pub use callsign::Callsign;
pub use error::AprsError;
pub use lonlat::{Latitude, Longitude};
pub use message::{AprsData, AprsMessage};
pub use position::AprsPosition;
pub use symbol::Symbol;
pub use timestamp::Timestamp;

pub fn parse(s: &str) -> Result<AprsMessage, AprsError> {
    AprsMessage::from_str(s)
}
