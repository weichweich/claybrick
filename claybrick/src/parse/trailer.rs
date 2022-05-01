use nom::{bytes, character};
use nom_tracable::tracable_parser;

use super::{backward_search, error::CbParseError, object::dictionary_object, CbParseResult, Span};
use crate::pdf::{trailer::TRAILER, Trailer};

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

    let trailer = Trailer::try_from(trailer).map_err(|err| nom::Err::Failure(CbParseError::new(input, err.into())))?;

    Ok((remainder, trailer))
}
