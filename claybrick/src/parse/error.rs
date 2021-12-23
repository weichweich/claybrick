use nom::error::{ErrorKind, ParseError};

#[derive(Debug)]
pub enum CbErrorKind {
    StringInvalidUft8,
    Nom(ErrorKind),
}

#[derive(Debug)]
pub struct CbError<I> {
    pub input: I,
    pub kind: CbErrorKind,
    pub from: Option<Box<Self>>,
}

impl<I> ParseError<I> for CbError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            input,
            kind: CbErrorKind::Nom(kind),
            from: None,
        }
    }

    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        Self {
            input,
            kind: CbErrorKind::Nom(kind),
            from: Some(other.into()),
        }
    }
}
