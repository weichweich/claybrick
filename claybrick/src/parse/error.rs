use nom::error::{ErrorKind, ParseError};

use super::xref::XrefError;
use crate::pdf::{object::stream::filter::FilterError, trailer::TrailerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CbParseErrorKind {
    InvalidTrailer(TrailerError),
    StartxrefInvalid,
    BackwardSearchNotFound,
    // TODO: More detailed errors
    XrefInvalid(XrefError),
    StreamError(FilterError),
    InvalidName,
    Nom(ErrorKind),
}

impl From<TrailerError> for CbParseErrorKind {
    fn from(err: TrailerError) -> Self {
        CbParseErrorKind::InvalidTrailer(err)
    }
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
