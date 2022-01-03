use APRSError;

pub fn parse_latitude(s: &str) -> Result<f32, APRSError> {
    let b = s.as_bytes();

    if b.len() != 8 || b[4] as char != '.' {
        return Err(APRSError::InvalidLatitude(s.to_owned()));
    }

    let north = match b[7] as char {
        'N' => true,
        'S' => false,
        _ => return Err(APRSError::InvalidLatitude(s.to_owned())),
    };

    let deg = s[0..2]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLatitude(s.to_owned()))? as f32;
    let min = s[2..4]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLatitude(s.to_owned()))? as f32;
    let min_frac = s[5..7]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLatitude(s.to_owned()))? as f32;

    let value = deg + min / 60. + min_frac / 6_000.;

    Ok(if north { value } else { -value })
}

pub fn parse_longitude(s: &str) -> Result<f32, APRSError> {
    let b = s.as_bytes();

    if b.len() != 9 || b[5] as char != '.' {
        return Err(APRSError::InvalidLongitude(s.to_owned()));
    }

    let east = match b[8] as char {
        'E' => true,
        'W' => false,
        _ => return Err(APRSError::InvalidLongitude(s.to_owned())),
    };

    let deg = s[0..3]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLongitude(s.to_owned()))? as f32;
    let min = s[3..5]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLongitude(s.to_owned()))? as f32;
    let min_frac = s[6..8]
        .parse::<u32>()
        .map_err(|_| APRSError::InvalidLongitude(s.to_owned()))? as f32;

    let value = deg + min / 60. + min_frac / 6_000.;

    Ok(if east { value } else { -value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latitude() {
        assert_relative_eq!(parse_latitude("4903.50N").unwrap(), 49.05833);
        assert_relative_eq!(parse_latitude("4903.50S").unwrap(), -49.05833);
        assert_eq!(
            parse_latitude("4903.50W"),
            Err(APRSError::InvalidLatitude("4903.50W".to_owned()))
        );
        assert_eq!(
            parse_latitude("4903.50E"),
            Err(APRSError::InvalidLatitude("4903.50E".to_owned()))
        );
        assert_relative_eq!(parse_latitude("0000.00N").unwrap(), 0.0);
        assert_relative_eq!(parse_latitude("0000.00S").unwrap(), 0.0);
    }

    #[test]
    fn test_longitude() {
        assert_relative_eq!(parse_longitude("12903.50E").unwrap(), 129.05833);
        assert_relative_eq!(parse_longitude("04903.50W").unwrap(), -49.05833);
        assert_eq!(
            parse_longitude("04903.50N"),
            Err(APRSError::InvalidLongitude("04903.50N".to_owned()))
        );
        assert_eq!(
            parse_longitude("04903.50S"),
            Err(APRSError::InvalidLongitude("04903.50S".to_owned()))
        );
        assert_relative_eq!(parse_longitude("00000.00E").unwrap(), 0.0);
        assert_relative_eq!(parse_longitude("00000.00W").unwrap(), 0.0);
    }
}
