// functions for working with byte arrays

pub fn parse_bytes<T: std::str::FromStr>(b: &[u8]) -> Option<T> {
    std::str::from_utf8(b).ok()?.parse().ok()
}
