use nom::{
    branch,
    bytes::{self, complete::take},
    character,
    combinator::{self, into},
    multi, number, sequence,
};
use nom_tracable::tracable_parser;

use crate::{
    parse::{comment, Span},
    pdf::{Array, Dictionary, IndirectObject, Name, Object, Reference, Stream},
};

use super::{
    error::{CbParseError, CbParseErrorKind},
    CbParseResult,
};

pub(crate) const TRUE_OBJECT: &str = "true";
pub(crate) const FALSE_OBJECT: &str = "false";
pub(crate) const NULL_OBJECT: &str = "null";

pub(crate) fn is_delimiter(chr: u8) -> bool {
    matches!(chr, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

pub(crate) fn is_regular(chr: u8) -> bool {
    !is_delimiter(chr) && !chr.is_ascii_whitespace()
}

/// Consume all whitespace. If input doesn't start with a whitespace, peek the
/// next char and require it to be a delimiter.
#[tracable_parser]
fn require_termination(input: Span) -> CbParseResult<()> {
    let (remainder, whitespace) = character::complete::multispace0(input)?;
    if whitespace.is_empty() && !input.is_empty() {
        // TODO: there has to be a better way to require one char that fullfils a
        // condition?
        bytes::complete::take_while_m_n(1, 1, is_delimiter)(remainder)?;
    }
    Ok((remainder, ()))
}

fn consume_until_parenthesis(input: Span) -> Span {
    bytes::complete::escaped::<_, (), _, _, _, _>(
        character::complete::none_of("\\()"),
        '\\',
        character::complete::anychar,
    )(input)
    .map(|res| res.0)
    .unwrap_or(input)
}

#[tracable_parser]
fn consume_string_content(input: Span) -> CbParseResult<()> {
    let mut open_parathesis = 0;
    let mut remainder = input;

    while open_parathesis >= 0 {
        remainder = consume_until_parenthesis(remainder);

        if let Ok((r, open_close)) = branch::alt::<_, _, (), _>((
            combinator::value(-1, character::complete::char(')')),
            combinator::value(1, character::complete::char('(')),
        ))(remainder)
        {
            open_parathesis += open_close;
            // we don't want to consume the ')' that terminates the string.
            if open_parathesis >= 0 {
                remainder = r;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((remainder, ()))
}

fn hex_char_to_nibble(c: u8) -> Option<u8> {
    match c {
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'0'..=b'9' => Some(c - b'0'),
        _ => None,
    }
}

/// Expect that all input chars are in the range of a..=f, A..=F, 0..=9
fn hex_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() / 2 + input.len() % 2);
    for s in input.chunks_exact(2) {
        out.push((hex_char_to_nibble(s[0])? << 4) + hex_char_to_nibble(s[1])?);
    }

    // if there is a remainder the last nibble is zero
    if let Some(&r) = input.chunks_exact(2).remainder().get(0) {
        out.push(r << 4);
    }

    Some(out)
}

#[tracable_parser]
pub(crate) fn hex_string_object(input: Span) -> CbParseResult<Object> {
    let (remainder, content) = sequence::delimited(
        character::complete::char('<'),
        character::complete::hex_digit1,
        character::complete::char('>'),
    )(input)?;

    let bytes =
        hex_decode(content.fragment()).expect("We checked the content and made sure it only contains hex chars.");

    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, Object::HexString(bytes.into())))
}

#[tracable_parser]
pub(crate) fn string_object(input: Span) -> CbParseResult<Object> {
    let (remainder, content) = sequence::delimited(
        character::complete::char('('),
        combinator::recognize(consume_string_content),
        character::complete::char(')'),
    )(input)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, Object::String(content.to_vec().into())))
}

#[tracable_parser]
pub(crate) fn bool_object(input: Span) -> CbParseResult<Object> {
    let (remainder, obj) = branch::alt((
        combinator::value(Object::Bool(true), bytes::complete::tag(TRUE_OBJECT)),
        combinator::value(Object::Bool(false), bytes::complete::tag(FALSE_OBJECT)),
    ))(input)?;

    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, obj))
}

