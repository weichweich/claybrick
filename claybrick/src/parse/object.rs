use nom::{
    branch,
    bytes::{self, complete::take},
    character,
    combinator::{self, into},
    multi, number, sequence, IResult,
};

use crate::{
    parse::comment,
    pdf::{Array, Dictionary, IndirectObject, Name, Object, Reference},
};

const TRUE_OBJECT: &str = "true";
const FALSE_OBJECT: &str = "false";

fn is_delimiter(chr: u8) -> bool {
    matches!(chr, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

fn is_regular(chr: u8) -> bool {
    !is_delimiter(chr) && !chr.is_ascii_whitespace()
}

/// Consume all whitespace. If input doesn't start with a whitespace, peek the
/// next char and require it to be a delimiter.
fn require_termination(input: &[u8]) -> IResult<&[u8], ()> {
    let (remainder, whitespace) = character::complete::multispace0(input)?;
    if whitespace.is_empty() && !input.is_empty() {
        // TODO: there has to be a better way to require one char that fullfils a
        // condition?
        bytes::complete::take_while_m_n(1, 1, is_delimiter)(remainder)?;
    }
    Ok((remainder, ()))
}

fn consume_until_parenthesis(input: &[u8]) -> (&[u8], &[u8]) {
    bytes::complete::escaped::<_, (), _, _, _, _>(
        character::complete::none_of("\\()"),
        '\\',
        character::complete::anychar,
    )(input)
    .unwrap_or((input, b""))
}

fn consume_string_content(input: &[u8]) -> IResult<&[u8], ()> {
    let mut open_parathesis = 0;
    let mut remainder = input;

    while open_parathesis >= 0 {
        remainder = consume_until_parenthesis(remainder).0;

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

pub(crate) fn string_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, content) = sequence::delimited(
        character::complete::char('('),
        combinator::recognize(consume_string_content),
        character::complete::char(')'),
    )(input)?;

    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, Object::String(String::from_utf8(content.to_vec()).unwrap())))
}

pub(crate) fn bool_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, obj) = branch::alt((
        combinator::value(Object::Bool(true), bytes::complete::tag(TRUE_OBJECT)),
        combinator::value(Object::Bool(false), bytes::complete::tag(FALSE_OBJECT)),
    ))(input)?;

    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, obj))
}

pub(crate) fn number_object(input: &[u8]) -> IResult<&[u8], Object> {
    // TODO: accept optional `+` sign
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

pub(crate) fn null_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, _) = bytes::complete::tag(b"null")(input)?;
    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, Object::Null))
}

pub(crate) fn name_object(input: &[u8]) -> IResult<&[u8], Name> {
    let (remainder, _) = character::complete::char('/')(input)?;
    let (remainder, name) = bytes::complete::take_while(is_regular)(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    // TODO: parse name and replace all #XX with char

    Ok((remainder, name.to_vec().into()))
}

pub(crate) fn dictionary_entry(input: &[u8]) -> IResult<&[u8], (Name, Object)> {
    let (remainder, name) = name_object(input)?;
    let (remainder, obj) = object(remainder)?;
    let (remainder, _) = multi::many0(comment)(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, (name, obj)))
}

pub(crate) fn dictionary_object(input: &[u8]) -> IResult<&[u8], Dictionary> {
    let (remainder, map) = sequence::delimited(
        sequence::terminated(bytes::complete::tag(b"<<"), character::complete::multispace1),
        multi::fold_many0(dictionary_entry, Dictionary::new, |mut acc, (name, obj)| {
            acc.insert(name, obj);
            acc
        }),
        sequence::terminated(bytes::complete::tag(b">>"), require_termination),
    )(input)?;

    Ok((remainder, map))
}

pub fn array_object(input: &[u8]) -> IResult<&[u8], Array> {
    let (remainder, array) = sequence::delimited(
        character::complete::char('['),
        multi::fold_many0(object, Array::new, |mut acc, obj| {
            acc.push(obj);
            acc
        }),
        sequence::terminated(character::complete::char(']'), require_termination),
    )(input)?;

    Ok((remainder, array))
}

pub fn stream_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, dict) = dictionary_object(input)?;

    let (remainder, _) = bytes::complete::tag(b"stream")(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    let length = match dict.get(&b"Length".to_vec().into()) {
        Some(Object::Integer(length)) => *length,
        err => {
            println!("Err got {:?} as length (dict: {:?}", err, dict);
            todo!()
        }
    };

    let (remainder, data) = combinator::map(take(usize::try_from(length).unwrap()), |b: &[u8]| b.to_vec())(remainder)?;
    let (remainder, _) = bytes::complete::tag(b"endstream")(remainder)?;
    let (remainder, _) = require_termination(remainder)?;

    Ok((remainder, Object::Stream(dict, data)))
}

