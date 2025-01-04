use Callsign;
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum DecodeError {
    #[error("Invalid Callsign: {0:?}")]
    InvalidCallsign(Vec<u8>),
    #[error("Invalid Via: {0:?}")]
    InvalidVia(Vec<u8>),
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
    #[error("Invalid Compressed cs: {0:?}")]
    InvalidCs([u8; 2]),
    #[error("Invalid Mic-E destination address: {0:}")]
    InvalidMicEDestination(Callsign),
    #[error("Invalid Mic-E information field: {0:?}")]
    InvalidMicEInformation(Vec<u8>),
    #[error("Invalid Object format {0:?}, {1}")]
    InvalidObjectFormat(Vec<u8>, String),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("Callsign can't be encoded: {0:}")]
    InvalidCallsign(Callsign),
    #[error("Invalid Latitude: {0}")]
    InvalidLatitude(f64),
    #[error("Invalid Longitude: {0}")]
    InvalidLongitude(f64),
    #[error("Invalid Aprs Data")]
    InvalidData,
    #[error("Invalid Message Addressee: {0:?}")]
    InvalidMessageAddressee(Vec<u8>),
    #[error("Compressed altitude requires the nmea source to be gga")]
    NonGgaAltitude,
    #[error(transparent)]
    Write(#[from] std::io::Error),
}