#[tracable_parser]
pub(crate) fn number_object(input: Span) -> CbParseResult<Object> {
    branch::alt((
        combinator::map(
            sequence::terminated(character::complete::i32, require_termination),
            Object::from,
        ),
        combinator::map(
            sequence::terminated(number::complete::float, require_termination),
            Object::from,
        ),
    ))(input)
}

#[tracable_parser]
pub(crate) fn null_object(input: Span) -> CbParseResult<Object> {
    let (remainder, _) = bytes::complete::tag(NULL_OBJECT)(input)?;
    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, Object::Null))
}

#[tracable_parser]
pub(crate) fn name_object(input: Span) -> CbParseResult<Name> {
    let (remainder, _) = character::complete::char('/')(input)?;
    let (remainder, name) = bytes::complete::take_while(is_regular)(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    let mut out = Vec::<u8>::with_capacity(name.len());
    let mut i = 0;
    while i < name.len() {
        match name[i] {
            b'#' => {
                let hex = name
                    .get(i + 1..=i + 2)
                    .ok_or_else(|| nom::Err::Error(CbParseError::new(input, CbParseErrorKind::InvalidName)))?;
                let nibbles = hex_char_to_nibble(hex[0])
                    .zip(hex_char_to_nibble(hex[1]))
                    .ok_or_else(|| nom::Err::Error(CbParseError::new(input, CbParseErrorKind::InvalidName)))?;
                out.push((nibbles.0 << 4) + nibbles.1);
                i += 3;
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }

    Ok((remainder, out.into()))
}

#[tracable_parser]
pub(crate) fn dictionary_entry(input: Span) -> CbParseResult<(Name, Object)> {
    let (remainder, name) = name_object(input)?;
    let (remainder, obj) = object(remainder)?;
    let (remainder, _) = multi::many0(comment)(remainder)?;

    Ok((remainder, (name, obj)))
}

#[tracable_parser]
pub(crate) fn dictionary_object(input: Span) -> CbParseResult<Dictionary> {
    let (remainder, map) = sequence::delimited(
        sequence::terminated(bytes::complete::tag(b"<<"), character::complete::multispace0),
        multi::fold_many0(dictionary_entry, Dictionary::new, |mut acc, (name, obj)| {
            acc.insert(name, obj);
            acc
        }),
        bytes::complete::tag(b">>"),
    )(input)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, map))
}

#[tracable_parser]
pub(crate) fn array_object(input: Span) -> CbParseResult<Array> {
    let (remainder, array) = sequence::delimited(
        sequence::pair(character::complete::char('['), character::complete::multispace0),
        multi::fold_many0(object, Array::new, |mut acc, obj| {
            acc.push(obj);
            acc
        }),
        character::complete::char(']'),
    )(input)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, array))
}

/// Get the stream content using the provided length.
fn stream_by_length(length: usize, input: Span) -> CbParseResult<Vec<u8>> {
    let (remainder, data) = combinator::map(take(length), |b: Span| b.to_vec())(input)?;
    let remainder = character::complete::line_ending::<_, CbParseError<Span>>(remainder)
        .map(|(r, _)| r)
        .unwrap_or(remainder);
    let (remainder, _) = bytes::complete::tag(b"endstream")(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, data))
}

/// Get the stream content by searching for the `endstream` keyword. This is a
/// fallback incase the stream length was invalid.
#[tracable_parser]
fn stream_by_keyword(input: Span) -> CbParseResult<Vec<u8>> {
    log::warn!("Using fallback stream content parser.");

    let (remainder, data) =
        combinator::map(bytes::complete::take_until(&b"endstream"[..]), |b: Span| b.to_vec())(input)?;
    let (remainder, _) = bytes::complete::tag(b"endstream")(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, data))
}

