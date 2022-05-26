//! XRef Parsing.

use nom::{branch, bytes, character, combinator, error::ParseError, multi, sequence, IResult};
use nom_tracable::tracable_parser;

use crate::{
    parse::{
        backward_search,
        error::{CbParseError, CbParseErrorKind},
        object, CbParseResult, Span,
    },
    pdf::xref::{
        FreeObject, Unsupported, UsedCompressedObject, UsedObject, Xref, XrefEntry, XREF_COMPRESSED, XREF_FREE,
        XREF_USED,
    },
};

const EOF_MARKER: &[u8] = b"%%EOF";
const STARTXREF: &[u8] = b"startxref";

/// Errors that occur while parsing the xref section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XrefError {
    /// The stream object wasn't the correct type.
    StreamObject,

    /// The W entry in the stream object dictionary was invalid.
    WEntry,

    /// There was an error in the content of the xref stream.
    StreamContent,
}

/// Find and returns the position of the xref table/stream by searching for
/// `startxref <number>` from the end of the input and parsing the number that
/// follows.
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

/// Parse a section of the XRef table.
///
/// Retruns a vector of free objects or used objects that can be accessed by the
/// byte offset.
#[tracable_parser]
fn xref_entries(input: Span) -> CbParseResult<Vec<XrefEntry>> {
    let (remainder, obj_index_offset) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;
    let (remainder, obj_count) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    // FIXME: is it fine to just take a user defined value and request memory like
    // that? Might be a way to crash software?
    let mut entries = if let Ok(count) = obj_count.try_into() {
        Vec::<XrefEntry>::with_capacity(count)
    } else {
        Vec::<XrefEntry>::new()
    };

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

        let entry = if free {
            XrefEntry::Free(FreeObject {
                // FIXME: no unwrap!
                number: (obj_index_offset + i).try_into().unwrap(),
                // FIXME: no unwrap!
                next_free: offset.try_into().unwrap(),
                // FIXME: no unwrap!
                generation: gen.try_into().unwrap(),
            })
        } else {
            XrefEntry::Used(UsedObject {
                // FIXME: no unwrap!
                number: (obj_index_offset + i).try_into().unwrap(),
                // FIXME: no unwrap!
                byte_offset: offset.try_into().unwrap(),
                // FIXME: no unwrap!
                generation: gen.try_into().unwrap(),
            })
        };

        entries.push(entry);
        remainder = inner_rmndr;
    }
    log::debug!("Expected {} xef entries, got {}", obj_count, entries.len());

    Ok((remainder, entries))
}

/// Parses a complete xref section which starts with the `xref` keyword.
///
/// Retruns a vector of free objects or used objects that can be accessed by the
/// byte offset.
pub(crate) fn xref_section(input: Span) -> CbParseResult<Xref> {
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

    let xref = Xref::new_table(entries_flatten);
    Ok((remainder, xref))
}

/// A stream entry consists of 3 numbers with variable length. This function
/// takes the 3 length values and returns a parser that accepts 3 numbers with
/// the supplied length.
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

/// Parses the xref-stream data.
///
/// `w` - the byte length of the three numbers in each stream entry.
/// Each entry contains three integers (Type, x, y). The byte length of each
/// integer is specified by the three w values.
pub(crate) fn xref_stream_data(w: [usize; 3], input: Span) -> CbParseResult<Vec<XrefEntry>> {
    let entry_len: usize = w.iter().sum();
    let mut entries = Vec::<XrefEntry>::with_capacity(input.len() / entry_len);
    let mut remainder = input;
    let mut entry_parser = xref_stream_entry(w);
    let mut index: usize = 0;
    while remainder.len() >= entry_len {
        let (r, entry) = entry_parser(remainder)?;

        entries.push(match entry {
            // type 0 entry (free object)
            (XREF_FREE, next_free, gen) => XrefEntry::Free(FreeObject {
                number: index,
                generation: gen,
                next_free,
            }),

            // type 1 entry (object position - byte offset)
            (XREF_USED, byte_offset, gen) => XrefEntry::Used(UsedObject {
                number: index,
                byte_offset,
                generation: gen,
            }),

            // type 2 entry (object position - compressed)
            (XREF_COMPRESSED, containing_object, object_index) => XrefEntry::UsedCompressed(UsedCompressedObject {
                number: index,
                containing_object,
                index: object_index,
            }),

            // unsupported entry
            (type_num, w1, w2) => XrefEntry::Unsupported(Unsupported {
                number: index,
                type_num,
                w1,
                w2,
            }),
        });

        index += 1;
        remainder = r;
    }
    Ok((remainder, entries))
}

/// Parse an indirect object that contains a xref stream.
pub(crate) fn xref_stream(input: Span) -> CbParseResult<Xref> {
    let (remainder, obj) = object::indirect_object(input)?;

    // get stream that is contained in the indirect object
    let indirect_obj = obj.indirect().ok_or_else(|| {
        log::error!("startxref didn't point to an indirect object");
        nom::Err::Error(CbParseError::new(
            input,
            CbParseErrorKind::XrefInvalid(XrefError::StreamObject),
        ))
    })?;
    let stream = indirect_obj.object.stream().ok_or_else(|| {
        log::error!("indirect object didn't contain a stream");
        nom::Err::Error(CbParseError::new(
            input,
            CbParseErrorKind::XrefInvalid(XrefError::StreamObject),
        ))
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
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?
        .array()
        .ok_or_else(|| {
            log::error!("W entry didn't contain an array object");
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?
        .iter()
        .map(|o| o.integer())
        .collect::<Option<Vec<i32>>>()
        .ok_or_else(|| {
            log::error!("Not all entries where integer objects");
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?
        .try_into()
        .map_err(|_| {
            log::error!("W didn't contain exactly 3 entries.");
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?;
    let w = [
        usize::try_from(w[0]).map_err(|e| {
            log::error!("W[0] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?,
        usize::try_from(w[1]).map_err(|e| {
            log::error!("W[1] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?,
        usize::try_from(w[2]).map_err(|e| {
            log::error!("W[2] can't be converted to usize ({})", e);
            nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::XrefInvalid(XrefError::WEntry),
            ))
        })?,
    ];

    let (_empty, entries) = xref_stream_data(w, data[..].into()).map_err(|err| {
        log::error!("Error while parsing xref stream content: {:?}", err);
        nom::Err::Error(CbParseError::new(
            input,
            CbParseErrorKind::XrefInvalid(XrefError::StreamContent),
        ))
    })?;

    log::debug!("xref stream data parsed");

    let xref = Xref::new_stream(entries, indirect_obj.index, indirect_obj.generation);
    Ok((remainder, xref))
}

/// Parse either a xref stream or xref table.
#[tracable_parser]
pub fn xref(input: Span) -> CbParseResult<Xref> {
    branch::alt((xref_section, combinator::into(xref_stream)))(input)
}

/// Parse the End-Of-File marker and removes it from the end of the input.
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
