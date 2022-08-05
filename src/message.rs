use std::fmt::{Display, Formatter};

use AprsError;
use FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsMessage {
    pub addressee: String,
    pub text: String,
    pub id: Option<u32>,
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
        let id_s = text_splitter.next();

        let id: Option<u32> = match id_s {
            Some(s) => {
                let id = s.parse();

                match id {
                    Ok(x) => {
                        if x < 100_000 {
                            Some(x)
                        } else {
                            return Err(AprsError::InvalidMessageId(s.to_string()));
                        }
                    }

                    Err(_) => return Err(AprsError::InvalidMessageId(s.to_string())),
                }
            }
            None => None,
        };

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
    fn parse_message_invalid_id() {
        let result =
            r"DESTINATI:Hello World! This msg has a : colon {329754".parse::<AprsMessage>();

        assert_eq!(
            result,
            Err(AprsError::InvalidMessageId("329754".to_string()))
        );
    }
}

impl Display for AprsMessage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, ":{: <9}:{}", self.addressee, self.text)?;

        if let Some(id) = self.id {
            write!(f, "{{{}", id)?;
        }

        Ok(())
    }
}
