use nom::error::{ErrorKind, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CbParseErrorKind {
    StartxrefInvalid,
    BackwardSearchNotFound,
    // TODO: More detailed errors
    XrefInvalid,
    Nom(ErrorKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CbParseError<I> {
    pub input: I,
    pub kind: CbParseErrorKind,
    pub from: Option<Box<Self>>,
}

impl<I> CbParseError<I> {
    pub fn new(input: I, kind: CbParseErrorKind) -> Self {
        Self {
            input,
            kind,
            from: None,
        }
    }
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
