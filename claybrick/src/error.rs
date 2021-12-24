use crate::parse::error::CbParseError;

#[derive(Debug, Clone)]
pub enum CbError {
    Parse,
    Io,
}

impl<I> From<nom::Err<CbParseError<I>>> for CbError {
    fn from(_err: nom::Err<CbParseError<I>>) -> Self {
        CbError::Parse
    }
}

impl From<std::io::Error> for CbError {
    fn from(_: std::io::Error) -> Self {
        CbError::Io
    }
}