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
//!     //     APRSMessage {
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

extern crate thiserror;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod callsign;
mod error;
mod lonlat;
mod message;
mod position;
mod timestamp;

use std::str::FromStr;

pub use callsign::Callsign;
pub use error::AprsError;
pub use message::{APRSMessage, AprsData};
pub use position::AprsPosition;
pub use timestamp::Timestamp;

pub fn parse(s: &str) -> Result<APRSMessage, AprsError> {
    APRSMessage::from_str(s)
}
