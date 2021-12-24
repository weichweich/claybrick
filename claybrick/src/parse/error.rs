use nom::error::{ErrorKind, ParseError};

#[derive(Debug, Clone)]
pub enum CbParseErrorKind {
    Nom(ErrorKind),
}

#[derive(Debug, Clone)]
pub struct CbParseError<I> {
    pub input: I,
    pub kind: CbParseErrorKind,
    pub from: Option<Box<Self>>,
}

impl<I> ParseError<I> for CbParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            input,
            kind: CbParseErrorKind::Nom(kind),
            from: None,
        }
    }

    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        Self {
            input,
            kind: CbParseErrorKind::Nom(kind),
            from: Some(other.into()),
        }
    }
}
