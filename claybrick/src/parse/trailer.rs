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
    InvalidRoot,
}

fn into_trailer(dict: Dictionary) -> Result<Trailer, TrailerError> {
    Ok(Trailer {
        // TODO: Error for missing size
        size: dict
            .get(K_SIZE)
            .and_then(Object::integer)
            .ok_or(TrailerError::InvalidSize)?,

        // TODO: Error for invalid previous
        previous: dict.get(K_PREVIOUS).and_then(Object::integer),

        root: dict
            .get(K_ROOT)
            // TODO: separate error for invalid or missing root
            .and_then(Object::reference)
            // TODO: don't clone
            .map(Clone::clone)
            .ok_or(TrailerError::InvalidRoot)?,

        // TODO: don't clone
        encrypt: dict.get(K_ENCRYPT).map(Clone::clone),

        // TODO: don't clone
        // TODO: Error for invalid info
        info: dict.get(K_ROOT).and_then(Object::dictionary).map(Clone::clone),

        // TODO: Error for invalid id
        id: dict.get(K_ID).and_then(Object::array).and_then(|a| {
            if a.len() == 2 {
                Some([
                    // TODO: don't clone
                    a.get(0).and_then(Object::hex_string)?.clone(),
                    // TODO: don't clone
                    a.get(1).and_then(Object::hex_string)?.clone(),
                ])
            } else {
                None
            }
        }),

        // TODO: Error for invalid XRefStm
        x_ref_stm: dict.get(K_X_REF_STM).and_then(Object::integer),
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
