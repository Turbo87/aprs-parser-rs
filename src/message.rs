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

        let addressee = addressee
            .split_once(' ')
            .map_or(addressee, |x| x.0)
            .to_string();

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
