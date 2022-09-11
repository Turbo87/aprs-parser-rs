#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum AprsError {
    #[error("Non-UTF8 Callsign: {0:?}")]
    NonUtf8Callsign(Vec<u8>),
    #[error("Empty Callsign: {0:?}")]
    EmptyCallsign(String),
    #[error("Empty Callsign SSID: {0:?}")]
    EmptySSID(String),
    #[error("Invalid Timestamp: {0:?}")]
    InvalidTimestamp(Vec<u8>),
    #[error("Unsupported Position Format: {0:?}")]
    UnsupportedPositionFormat(Vec<u8>),
    #[error("Invalid Position: {0:?}")]
    InvalidPosition(Vec<u8>),
    #[error("Invalid Latitude: {0:?}")]
    InvalidLatitude(Vec<u8>),
    #[error("Invalid Longitude: {0:?}")]
    InvalidLongitude(Vec<u8>),
    #[error("Invalid Packet: {0:?}")]
    InvalidPacket(Vec<u8>),
    #[error("Invalid Message Destination: {0:?}")]
    InvalidMessageDestination(Vec<u8>),
    #[error("Invalid Message ID: {0:?}")]
    InvalidMessageId(Vec<u8>),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("Invalid Latitude: {0}")]
    InvalidLatitude(f32),
    #[error("Invalid Longitude: {0}")]
    InvalidLongitude(f32),
    #[error("Invalid Aprs Data")]
    InvalidData,
    #[error("Invalid Message Addressee: {0:?}")]
    InvalidMessageAddressee(Vec<u8>),
    #[error(transparent)]
    Write(#[from] std::io::Error),
}
