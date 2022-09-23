// functions for working with byte arrays

pub fn parse_bytes<T: std::str::FromStr>(b: &[u8]) -> Option<T> {
    std::str::from_utf8(b).ok()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correctly_u32() {
        assert_eq!(Some(123), parse_bytes::<u32>(b"0123"));
    }

    #[test]
    fn parse_correctly_f32() {
        assert_relative_eq!(123.456, parse_bytes::<f32>(b"0123.4560").unwrap());
    }

    #[test]
    fn parse_fail_on_non_utf8() {
        assert_eq!(None, parse_bytes::<u32>(b"\xF0\xA4\xAD"));
    }

    #[test]
    fn parse_fail_on_not_a_number() {
        assert_eq!(None, parse_bytes::<u32>(b"0123NotANumber"));
        assert_eq!(None, parse_bytes::<u32>(b"NotANumber0123"))
    }
}
