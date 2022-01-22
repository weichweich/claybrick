use nom::{bytes, character};
use nom_tracable::tracable_parser;

use super::{backward_search, error::CbParseError, object::dictionary_object, CbParseResult, Span};
use crate::pdf::{Dictionary, Object, Trailer};

pub const TRAILER: &[u8] = b"trailer";
pub const K_SIZE: &[u8] = b"Size";
pub const K_PREVIOUS: &[u8] = b"Prev";
pub const K_ENCRYPT: &[u8] = b"Encrypt";
pub const K_ROOT: &[u8] = b"Root";
pub const K_ID: &[u8] = b"ID";
pub const K_X_REF_STM: &[u8] = b"XRefStm";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrailerError {
    InvalidSize,
    MissingSize,
    InvalidRoot,
    MissingRoot,
    InvalidXRefStm,
    MissingXRefStm,
    InvalidPrevious,
    InvalidInfo,
    InvalidId,
}

fn into_trailer(dict: Dictionary) -> Result<Trailer, TrailerError> {
    Ok(Trailer {
        size: dict
            .get(K_SIZE)
            .ok_or(TrailerError::MissingSize)?
            .integer()
            .ok_or(TrailerError::InvalidSize)?
            .try_into()
            .map_err(|_| TrailerError::InvalidSize)?,

        previous: dict
            .get(K_PREVIOUS)
            .and_then(Object::integer)
            .map(TryInto::try_into)
            .transpose()
            .map_err(|_| TrailerError::InvalidPrevious)?,

        root: dict
            .get(K_ROOT)
            .ok_or(TrailerError::MissingRoot)?
            .reference()
            // TODO: don't clone
            .map(Clone::clone)
            .ok_or(TrailerError::InvalidRoot)?,

        // TODO: don't clone
        encrypt: dict.get(K_ENCRYPT).map(Clone::clone),

        // TODO: don't clone
        info: dict
            .get(K_ROOT)
            .map(|o| o.reference().ok_or(TrailerError::InvalidInfo))
            .transpose()?
            .map(Clone::clone),

        id: dict
            .get(K_ID)
            .map(|o| o.array().ok_or(TrailerError::InvalidId))
            .transpose()?
            .map(|a| {
                if a.len() == 2 {
                    Ok([
                        // TODO: don't clone
                        a.get(0)
                            .and_then(Object::hex_string)
                            .ok_or(TrailerError::InvalidId)?
                            .clone(),
                        // TODO: don't clone
                        a.get(1)
                            .and_then(Object::hex_string)
                            .ok_or(TrailerError::InvalidId)?
                            .clone(),
                    ])
                } else {
                    Err(TrailerError::InvalidId)
                }
            })
            .transpose()?,

        x_ref_stm: dict
            .get(K_X_REF_STM)
            .ok_or(TrailerError::MissingXRefStm)?
            .integer()
            .map(TryInto::try_into)
            .transpose()
            .map_err(|_| TrailerError::InvalidXRefStm)?,
    })
}

#[tracable_parser]
pub fn trailer_tail(input: Span) -> CbParseResult<Trailer> {
    // find `trailer` key word (start search from the end)
    let (remainder, (trailing, _)) = backward_search::<_, _, _, CbParseError<Span>>(
        TRAILER.len() + 4096,
        bytes::complete::tag_no_case(TRAILER),
    )(input)?;

    // remove any whitespace after `trailer` key word and after the dictionary
    let (trailing, _) = character::complete::multispace0(trailing)?;
    let (trailing, trailer) = dictionary_object(trailing)?;
    let (trailing, _) = character::complete::multispace0(trailing)?;
    if trailing.len() > 0 {
        log::warn!("Unexpected bytes after trailer: {:?}", trailing);
    }

    let trailer = into_trailer(trailer).map_err(|err| nom::Err::Failure(CbParseError::new(input, err.into())))?;

    Ok((remainder, trailer))
}
