use std::io::Write;

use EncodeError;

pub fn encode_ascii<W: Write>(val: f64, buf: &mut W, padding: usize) -> Result<(), EncodeError> {
    let mut val = val.round();
    let mut digit_buf = vec![];

    while val > 1.0 {
        let x = val % 91.0;
        val /= 91.0;

        // APRS standard - add 33
        digit_buf.push(x as u8 + 33);
    }

    // pad with zeroes, plus 33
    for _ in digit_buf.len()..padding {
        buf.write_all(&[33])?;
    }

    digit_buf.reverse();
    buf.write_all(&digit_buf)?;

    Ok(())
}

pub fn decode_ascii(bytes: &[u8]) -> Option<f64> {
    let mut val = 0.0;

    for b in bytes {
        // APRS standard - subtract 33
        let x = b.checked_sub(33)?;

        val *= 91.0;
        val += x as f64;
    }
    Some(val)
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
    fn test_edge_case() {
        let ascii = &b"#$%^"[..];
        let num = 1532410.0;

        assert_eq!(num, decode_ascii(ascii).unwrap());

        let mut buf = vec![];
        encode_ascii(num, &mut buf, 4).unwrap();
        assert_eq!(ascii, buf);
    }
}
