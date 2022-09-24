pub fn encode_ascii(val: f32) -> Vec<u8> {
    todo!(); // TODO this should take in a writer
}

pub fn decode_ascii(bytes: &[u8]) -> Option<f32> {
    let mut val = 0.0;

    for b in bytes {
        // APRS standard - subtract 33
        let x = b.checked_sub(33)?;

        val *= 91.0;
        val += x as f64;
    }

    // Need to do this as f64 and then cast
    // If we keep f32 the whole way, the
    // errors will stack up quickly
    Some(val as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let val = 20427156.0;
        let expected = &b"<*e7"[..];

        assert_eq!(expected, encode_ascii(val));
    }

    #[test]
    fn decode_works() {
        let ascii = &b"<*e7"[..];
        let expected = 20427156.0;

        assert_eq!(Some(expected), decode_ascii(ascii));
    }
}