fn referred_object<'a>(index: u32, generation: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Object> {
    combinator::map(
        sequence::delimited(
            sequence::terminated(bytes::complete::tag(b"obj"), character::complete::multispace1),
            branch::alt((stream_object, object)),
            sequence::terminated(bytes::complete::tag(b"endobj"), require_termination),
        ),
        move |obj| {
            Object::IndirectObject(IndirectObject {
                index: index,
                generation: generation,
                object: Box::new(obj),
            })
        },
    )
}

fn reference_object<'a>(index: u32, generation: u32) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Object> {
    combinator::map(
        sequence::terminated(character::complete::char('R'), require_termination),
        move |_| {
            Object::Reference(Reference {
                index: index,
                generation: generation,
            })
        },
    )
}

pub(crate) fn indirect_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, index) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, generation) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;

    branch::alt((reference_object(index, generation), referred_object(index, generation)))(remainder)
}

pub(crate) fn object(input: &[u8]) -> IResult<&[u8], Object> {
    // The order is important!
    branch::alt((
        into(dictionary_object),
        into(array_object),
        string_object,
        // indirect object has to be tested before we try to parse an integer.
        // `0 0 R` is an inderect object while `0 0` are two integers.
        indirect_object,
        number_object,
        bool_object,
        null_object,
        into(name_object),
    ))(input)
}

