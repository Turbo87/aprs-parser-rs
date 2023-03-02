use std::convert::TryFrom;
use std::io::Write;

use DecodeError;
use EncodeError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AprsMessage {
    pub addressee: Vec<u8>,
    pub text: Vec<u8>,
    pub id: Option<Vec<u8>>,
}

impl AprsMessage {
    pub fn encode<W: Write>(&self, buf: &mut W) -> Result<(), EncodeError> {
        if self.addressee.len() > 9 {
            return Err(EncodeError::InvalidMessageAddressee(
                self.addressee.to_owned(),
            ));
        }

        buf.write_all(b":")?;
        buf.write_all(&self.addressee)?;
        for _ in self.addressee.len()..9 {
            buf.write_all(b" ")?;
        }

        buf.write_all(b":")?;
        buf.write_all(&self.text)?;

        if let Some(id) = &self.id {
            buf.write_all(b"{")?;
            buf.write_all(id)?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for AprsMessage {
    type Error = DecodeError;

    fn try_from(b: &[u8]) -> Result<Self, DecodeError> {
        let mut splitter = b.splitn(2, |x| *x == b':');

        let mut addressee = match splitter.next() {
            Some(x) => x.to_vec(),
            None => {
                return Err(DecodeError::InvalidMessageDestination(vec![]));
            }
        };

        if addressee.len() != 9 {
            return Err(DecodeError::InvalidMessageDestination(addressee.to_owned()));
        }

        trim_spaces_end(&mut addressee);

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

fn trim_spaces_end(arr: &mut Vec<u8>) {
    let space_count = arr.iter().rev().take_while(|&&b| b == b' ').count();

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
            Err(DecodeError::InvalidMessageDestination(b"DEST  ".to_vec()))
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
