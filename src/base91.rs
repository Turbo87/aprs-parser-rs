use std::io::Write;

use EncodeError;

pub(crate) fn encode_ascii<W: Write>(
    val: f64,
    buf: &mut W,
    padding: usize,
) -> Result<(), EncodeError> {
    let mut val = val.round();
    let mut digit_buf = vec![];

    debug_assert!(!val.is_nan() && val > 0.0 && !val.is_infinite());

    while val > 1.0 {
        let x = val % 91.0;
        val /= 91.0;

        digit_buf.push(digit_to_ascii(x as u8));
    }

    // pad with zeroes
    for _ in digit_buf.len()..padding {
        buf.write_all(&[digit_to_ascii(0)])?;
    }

    digit_buf.reverse();
    buf.write_all(&digit_buf)?;

    Ok(())
}

pub(crate) fn decode_ascii(bytes: &[u8]) -> Option<f64> {
    let mut val = 0.0;

    for b in bytes {
        let x = digit_from_ascii(*b)?;

        val *= 91.0;
        val += x as f64;
    }
    Some(val)
}

// APRS standard - add 33
pub(crate) fn digit_to_ascii(digit: u8) -> u8 {
    digit + 33
}

// APRS standard - subtract 33
pub(crate) fn digit_from_ascii(ascii: u8) -> Option<u8> {
    ascii.checked_sub(33)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let val = 20427156.0;
        let expected = &b"<*e7"[..];

        let mut buf = vec![];
        encode_ascii(val, &mut buf, 4).unwrap();
        assert_eq!(expected, buf);
    }

    #[test]
    fn encode_with_padding() {
        let val = 20427156.0;
        let expected = &b"!!!!<*e7"[..];

        let mut buf = vec![];
        encode_ascii(val, &mut buf, 8).unwrap();
        assert_eq!(expected, buf);
    }

    #[test]
    fn encode_with_under_padding() {
        let val = 20427156.0;
        let expected = &b"<*e7"[..];

        let mut buf = vec![];
        encode_ascii(val, &mut buf, 1).unwrap();
        assert_eq!(expected, buf);
    }

    #[test]
    fn decode_works() {
        let ascii = &b"<*e7"[..];
        let expected = 20427156.0;

        assert_eq!(Some(expected), decode_ascii(ascii));
    }

    #[test]
    fn decode_invalid_digits_returns_none() {
        let ascii = &b"<* 1"[..];
        assert_eq!(None, decode_ascii(ascii));
    }

    #[test]
    fn edge_case() {
        let ascii = &b"#$%^"[..];
        let num = 1532410.0;

        assert_eq!(num, decode_ascii(ascii).unwrap());

        let mut buf = vec![];
        encode_ascii(num, &mut buf, 4).unwrap();
        assert_eq!(ascii, buf);
    }
}
