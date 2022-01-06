use nom::{branch, bytes, character, combinator, error::ParseError, multi, sequence, AsBytes, IResult};
use nom_tracable::tracable_parser;

use crate::pdf::{FreeObject, Unsupported, UsedObject, Xref, XrefStreamEntry, XrefTableEntry};

use super::{
    backward_search,
    error::{CbParseError, CbParseErrorKind},
    object, CbParseResult, Span,
};

const EOF_MARKER: &[u8] = b"%%EOF";
const STARTXREF: &[u8] = b"startxref";

#[tracable_parser]
pub fn startxref_tail(input: Span) -> CbParseResult<usize> {
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

    // FIXME: is it fine to just take a user defined value and request memory like
    // that? Might be a way to crash software?

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

pub(crate) fn xref_section(input: Span) -> CbParseResult<Vec<XrefTableEntry>> {
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

fn xref_stream_entry<'a, E: ParseError<Span<'a>>>(
    w: [usize; 3],
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (usize, usize, usize), E> {
    combinator::map(
        sequence::tuple((
            bytes::complete::take(w[0]),
            bytes::complete::take(w[1]),
            bytes::complete::take(w[2]),
        )),
        |(b1, b2, b3): (Span, Span, Span)| {
            (
                b1.iter().fold(0_usize, |acc, &v| (acc << 8) + usize::from(v)),
                b2.iter().fold(0_usize, |acc, &v| (acc << 8) + usize::from(v)),
                b3.iter().fold(0_usize, |acc, &v| (acc << 8) + usize::from(v)),
            )
        },
    )
}

pub(crate) fn xref_stream_data(w: [usize; 3], input: Span) -> CbParseResult<Vec<XrefStreamEntry>> {
    let entry_len: usize = w.iter().sum();
    let mut entries = Vec::<XrefStreamEntry>::with_capacity(input.len() / entry_len);
    let mut remainder = input;
    let mut entry_parser = xref_stream_entry(w);
    let mut index: usize = 0;
    while remainder.len() >= entry_len {
        let (r, entry) = entry_parser(remainder)?;
        entries.push(match entry {
            // type 1 entry (free object)
            (0, next_free, gen) => XrefStreamEntry::Free(FreeObject {
                number: index,
                gen,
                next_free,
            }),

            // type 2 entry (object position - byte offset)
            (1, byte_offset, gen) => XrefStreamEntry::Used(UsedObject {
                number: index,
                byte_offset,
                gen,
            }),

            // type 3 entry (object position - compressed)
            (2, next_free, gen) => XrefStreamEntry::Free(FreeObject {
                number: index,
                gen,
                next_free,
            }),

            // unsupported entry
            (type_num, w1, w2) => XrefStreamEntry::Unsupported(Unsupported {
                number: index,
                type_num,
                w1,
                w2,
            }),
        });

        index += 1;
        remainder = r;
    }
    Ok((b"".as_bytes().into(), entries))
}

pub(crate) fn xref_stream(input: Span) -> CbParseResult<Vec<XrefStreamEntry>> {
    let (remainder, obj) = object::indirect_object(input)?;

    // get stream that is contained in the indirect object
    let stream = obj
        .indirect()
        .ok_or_else(|| {
            log::error!("startxref didn't point to an indirect object");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?
        .object
        .stream()
        .ok_or_else(|| {
            log::error!("indirect object didn't contain a stream");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?;

    // get the data that is contained in the stream
    log::trace!("Xref stream: {:?}", stream);
    let data = stream
        .filtered_data()
        .map_err(|err| nom::Err::Error(CbParseError::new(input, CbParseErrorKind::StreamError(err))))?;
    log::trace!("Parse Xref stream data");

    // get the W entry in from the stream dictionary
    let w: [i32; 3] = stream
        .dictionary
        .get(&b"W"[..])
        .ok_or_else(|| {
            log::error!("Missing W entry in xref stream dictionary");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?
        .array()
        .ok_or_else(|| {
            log::error!("W entry didn't contain an array object");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?
        .iter()
        .map(|o| o.integer())
        .collect::<Option<Vec<i32>>>()
        .ok_or_else(|| {
            log::error!("Not all entries where integer objects");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?
        .try_into()
        .map_err(|_| {
            log::error!("W didn't contain exactly 3 entries.");
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?;
    let w = [
        usize::try_from(w[0]).map_err(|e| {
            log::error!("W[0] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?,
        usize::try_from(w[1]).map_err(|e| {
            log::error!("W[1] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?,
        usize::try_from(w[2]).map_err(|e| {
            log::error!("W[2] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
        })?,
    ];

    let (empty, stream) = xref_stream_data(w, (&data[..]).into()).map_err(|err| {
        log::error!("Error while parsing xref stream content: {:?}", err);
        nom::Err::Error(CbParseError::new(input, CbParseErrorKind::XrefInvalid))
    })?;
    debug_assert!(empty.len() == 0);

    log::debug!("xref stream data parsed");

    Ok((remainder, stream))
}

#[tracable_parser]
pub fn xref(input: Span) -> CbParseResult<Xref> {
    branch::alt((combinator::into(xref_section), combinator::into(xref_stream)))(input)
}

#[tracable_parser]
pub fn eof_marker_tail(input: Span) -> CbParseResult<()> {
    // trailing bytes that follow the EOF marker are not possible since the limit we
    // provided is the length of the EOF marker
    let (remainder, _trailing) = backward_search::<_, _, _, CbParseError<Span>>(
        EOF_MARKER.len() + 4,
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
