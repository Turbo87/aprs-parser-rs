use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use AprsError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsMessage {
    pub addressee: Vec<u8>,
    pub text: Vec<u8>,
    pub id: Option<Vec<u8>>,
}

impl TryFrom<&[u8]> for AprsMessage {
    type Error = AprsError;

    fn try_from(b: &[u8]) -> Result<Self, AprsError> {
        let mut splitter = b.splitn(2, |x| *x == b':');

        let mut addressee = match splitter.next() {
            Some(x) => x.to_vec(),
            None => {
                return Err(AprsError::InvalidMessageDestination(vec![]));
            }
        };

        if addressee.len() != 9 {
            return Err(AprsError::InvalidMessageDestination(addressee.to_owned()));
        }

        remove_trailing_spaces(&mut addressee);

        let text = splitter.next().unwrap_or(&[]);
        let mut text_splitter = text.splitn(2, |x| *x == b'{');
        let text = text_splitter.next().unwrap_or(&[]).to_vec();
        let id = text_splitter.next().map(|x| x.to_vec());

        Ok(Self {
            addressee,
            text,
            id,
        })
    }
}

impl Display for AprsMessage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            ":{: <9}:{}",
            String::from_utf8_lossy(&self.addressee),
            String::from_utf8_lossy(&self.text)
        )?;

        if let Some(id) = &self.id {
            write!(f, "{{{}", String::from_utf8_lossy(id))?;
        }

        Ok(())
    }
}

fn remove_trailing_spaces(arr: &mut Vec<u8>) {
    let mut space_count = 0;

    for b in arr.iter_mut().rev() {
        if *b == b' ' {
            space_count += 1;
        } else {
            break;
        }
    }

    arr.truncate(arr.len() - space_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message_invalid_dest() {
        // Dest must be padded with spaces to 9 characters long
        let result =
            AprsMessage::try_from(&b"DEST  :Hello World! This msg has a : colon {32975"[..]);

        assert_eq!(
            result,
            Err(AprsError::InvalidMessageDestination(b"DEST  ".to_vec()))
        );
    }

    #[test]
    fn parse_message_id() {
        let result = AprsMessage::try_from(
            r"DESTINATI:Hello World! This msg has a : colon {329A7D5Z4".as_bytes(),
        );

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: b"DESTINATI".to_vec(),
                id: Some(b"329A7D5Z4".to_vec()),
                text: b"Hello World! This msg has a : colon ".to_vec()
            })
        );
    }

    #[test]
    fn parse_message_empty_id() {
        let result =
            AprsMessage::try_from(r"DESTINATI:Hello World! This msg has a : colon {".as_bytes());

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: b"DESTINATI".to_vec(),
                id: Some(vec![]),
                text: b"Hello World! This msg has a : colon ".to_vec()
            })
        );
    }

    #[test]
    fn parse_message_no_id() {
        let result =
            AprsMessage::try_from(r"DESTINATI:Hello World! This msg has a : colon ".as_bytes());

        assert_eq!(
            result,
            Ok(AprsMessage {
                addressee: b"DESTINATI".to_vec(),
                id: None,
                text: b"Hello World! This msg has a : colon ".to_vec()
            })
        );
    }
}
