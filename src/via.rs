use std::io::{self, Write};

use Callsign;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Via {
    Callsign(Callsign, bool),
    QConstruct(QConstruct),
}

impl Via {
    pub fn decode_textual(bytes: &[u8]) -> Option<Self> {
        if let Some(q) = QConstruct::decode_textual(bytes) {
            return Some(Self::QConstruct(q));
        }

        if let Some((c, heard)) = Callsign::decode_textual(bytes) {
            return Some(Self::Callsign(c, heard));
        }

        None
    }

    pub fn encode_textual<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            Self::Callsign(c, heard) => {
                c.encode_textual(*heard, w)?;
            }
            Self::QConstruct(q) => {
                write!(w, "{}", q.as_textual())?;
            }
        }

        Ok(())
    }

    pub fn callsign(&self) -> Option<(&Callsign, bool)> {
        match self {
            Self::Callsign(c, heard) => Some((c, *heard)),
            Self::QConstruct(_) => None,
        }
    }

    pub fn callsign_mut(&mut self) -> Option<(&mut Callsign, &mut bool)> {
        match self {
            Self::Callsign(c, heard) => Some((c, heard)),
            Self::QConstruct(_) => None,
        }
    }
}

// Can't be encoded/decoded as ax.25
// These should never go on the air
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum QConstruct {
    AC,
    AX,
    AU,
    Ao,
    AO,
    AS,
    Ar,
    AR,
    AZ,
    AI,
}

impl QConstruct {
    pub fn decode_textual(bytes: &[u8]) -> Option<Self> {
        let q = match bytes {
            b"qAC" => QConstruct::AC,
            b"qAX" => QConstruct::AX,
            b"qAU" => QConstruct::AU,
            b"qAo" => QConstruct::Ao,
            b"qAO" => QConstruct::AO,
            b"qAS" => QConstruct::AS,
            b"qAr" => QConstruct::Ar,
            b"qAR" => QConstruct::AR,
            b"qAZ" => QConstruct::AZ,
            b"qAI" => QConstruct::AI,
            _ => return None,
        };

        Some(q)
    }

    pub fn as_textual(&self) -> &'static str {
        match self {
            QConstruct::AC => "qAC",
            QConstruct::AX => "qAX",
            QConstruct::AU => "qAU",
            QConstruct::Ao => "qAo",
            QConstruct::AO => "qAO",
            QConstruct::AS => "qAS",
            QConstruct::Ar => "qAr",
            QConstruct::AR => "qAR",
            QConstruct::AZ => "qAZ",
            QConstruct::AI => "qAI",
        }
    }
}
