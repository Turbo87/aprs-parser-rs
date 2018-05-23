use std::str::FromStr;

use APRSError;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Timestamp {
    /// Day of month, Hour and Minute in UTC
    DDHHMM(u8, u8, u8),
    /// Hour, Minute and Second in UTC
    HHMMSS(u8, u8, u8),
    /// Unsupported timestamp format
    Unsupported(String),
}

impl FromStr for Timestamp {
    type Err = APRSError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let b = s.as_bytes();

        if b.len() != 7 {
            return Err(APRSError::InvalidTimestamp(s.to_owned()));
        }

        let one = s[0..2].parse::<u8>().map_err(|_| APRSError::InvalidTimestamp(s.to_owned()))?;
        let two = s[2..4].parse::<u8>().map_err(|_| APRSError::InvalidTimestamp(s.to_owned()))?;
        let three = s[4..6].parse::<u8>().map_err(|_| APRSError::InvalidTimestamp(s.to_owned()))?;

        Ok(match b[6] as char {
            'z' => Timestamp::DDHHMM(one, two, three),
            'h' => Timestamp::HHMMSS(one, two, three),
            '/' => Timestamp::Unsupported(s.to_owned()),
            _ => return Err(APRSError::InvalidTimestamp(s.to_owned())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ddhhmm() {
        assert_eq!("123456z".parse(), Ok(Timestamp::DDHHMM(12, 34, 56)));
    }

    #[test]
    fn parse_hhmmss() {
        assert_eq!("123456h".parse(), Ok(Timestamp::HHMMSS(12, 34, 56)));
    }

    #[test]
    fn parse_local_time() {
        assert_eq!("123456/".parse::<Timestamp>(), Ok(Timestamp::Unsupported("123456/".to_owned())));
    }

    #[test]
    fn invalid_timestamp() {
        assert_eq!("1234567".parse::<Timestamp>(), Err(APRSError::InvalidTimestamp("1234567".to_owned())));
    }

    #[test]
    fn invalid_timestamp2() {
        assert_eq!("123a56z".parse::<Timestamp>(), Err(APRSError::InvalidTimestamp("123a56z".to_owned())));
    }
}
