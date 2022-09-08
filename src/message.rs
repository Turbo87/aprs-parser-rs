use std::fmt::{Display, Formatter};

use AprsError;
use FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsMessage {
    pub addressee: String,
    pub text: String,
    pub id: Option<String>,
}

impl FromStr for AprsMessage {
    type Err = AprsError;

    fn from_str(s: &str) -> Result<Self, AprsError> {
        let mut splitter = s.splitn(2, ':');

        let addressee = match splitter.next() {
            Some(x) => x,
            None => {
                return Err(AprsError::InvalidMessageDestination("".to_string()));
            }
        };

        if addressee.len() != 9 {
            return Err(AprsError::InvalidMessageDestination(addressee.to_string()));
        }

        let addressee = addressee.trim().to_string();

        let text = splitter.next().unwrap_or("");
        let mut text_splitter = text.splitn(2, '{');
        let text = text_splitter.next().unwrap_or("").to_string();
        let id = text_splitter.next().map(|s| s.to_string());

        Ok(Self {
            addressee,
            text,
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message_invalid_dest() {
        // Dest must be padded with spaces to 9 characters long
        let result = r"DEST  :Hello World! This msg has a : colon {32975".parse::<AprsMessage>();

        assert_eq!(
            result,
            Err(AprsError::InvalidMessageDestination("DEST  ".to_string()))
        );
    }

    #[test]
    fn parse_message_id() {
        let result =
            r"DESTINATI:Hello World! This msg has a : colon {329A7D5Z4".parse::<AprsMessage>();

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: "DESTINATI".to_string(),
                id: Some("329A7D5Z4".to_string()),
                text: "Hello World! This msg has a : colon ".to_string()
            })
        );
    }

    #[test]
    fn parse_message_empty_id() {
        let result = r"DESTINATI:Hello World! This msg has a : colon {".parse::<AprsMessage>();

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: "DESTINATI".to_string(),
                id: Some("".to_string()),
                text: "Hello World! This msg has a : colon ".to_string()
            })
        );
    }

    #[test]
    fn parse_message_no_id() {
        let result = r"DESTINATI:Hello World! This msg has a : colon ".parse::<AprsMessage>();

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: "DESTINATI".to_string(),
                id: None,
                text: "Hello World! This msg has a : colon ".to_string()
            })
        );
    }
}

impl Display for AprsMessage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, ":{: <9}:{}", self.addressee, self.text)?;

        if let Some(id) = &self.id {
            write!(f, "{{{}", id)?;
        }

        Ok(())
    }
}