#[tracable_parser]
pub(crate) fn stream_object(input: Span) -> CbParseResult<Stream> {
    let (remainder, dict) = dictionary_object(input)?;

    let (remainder, _) = bytes::complete::tag(b"stream")(remainder)?;
    // stream keyword must not be followed by \r only because that would prevent
    // streams from beginning with \n.
    let (remainder, _) = branch::alt((bytes::complete::tag("\r\n"), bytes::complete::tag("\n")))(remainder)?;

    let length = match dict.get(&b"Length"[..]) {
        Some(Object::Integer(length)) => *length,
        l => {
            log::warn!("ignoring length object: {:?}", l);
            0
        }
    };

    // FIXME: handle huge streams
    let (remainder, data) =
        stream_by_length(usize::try_from(length).unwrap(), remainder).or_else(|_| stream_by_keyword(remainder))?;

    Ok((
        remainder,
        Stream {
            dictionary: dict,
            data: data.into(),
        },
    ))
}

pub(crate) fn referred_object<'a>(index: u32, generation: u32) -> impl FnMut(Span<'a>) -> CbParseResult<'a, Object> {
    combinator::map(
        sequence::delimited(
            sequence::terminated(bytes::complete::tag(b"obj"), character::complete::multispace0),
            branch::alt((into(stream_object), object)),
            sequence::terminated(bytes::complete::tag(b"endobj"), require_termination),
        ),
        move |obj| {
            Object::Indirect(IndirectObject {
                index,
                generation,
                object: Box::new(obj),
            })
        },
    )
}

pub(crate) fn reference_object<'a>(index: u32, generation: u32) -> impl FnMut(Span<'a>) -> CbParseResult<'a, Object> {
    combinator::map(
        sequence::terminated(character::complete::char('R'), require_termination),
        move |_| Object::Reference(Reference { index, generation }),
    )
}

#[tracable_parser]
pub(crate) fn indirect_object(input: Span) -> CbParseResult<Object> {
    let (remainder, index) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, generation) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;

    branch::alt((reference_object(index, generation), referred_object(index, generation)))(remainder)
}

