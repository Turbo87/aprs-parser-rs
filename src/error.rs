#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum AprsError {
    #[error("Empty Callsign: {0}")]
    EmptyCallsign(String),
    #[error("Empty Callsign SSID: {0}")]
    EmptySSID(String),
    #[error("Invalid Timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Unsupported Position Format: {0}")]
    UnsupportedPositionFormat(String),
    #[error("Invalid Position: {0}")]
    InvalidPosition(String),
    #[error("Invalid Latitude: {0}")]
    InvalidLatitude(String),
    #[error("Invalid Longitude: {0}")]
    InvalidLongitude(String),
    #[error("Invalid Symbol Table: {0}")]
    InvalidSymbolTable(String),
    #[error("Invalid Message: {0}")]
    InvalidMessage(String),
}