pub(crate) fn object0(input: &[u8]) -> IResult<&[u8], Vec<Object>> {
    let (remainder, _) = character::complete::multispace0(input)?;
    multi::many0(object)(remainder)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nom::AsBytes;

    use crate::pdf::Reference;

    use super::*;

    #[test]
    pub fn test_termination() {
        assert_eq!(require_termination(b"  asdf"), Ok((&b"asdf"[..], ())));
        assert_eq!(require_termination(b""), Ok((&b""[..], ())));
        assert_eq!(require_termination(b"("), Ok((&b"("[..], ())));
        assert!(require_termination(b"asdf").is_err());
    }

    #[test]
    pub fn test_consume_until_parenthesis() {
        assert_eq!(
            consume_until_parenthesis(r"aasd(sadf".as_bytes()),
            ("(sadf".as_bytes(), r"aasd".as_bytes())
        );
        assert_eq!(
            consume_until_parenthesis(r"aasd\(asd(".as_bytes()),
            ("(".as_bytes(), r"aasd\(asd".as_bytes())
        );
        assert_eq!(
            consume_until_parenthesis(r")".as_bytes()),
            (")".as_bytes(), r"".as_bytes())
        );
    }

    #[test]
    pub fn test_bool_object() {
        let empty = &b""[..];
        assert_eq!(object(b"true "), Ok((empty, Object::Bool(true))));
        assert_eq!(object(b"false "), Ok((empty, Object::Bool(false))));
        assert_eq!(
            object(b"false%a-comment"),
            Ok((b"%a-comment".as_bytes(), Object::Bool(false)))
        );
        assert!(object(b"falsee").is_err());
        assert!(object(b"afalse").is_err());
    }

    #[test]
    pub fn test_integer_object() {
        let empty = &b""[..];
        assert_eq!(object(b"123"), Ok((empty, Object::Integer(123))));
        assert_eq!(object(b"-123"), Ok((empty, Object::Integer(-123))));
        assert_eq!(object(b"+123"), Ok((empty, Object::Integer(123))));
        assert_eq!(
            object(b"-123%a-comment"),
            Ok((b"%a-comment".as_bytes(), Object::Integer(-123)))
        );
    }

    #[test]
    pub fn test_float_object() {
        let empty = &b""[..];
        assert_eq!(object(b"123.123 "), Ok((empty, Object::Float(123.123))));
        assert_eq!(object(b"-123.123 "), Ok((empty, Object::Float(-123.123))));
        assert_eq!(
            object(b"-123.123%a-comment"),
            Ok((b"%a-comment".as_bytes(), Object::Float(-123.123)))
        );
        assert!(object(b"d123.123 ").is_err());
        assert!(object(b"-1c23.123 ").is_err());
    }

    #[test]
    pub fn test_string_object() {
        let empty = &b""[..];
        assert_eq!(object("()\n".as_bytes()), Ok((empty, Object::String("".to_owned()))));
        assert_eq!(object("(a) ".as_bytes()), Ok((empty, Object::String("a".to_owned()))));
        assert_eq!(
            object("((a)) ".as_bytes()),
            Ok((empty, Object::String("(a)".to_owned())))
        );
        assert_eq!(
            object(r"((\(a)) ".as_bytes()),
            Ok((empty, Object::String(r"(\(a)".to_owned())))
        );
        assert_eq!(
            object(r"(a\)\)\)) ".as_bytes()),
            Ok((empty, Object::String(r"a\)\)\)".to_owned())))
        );
        assert_eq!(
            object("(123\\nmnbvcx)\n".as_bytes()),
            Ok((empty, Object::String("123\\nmnbvcx".to_owned())))
        );
    }

    #[test]
    pub fn test_null_object() {
        let empty = &b""[..];
        assert_eq!(object("null\n".as_bytes()), Ok((empty, Object::Null)));
    }

    #[test]
    pub fn test_name_object() {
        assert!(object(b"/Name1").is_ok());
        assert!(object(b"/ASomewhatLongerName").is_ok());
        assert!(object(b"/A;Name_With-Various***Characters?").is_ok());
        assert!(object(b"/1.2").is_ok());
        assert!(object(b"/$$").is_ok());
        assert!(object(b"/@pattern").is_ok());
        assert!(object(b"/.notdef").is_ok());
        assert!(object(b"/lime#20Green").is_ok());
        assert!(object(b"/paired#28#29parentheses").is_ok());
        assert!(object(b"/The_Key_of_F#23_Minor").is_ok());
        assert!(object(b"/A#42").is_ok());
    }

    #[test]
    pub fn test_dictionary() {
        let empty = &b""[..];

        let obj = Object::Dictionary(HashMap::from([(b"Length".to_vec().into(), Object::Integer(93))]));
        assert_eq!(object(b"<< /Length 93 >>"), Ok((empty, obj)));

        let obj = Object::Dictionary(HashMap::from([
            (b"Type".to_vec().into(), Object::Name(b"Example".to_vec().into())),
            (
                b"Subtype".to_vec().into(),
                Object::Name(b"DictionaryExample".to_vec().into()),
            ),
            (b"Version".to_vec().into(), Object::Float(0.01)),
            (b"IntegerItem".to_vec().into(), Object::Integer(12)),
            (b"StringItem".to_vec().into(), Object::String("a string".to_string())),
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
            ),
            Ok((empty, obj))
        );
    }

    #[test]
    pub fn test_array_object() {
        let empty = &b""[..];
        assert_eq!(
            object(b"[549 3.14 false (Ralph) /SomeName]"),
            Ok((
                empty,
                Object::Array(Array::from([
                    Object::Integer(549),
                    Object::Float(3.14),
                    Object::Bool(false),
                    Object::String("Ralph".to_string()),
                    Object::Name(b"SomeName".to_vec().into())
                ]))
            ))
        );
        assert_eq!(object(b"[]"), Ok((empty, Object::Array(Array::from([])))));
        assert_eq!(
            object(b"[459]"),
            Ok((empty, Object::Array(Array::from([Object::Integer(459),]))))
        );
        assert_eq!(
            object(b"[false]"),
            Ok((empty, Object::Array(Array::from([Object::Bool(false),]))))
        );
    }

    #[test]
    pub fn test_indirect_object() {
        let empty = &b""[..];
        assert_eq!(
            object(b"0 0 obj null endobj"),
            Ok((
                empty,
                Object::IndirectObject(IndirectObject {
                    index: 0,
                    generation: 0,
                    object: Box::new(Object::Null)
                })
            ))
        );
        assert_eq!(
            object(
                b"1 0 obj
            << /Type /Catalog
               /Pages 2 0 R
            >>
            endobj"
            ),
            Ok((
                empty,
                Object::IndirectObject(IndirectObject {
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
            ))
        )
    }

    #[test]
    pub fn test_reference_object() {
        let empty = &b""[..];
        assert_eq!(
            object("0 0 R".as_bytes()),
            Ok((
                empty,
                Object::Reference(Reference {
                    index: 0,
                    generation: 0,
                })
            ))
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
endstream",
        );
        assert!(matches!(stream, Ok(_)), "Expected OK got: {:?}", stream);
    }

    #[test]
    pub fn test_object0() {
        let parsed_obj = object0(
            b"     1 0 obj
        << /Type /Catalog
           /Pages 2 0 R
        >>
        endobj
        2 0 obj
        << /Kids [3 0 R]
           /Type /Pages
           /Count 1
        >>
        endobj
        3 0 obj
        << /Contents 4 0 R
           /Type /Page
           /Resources << /XObject << /Im0 5 0 R >> >>
           /Parent 2 0 R
           /MediaBox [0 0 180 240]
        >>
        endobj",
        )
        .unwrap()
        .1;
        assert!(
            matches!(
                &parsed_obj[..],
                [
                    Object::IndirectObject(_),
                    Object::IndirectObject(_),
                    Object::IndirectObject(_),
                ]
            ),
            "Unexpected parsing result: {:#?}",
            parsed_obj
        );
    }
}