#[tracable_parser]
pub(crate) fn object(input: Span) -> CbParseResult<Object> {
    // The order is important!
    branch::alt((
        into(dictionary_object),
        into(array_object),
        string_object,
        // indirect object has to be tested before we try to parse an integer.
        // `0 0 R` is an indirect object while `0 0` are two integers.
        indirect_object,
        number_object,
        bool_object,
        null_object,
        hex_string_object,
        into(name_object),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::AsBytes;
    use std::collections::HashMap;

    use crate::pdf::Reference;

    use super::*;

    #[test]
    pub fn test_termination() {
        assert_eq!(
            require_termination(b"  asdf".as_bytes().into()).unwrap().0.fragment(),
            &b"asdf".as_bytes()
        );
        assert_eq!(
            require_termination(b"".as_bytes().into()).unwrap().0.fragment(),
            &b"".as_bytes()
        );
        assert_eq!(
            require_termination(b"(".as_bytes().into()).unwrap().0.fragment(),
            &b"(".as_bytes()
        );
        assert!(require_termination(b"asdf".as_bytes().into()).is_err());
    }

    #[test]
    pub fn test_consume_until_parenthesis() {
        assert_eq!(
            consume_until_parenthesis(r"aasd(sadf".as_bytes().into()).fragment(),
            &"(sadf".as_bytes()
        );
        assert_eq!(
            consume_until_parenthesis(r"aasd\(asd(".as_bytes().into()).fragment(),
            &"(".as_bytes()
        );
        assert_eq!(
            consume_until_parenthesis(r")".as_bytes().into()).fragment(),
            &")".as_bytes()
        );
    }

    #[test]
    pub fn test_bool_object() {
        assert_eq!(object(b"true ".as_bytes().into()).unwrap().1, Object::Bool(true));
        assert_eq!(object(b"false ".as_bytes().into()).unwrap().1, Object::Bool(false));
        assert_eq!(
            object(b"false%a-comment".as_bytes().into()).unwrap().1,
            Object::Bool(false)
        );
        assert!(object(b"falsee".as_bytes().into()).is_err());
        assert!(object(b"afalse".as_bytes().into()).is_err());
    }

    #[test]
    pub fn test_integer_object() {
        assert_eq!(object(b"123".as_bytes().into()).unwrap().1, Object::Integer(123));
        assert_eq!(object(b"-123".as_bytes().into()).unwrap().1, Object::Integer(-123));
        assert_eq!(object(b"+123".as_bytes().into()).unwrap().1, Object::Integer(123));
        assert_eq!(
            object(b"-123%a-comment".as_bytes().into()).unwrap().1,
            Object::Integer(-123)
        );
    }

    #[test]
    pub fn test_float_object() {
        assert_eq!(object(b"123.123 ".as_bytes().into()).unwrap().1, Object::Float(123.123));
        assert_eq!(
            object(b"-123.123 ".as_bytes().into()).unwrap().1,
            Object::Float(-123.123)
        );
        assert_eq!(
            object(b"-123.123%a-comment".as_bytes().into()).unwrap().1,
            Object::Float(-123.123)
        );
        assert!(object(b"d123.123 ".as_bytes().into()).is_err());
        assert!(object(b"-1c23.123 ".as_bytes().into()).is_err());
    }

    #[test]
    pub fn test_string_object() {
        assert_eq!(
            object(b"()\n".as_bytes().into()).unwrap().1,
            Object::String(b"".to_vec().into())
        );
        assert_eq!(
            object(b"(a) ".as_bytes().into()).unwrap().1,
            Object::String(b"a".to_vec().into())
        );
        assert_eq!(
            object(b"((a)) ".as_bytes().into()).unwrap().1,
            Object::String(b"(a)".to_vec().into())
        );
        assert_eq!(
            object(br"((\(a)) ".as_bytes().into()).unwrap().1,
            Object::String(br"(\(a)".to_vec().into())
        );
        assert_eq!(
            object(br"(a\)\)\)) ".as_bytes().into()).unwrap().1,
            Object::String(br"a\)\)\)".to_vec().into())
        );
        assert_eq!(
            object(b"(123\\nmnbvcx)\n".as_bytes().into()).unwrap().1,
            Object::String(b"123\\nmnbvcx".to_vec().into())
        );
    }

    #[test]
    fn test_hex_to_nibble() {
        assert_eq!(hex_char_to_nibble(b'f'), Some(15));
        assert_eq!(hex_char_to_nibble(b'F'), Some(15));
        assert_eq!(hex_char_to_nibble(b'0'), Some(0));
        assert_eq!(hex_char_to_nibble(b'5'), Some(5));
    }

    #[test]
    pub fn test_hex_decode() {
        assert_eq!(
            hex_decode(b"FFFFFFFFFFFF".as_bytes()),
            Some(b"\xFF\xFF\xFF\xFF\xFF\xFF".to_vec())
        )
    }

    #[test]
    pub fn test_hex_string_object() {
        assert_eq!(
            object(b"<FFFFFFFFFFFF>".as_bytes().into()).unwrap().1,
            Object::HexString(b"\xFF\xFF\xFF\xFF\xFF\xFF".to_vec().into())
        )
    }

    #[test]
    pub fn test_null_object() {
        assert_eq!(object("null\n".as_bytes().into()).unwrap().1, Object::Null);
    }

    #[test]
    pub fn test_name_object() {
        let pairs = vec![
            (b"/lime#20Green".as_bytes(), b"lime Green".as_bytes()),
            (
                b"/paired#28#29parentheses".as_bytes(),
                b"paired()parentheses".as_bytes(),
            ),
            (b"/The_Key_of_F#23_Minor".as_bytes(), b"The_Key_of_F#_Minor".as_bytes()),
            (b"/A#42".as_bytes(), b"AB".as_bytes()),
            (b"/Name1".as_bytes(), b"Name1".as_bytes()),
            (b"/ASomewhatLongerName".as_bytes(), b"ASomewhatLongerName".as_bytes()),
            (
                b"/A;Name_With-Various***Characters?".as_bytes(),
                b"A;Name_With-Various***Characters?".as_bytes(),
            ),
            (b"/1.2".as_bytes(), b"1.2".as_bytes()),
            (b"/$$".as_bytes(), b"$$".as_bytes()),
            (b"/@pattern".as_bytes(), b"@pattern".as_bytes()),
            (b"/.notdef".as_bytes(), b".notdef".as_bytes()),
        ];

        for (input, expected) in pairs {
            let out = name_object(input.into());
            assert!(out.is_ok(), "Error while parsing `{}`", String::from_utf8_lossy(input));
            let out = out.unwrap().1;
            assert_eq!(
                &out[..],
                expected,
                "Failed to decode name `{}`. Expected `{}`, got `{}`.",
                String::from_utf8_lossy(input),
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&out)
            );
        }
    }

    #[test]
    pub fn test_dictionary() {
        let obj = Object::Dictionary(HashMap::from([(b"Length".to_vec().into(), Object::Integer(93))]));
        assert_eq!(object(b"<< /Length 93 >>".as_bytes().into()).unwrap().1, obj);

        let obj = Object::Dictionary(HashMap::from([
            (b"Type".to_vec().into(), Object::Name(b"Example".to_vec().into())),
            (
                b"Subtype".to_vec().into(),
                Object::Name(b"DictionaryExample".to_vec().into()),
            ),
            (b"Version".to_vec().into(), Object::Float(0.01)),
            (b"IntegerItem".to_vec().into(), Object::Integer(12)),
            (
                b"StringItem".to_vec().into(),
                Object::String(b"a string".to_vec().into()),
            ),
            (
                b"Subdictionary".to_vec().into(),
                Object::Dictionary(HashMap::from([
                    (b"Item2".to_vec().into(), Object::Bool(true)),
                    (b"Item2".to_vec().into(), Object::Bool(true)),
                ])),
            ),
        ]));
        assert_eq!(
            object(
                b"<< /Type /Example
        /Subtype /DictionaryExample
        /Version 0.01%A COMMENT
        /IntegerItem 12
        /StringItem (a string)
        /Subdictionary <<
        /Item2 true
        >>
        >>"
                .as_bytes()
                .into()
            )
            .unwrap()
            .1,
            obj
        );
    }

    #[test]
    pub fn test_array_object() {
        assert_eq!(
            object(b"[549 1.234 false (Ralph) /SomeName]".as_bytes().into())
                .unwrap()
                .1,
            Object::Array(Array::from(vec![
                Object::Integer(549),
                Object::Float(1.234),
                Object::Bool(false),
                Object::String(b"Ralph".to_vec().into()),
                Object::Name(b"SomeName".to_vec().into())
            ]))
        );
        assert_eq!(object(b"[]".as_bytes().into()).unwrap().1, Object::Array(Array::new()));
        assert_eq!(
            object(b"[459]".as_bytes().into()).unwrap().1,
            Object::Array(Array::from(vec![Object::Integer(459)]))
        );
        assert_eq!(
            object(b"[false]".as_bytes().into()).unwrap().1,
            Object::Array(Array::from(vec![Object::Bool(false)]))
        );
    }

    #[test]
    pub fn test_indirect_object() {
        assert_eq!(
            object(b"0 0 obj null endobj".as_bytes().into()).unwrap().1,
            Object::Indirect(IndirectObject {
                index: 0,
                generation: 0,
                object: Box::new(Object::Null)
            })
        );
        assert_eq!(
            object(
                b"1 0 obj
            << /Type /Catalog
               /Pages 2 0 R
            >>
            endobj"
                    .as_bytes()
                    .into()
            )
            .unwrap()
            .1,
            Object::Indirect(IndirectObject {
                index: 1,
                generation: 0,
                object: Box::new(Object::Dictionary(Dictionary::from([
                    (
                        b"Pages".to_vec().into(),
                        Object::Reference(Reference {
                            index: 2,
                            generation: 0
                        })
                    ),
                    (b"Type".to_vec().into(), Object::Name(b"Catalog".to_vec().into()))
                ])))
            })
        )
    }

    #[test]
    pub fn test_reference_object() {
        assert_eq!(
            object("0 0 R".as_bytes().into()).unwrap().1,
            Object::Reference(Reference {
                index: 0,
                generation: 0,
            })
        );
    }

    #[test]
    pub fn test_stream() {
        let stream = stream_object(
            b"<< /Length 93 >>
stream
/DeviceRGB cs /DeviceRGB CS
0 0 0.972549 SC
21.68 194 136.64 26 re
10 10 m 20 20 l S
/Im0 Do
endstream"
                .as_bytes()
                .into(),
        );
        assert!(
            matches!(stream, Ok((_, Stream { .. }))),
            "Expected OK got: {:?}",
            stream
        );
    }

    #[test]
    pub fn test_stream_line_feed_start() {
        let stream = stream_object(
            b"<< /Length 94 >>
stream\r\n\n\n/DeviceRGB cs /DeviceRGB CS
0 0 0.972549 SC
21.68 194 136.64 26 re
10 10 m 20 20 l S
/Im0 Do
endstream"
                .as_bytes()
                .into(),
        );
        assert!(
            matches!(stream, Ok((_, Stream { .. }))),
            "Expected OK got: {:?}",
            stream
        );
    }

    #[test]
    fn test_object_00() {
        let parsed_obj = object(
            b"8784 0 obj <</Linearized 1/L 6962693/O 8787/E 131293/N 768/T 6954970/H [ 2799 5432]>>\rendobj"
                .as_bytes()
                .into(),
        )
        .unwrap()
        .1;
        assert!(
            matches!(
                parsed_obj,
                Object::Indirect(IndirectObject {
                    index: 8784,
                    generation: 0,
                    object: _
                })
            ),
            "Unexpected parsing result: {:#?}",
            parsed_obj
        );
    }

    #[test]
    fn test_object_01() {
        let parsed_obj = object(
            b"20 0 obj
<</Author<FEFF004A006F0073002000760061006E002000640065006E0020004F0065007600650072>
/Creator<FEFF005700720069007400650072>
/Producer<FEFF004C0069006200720065004F0066006600690063006500200035002E0033>
/CreationDate(D:20170913090857+02'00')>>
endobj
"
            .as_bytes()
            .into(),
        )
        .unwrap()
        .1;
        assert!(
            matches!(
                parsed_obj,
                Object::Indirect(IndirectObject {
                    index: 20,
                    generation: 0,
                    object: _
                })
            ),
            "Unexpected parsing result: {:#?}",
            parsed_obj
        );
    }

    #[test]
    fn test_object_02() {
        let parsed_obj = object(
            b"4 0 obj
<</Type/Font/Subtype/CIDFontType2/BaseFont/TBSXET+Arial/CIDSystemInfo<</Registry(Adobe)/Ordering(Identity)/Supplement 0>>/FontDescriptor 3 0 R/CIDToGIDMap/Identity/W[0[750 277]2 3 722 5 6 556 7[333 500 222 500 556 277 610 222 556 500]17 18 556 19[277 556 583 277]23 30 556 31[610 556 277 833]35 36 556 37[500 722 666 943 722 777]43 44 666 45[556 277 833 666 777 556 610 556 610 722 500 556 333 666 333 277]61 62 722 63 64 583 65[722 583 666 500 666 500 556]72 73 277 74[222]75 76 333 77[556 1015 500 333 556]]>>
endobj"
            .as_bytes()
            .into(),
        )
        .unwrap()
        .1;
        assert!(
            matches!(
                parsed_obj,
                Object::Indirect(IndirectObject {
                    index: 4,
                    generation: 0,
                    object: _
                })
            ),
            "Unexpected parsing result: {:#?}",
            parsed_obj
        );
    }
}
