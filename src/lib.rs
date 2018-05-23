extern crate failure;
#[macro_use] extern crate failure_derive;

mod error;
mod callsign;

pub use error::APRSError;
pub use callsign::Callsign;
