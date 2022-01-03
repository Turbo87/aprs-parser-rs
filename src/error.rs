#[derive(Debug, Fail, Eq, PartialEq)]
pub enum APRSError {
    #[fail(display = "Empty Callsign: {}", _0)]
    EmptyCallsign(String),
    #[fail(display = "Empty Callsign SSID: {}", _0)]
    EmptySSID(String),
    #[fail(display = "Invalid Timestamp: {}", _0)]
    InvalidTimestamp(String),
    #[fail(display = "Unsupported Position Format: {}", _0)]
    UnsupportedPositionFormat(String),
    #[fail(display = "Invalid Position: {}", _0)]
    InvalidPosition(String),
    #[fail(display = "Invalid Latitude: {}", _0)]
    InvalidLatitude(String),
    #[fail(display = "Invalid Longitude: {}", _0)]
    InvalidLongitude(String),
    #[fail(display = "Invalid Message: {}", _0)]
    InvalidMessage(String),
}
