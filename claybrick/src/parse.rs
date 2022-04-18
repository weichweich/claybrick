use nom::{bytes, character, error::ParseError, IResult, InputIter, InputLength, InputTake, Parser};
use nom_locate::LocatedSpan;
use nom_tracable::{tracable_parser, TracableInfo};

use crate::pdf::{PdfSection, RawPdf};

use self::{
    error::{CbParseError, CbParseErrorKind},
    object::{indirect_object, object},
    object_stream::object_stream,
    trailer::trailer_tail,
};

pub use self::xref::{eof_marker_tail, startxref_tail, xref};

pub mod error;
pub(crate) mod object;
mod object_stream;
mod trailer;
mod xref;

pub type Span<'a> = LocatedSpan<&'a [u8], TracableInfo>;
type CbParseResult<'a, O> = IResult<Span<'a>, O, error::CbParseError<Span<'a>>>;

#[tracable_parser]
fn version(input: Span) -> CbParseResult<(u8, u8)> {
    let (remainder, _) = bytes::complete::tag_no_case("%PDF-")(input)?;
    let (remainder, major) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::char('.')(remainder)?;
    let (remainder, minor) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, (major, minor)))
}

#[tracable_parser]
fn comment(input: Span) -> CbParseResult<Span> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, _) = character::complete::char('%')(remainder)?;
    let (remainder, comment) = character::complete::not_line_ending(remainder)?;
    let (remainder, _) = character::complete::line_ending(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, comment))
}

#[tracable_parser]
fn binary_indicator(input: Span) -> CbParseResult<bool> {
    if let Ok((r, comment)) = comment(input) {
        if comment.len() > 3 && !comment.iter().any(|&d| d < 128) {
            Ok((r, true))
        } else {
            Ok((input, false))
        }
    } else {
        Ok((input, false))
    }
}

/// parse version and binary indicator comment.
#[tracable_parser]
pub(crate) fn header(input: Span) -> CbParseResult<((u8, u8), bool)> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, version) = version(remainder)?;
    let (remainder, announced_binary) = binary_indicator(remainder)?;

    Ok((remainder, (version, announced_binary)))
}

#[tracable_parser]
pub(crate) fn pdf_section(input: Span) -> CbParseResult<Vec<PdfSection>> {
    // find start of the xref section and trailer
    let (remainder_xref, _) = xref::eof_marker_tail(input)?;
    let (remainder_xref, startxref) = xref::startxref_tail(remainder_xref)?;

    let mut pdf_sections: Vec<PdfSection> = Vec::with_capacity(5);
    let mut maybe_startxref: Option<usize> = Some(startxref);

    while let Some(startxref) = maybe_startxref.take() {
        log::debug!("Parse section {}", startxref);

        let trailer = trailer_tail(remainder_xref)
            .map_err(|err| match err {
                nom::Err::Error(CbParseError {
                    kind: CbParseErrorKind::BackwardSearchNotFound,
                    ..
                }) => log::debug!("No trailer in PDF section"),
                _ => log::error!("Error in trailer {:?}", err),
            })
            .ok()
            .map(|(_, trailer)| trailer);
        let (remainder_xref, _) = nom::bytes::complete::take(startxref)(input)?;
        let (_, xref) = xref::xref(remainder_xref)?;

        let object_count = xref.used_objects().count();
        let mut objects = fnv::FnvHashMap::with_capacity_and_hasher(object_count, Default::default());

        for obj_xref in xref.used_objects() {
            // we always use input since the byte_offset is from the start of the file
            log::debug!("Parse object {:?}", obj_xref);
            let (obj_bytes, _) = bytes::complete::take(obj_xref.byte_offset)(input)?;
            let (_, obj) = indirect_object(obj_bytes)?;

            objects.insert(obj_xref.number, obj);
        }

        // TODO: read compressed objects
        for obj_xref in xref.compressed_objects() {
            let obj = objects.get(&obj_xref.number).expect("FIXME: missing stream object");
            let stream = obj
                .indirect()
                .expect("FIXME: handle invalid object")
                .object
                .stream()
                .expect("FIXME: handle invalid object");

            for (number, obj) in object_stream(stream).expect("FIXME: handle error") {
                objects.insert(number, obj);
            }
        }

        // The filter ensures that each new section is before the current one, thus
        // preventing a loop.
        maybe_startxref = trailer.as_ref().and_then(|t| t.previous).filter(|&new| new < startxref);
        pdf_sections.push(PdfSection { objects, xref, trailer });
    }

    Ok((remainder_xref, pdf_sections))
}

#[tracable_parser]
pub(crate) fn parse_complete(input: Span) -> CbParseResult<RawPdf> {
    let (_, (version, announced_binary)) = header(input)?;

    let (_, sections) = pdf_section(input)?;

    Ok((
        input,
        RawPdf {
            version,
            announced_binary,
            sections,
        },
    ))
}

/// Applies the supplied parser to the end of the input. Returns the beginning
/// of the input that wasn't recognized and the output of the supplied parser.
pub(crate) fn backward_search<P, Input, O, Error: ParseError<Input>>(
    limit: usize,
    mut parser: P,
) -> impl FnMut(Input) -> IResult<Input, (Input, O), CbParseError<Input>>
where
    Input: InputIter + InputTake + InputLength + Copy,
    P: Parser<Input, O, Error>,
{
    move |input: Input| {
        for i in 1..=input.input_len().min(limit) {
            let (end, start) = bytes::complete::take(input.input_len() - i)(input)?;
            let res = parser.parse(end);
            if let Ok(res) = res {
                return Ok((start, res));
            }
        }
        Err(nom::Err::Error(CbParseError::new(
            input,
            CbParseErrorKind::BackwardSearchNotFound,
        )))
    }
}

#[cfg(test)]
mod tests {
    use nom::AsBytes;
    use nom_tracable::TracableInfo;

    use super::*;

    #[test]
    fn test_backward_search() {
        let input = &b"Hello World!"[..];

        let res = backward_search::<_, _, _, CbParseError<&[u8]>>(6, nom::bytes::complete::tag(b"World"))(input);
        assert_eq!(res, Ok((&b"Hello "[..], (&b"!"[..], &b"World"[..]))));

        let res = backward_search::<_, _, _, CbParseError<&[u8]>>(5, nom::bytes::complete::tag(b"World"))(input);
        assert_eq!(
            res,
            Err(nom::Err::Error(CbParseError::new(
                input,
                CbParseErrorKind::BackwardSearchNotFound
            )))
        );
    }

    #[test]
    fn test_parse_version() {
        let info = TracableInfo::new().forward(true).backward(true);
        let input = LocatedSpan::new_extra(b"%PDF-1.8".as_bytes(), info);

        assert_eq!((1, 8), version(input).unwrap().1);
    }

    #[test]
    fn test_parse_binary_indicator() {
        let info = TracableInfo::new().forward(true).backward(true);
        let input = LocatedSpan::new_extra(b"%\xbf\xbf\xbf\xbf\xbf\n".as_bytes(), info);

        assert!(binary_indicator(input).unwrap().1);
    }
}
