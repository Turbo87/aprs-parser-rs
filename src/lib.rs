extern crate failure;
#[macro_use] extern crate failure_derive;

#[cfg(test)] #[macro_use] extern crate approx;

mod error;
mod callsign;
mod lonlat;
mod message;
mod position;
mod timestamp;

use std::str::FromStr;

pub use error::APRSError;
pub use callsign::Callsign;
pub use message::APRSMessage;
pub use position::APRSPosition;
pub use timestamp::Timestamp;

pub fn parse(s: &str) -> Result<APRSMessage, APRSError> {
    APRSMessage::from_str(s)
}
