use nom::{branch, bytes, character, combinator, multi};
use nom_tracable::tracable_parser;

use crate::pdf::XrefTableEntry;

use super::{
    backward_search,
    error::{CbParseError, CbParseErrorKind},
    CbParseResult, Span,
};

const EOF_MARKER: &[u8] = b"%%EOF";
const STARTXREF: &[u8] = b"startxref";

#[tracable_parser]
pub(crate) fn startxref_tail(input: Span) -> CbParseResult<usize> {
    let (remainder, (trailing, _)) = backward_search::<_, _, _, CbParseError<Span>>(
        STARTXREF.len() + 2048,
        bytes::complete::tag_no_case(STARTXREF),
    )(input)?;
    let (trailing, _) = character::complete::multispace0(trailing)?;
    let (_, xref_pos) = character::complete::u64(trailing)?;
    let xref_pos: usize = xref_pos
        .try_into()
        .map_err(|_| nom::Err::Error(CbParseError::new(input, CbParseErrorKind::StartxrefInvalid)))?;

    Ok((remainder, xref_pos))
}

#[tracable_parser]
fn xref_entries(input: Span) -> CbParseResult<Vec<XrefTableEntry>> {
    let (remainder, obj_index_offset) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;
    let (remainder, obj_count) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    // FIXME: is it fine to just take a user defined value and request memory like that? Might be a way to crash software?
    // FIXME: Iterate on convertion error handling (is 5 a good default?)
    let mut entries = Vec::<XrefTableEntry>::with_capacity(obj_count.try_into().unwrap_or(5));

    let mut remainder = remainder;
    for i in 0..obj_count {
        let (inner_rmndr, offset) = character::complete::u64(remainder)?;
        let (inner_rmndr, _) = character::complete::multispace0(inner_rmndr)?;
        let (inner_rmndr, gen) = character::complete::u32(inner_rmndr)?;
        let (inner_rmndr, _) = character::complete::multispace0(inner_rmndr)?;
        let (inner_rmndr, free) = branch::alt((
            combinator::value(false, bytes::complete::tag(b"n")),
            combinator::value(true, bytes::complete::tag(b"f")),
        ))(inner_rmndr)?;
        let (inner_rmndr, _) = character::complete::multispace0(inner_rmndr)?;

        entries.push(XrefTableEntry {
            // FIXME: no unwrap!
            object: (obj_index_offset + i).try_into().unwrap(),
            // FIXME: no unwrap!
            byte_offset: offset.try_into().unwrap(),
            generation: gen,
            free,
        });
        remainder = inner_rmndr;
    }

    Ok((remainder, entries))
}

#[tracable_parser]
pub(crate) fn xref_table(input: Span) -> CbParseResult<Vec<XrefTableEntry>> {
    // xref keyword
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, _) = bytes::complete::tag(b"xref")(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;
    let (remainder, entries) = multi::many1(xref_entries)(remainder)?;
    let size = entries.iter().map(Vec::len).sum();
    let mut entries_flatten = Vec::with_capacity(size);
    for v in entries {
        entries_flatten.extend_from_slice(&v[..]);
    }
    Ok((remainder, entries_flatten))
}

#[tracable_parser]
pub(crate) fn eof_marker_tail(input: Span) -> CbParseResult<()> {
    // trailing bytes that follow the EOF marker are not possible since the limit we
    // provided is the length of the EOF marker
    let (remainder, _trailing) = backward_search::<_, _, _, CbParseError<Span>>(
        EOF_MARKER.len(),
        bytes::complete::tag_no_case(EOF_MARKER),
    )(input)?;

    Ok((remainder, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startxref_tail() {
        let input = &b"         startxref\n2132"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Ok((_, 2132))));

        let input = &b"         startxref\n555\nasdfsadfasdfsadfasdfsadfsadf"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Ok((_, 555))));
    }

    #[test]
    fn test_invalid_startxref_tail() {
        // to big
        let input = &b"         startxref\n9999999999999999999999999999999"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Err(nom::Err::Error(_))));
    }
}
