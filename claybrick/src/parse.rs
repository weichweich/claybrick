use nom::{bytes, character, error::ParseError, IResult, InputIter, InputLength, InputTake, Parser};
use nom_locate::LocatedSpan;
use nom_tracable::{tracable_parser, TracableInfo};

use crate::pdf::Pdf;

use self::error::{CbParseError, CbParseErrorKind};

pub mod error;
mod object;

type Span<'a> = LocatedSpan<&'a [u8], TracableInfo>;
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
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, version) = version(remainder)?;
    let (remainder, announced_binary) = binary_indicator(remainder)?;
    let (remainder, objects) = object::object0(remainder)?;

    Ok((
        remainder,
        Pdf {
            version,
            announced_binary,
            objects,
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

    use super::*;
    use crate::pdf::{Dictionary, IndirectObject, Object, Reference};

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

    #[test]
    fn test_parse() {
        let info = TracableInfo::new().forward(true).backward(true);
        let input = LocatedSpan::new_extra(
            b"%PDF-1.7
        %\xbf\xbf\xbf\xbf\xbf
        1 0 obj
        << /Type /Catalog
           /Pages 2 0 R
        >>
        endobj
        2 0 obj
        << /Kids [3 0 R]
           /Type /Pages
           /Count 1
        >>
        endobj"
                .as_bytes(),
            info,
        );

        assert_eq!(
            parse(input).unwrap().1,
            Pdf {
                version: (1, 7),
                announced_binary: true,
                objects: vec![
                    Object::Indirect(IndirectObject {
                        index: 1,
                        generation: 0,
                        object: Box::new(Object::Dictionary(Dictionary::from([
                            (b"Type".to_vec().into(), Object::Name(b"Catalog".to_vec().into())),
                            (
                                b"Pages".to_vec().into(),
                                Object::Reference(Reference {
                                    index: 2,
                                    generation: 0
                                })
                            )
                        ])))
                    }),
                    Object::Indirect(IndirectObject {
                        index: 2,
                        generation: 0,
                        object: Box::new(Object::Dictionary(Dictionary::from([
                            (
                                b"Kids".to_vec().into(),
                                Object::Array(
                                    vec![Object::Reference(Reference {
                                        index: 3,
                                        generation: 0
                                    })]
                                    .into()
                                )
                            ),
                            (b"Type".to_vec().into(), Object::Name(b"Pages".to_vec().into())),
                            (b"Count".to_vec().into(), Object::Integer(1))
                        ])))
                    })
                ]
            }
        )
    }
}
