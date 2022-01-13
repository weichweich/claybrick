use nom::{bytes, character, error::ParseError, IResult, InputIter, InputLength, InputTake, Parser};
use nom_locate::LocatedSpan;
use nom_tracable::{tracable_parser, TracableInfo};

use crate::pdf::{Pdf, PdfSection, Trailer};

use self::{
    error::{CbParseError, CbParseErrorKind},
    trailer::trailer_tail,
};

pub use self::xref::{eof_marker_tail, startxref_tail, xref};

pub mod error;
mod object;
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

#[tracable_parser]
pub(crate) fn parse(input: Span) -> CbParseResult<Pdf> {
    // parse version and binary indicator comment. remainder_obj should contain
    // objects
    let (remainder_obj, _) = character::complete::multispace0(input)?;
    let (remainder_obj, version) = version(remainder_obj)?;
    let (remainder_obj, announced_binary) = binary_indicator(remainder_obj)?;
    // NOTE: no need to read all objects, we could parse xref table first.
    let (_, objects) = object::object0(remainder_obj)?;

    // find start of the xref section and trailer
    let (remainder_xref, _) = xref::eof_marker_tail(input)?;
    let (remainder_xref, startxref) = xref::startxref_tail(remainder_xref)?;
    let trailer = trailer_tail(remainder_xref)
        .ok()
        .map(|(_, trailer)| trailer)
        .map(|t| Trailer::try_from(t))
        .transpose()
        .map_err(|e| {
            log::error!("Invalid trailer: {:?}", e);
            ()
        })
        .unwrap();

    let (remainder_xref, _) = nom::bytes::complete::take(startxref)(input)?;
    let (_, xref) = xref::xref(remainder_xref)?;

    Ok((
        input,
        Pdf {
            version,
            announced_binary,
            sections: vec![PdfSection {
                objects,
                xref,
                trailer,
                startxref,
            }],
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

        assert_eq!((true), binary_indicator(input).unwrap().1);
    }
}
