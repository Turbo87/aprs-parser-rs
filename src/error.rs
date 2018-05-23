#[derive(Debug, Fail, Eq, PartialEq)]
pub enum APRSError {
    #[fail(display = "Empty Callsign: {}", _0)]
    EmptyCallsign(String),
    #[fail(display = "Empty Callsign SSID: {}", _0)]
    EmptySSID(String),
}