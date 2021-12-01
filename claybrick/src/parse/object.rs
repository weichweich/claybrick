use nom::{branch, bytes, character, combinator, multi, number, sequence, IResult};

use crate::pdf::{Dictionary, IndirectObject, Name, Object};

const TRUE_OBJECT: &str = "true";
const FALSE_OBJECT: &str = "false";

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

    let (remainder, _) = character::complete::multispace1(remainder)?;

    Ok((
        remainder,
        Object::String(String::from_utf8(content.to_vec()).unwrap()),
    ))
}

pub(crate) fn bool_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, (obj, _)) = sequence::pair(
        branch::alt((
            combinator::value(Object::Bool(true), bytes::complete::tag(TRUE_OBJECT)),
            combinator::value(Object::Bool(false), bytes::complete::tag(FALSE_OBJECT)),
        )),
        character::complete::multispace1,
    )(input)?;

    Ok((remainder, obj))
}

pub(crate) fn number_object(input: &[u8]) -> IResult<&[u8], Object> {
    branch::alt((
        combinator::map(
            sequence::terminated(character::complete::i32, character::complete::multispace1),
            Object::from,
        ),
        combinator::map(
            sequence::terminated(number::complete::float, character::complete::multispace1),
            Object::from,
        ),
    ))(input)
}

pub(crate) fn null_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, _) = bytes::complete::tag(b"null")(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;

    Ok((remainder, Object::Null))
}

pub(crate) fn name(input: &[u8]) -> IResult<&[u8], Name> {
    let (remainder, _) = character::complete::char('/')(input)?;
    let (remainder, name) = sequence::terminated(
        bytes::complete::take_till(|c| u8::is_ascii_whitespace(&c)),
        character::complete::multispace1,
    )(remainder)?;

    // TODO: parse name and replace all #XX with char

    Ok((remainder, name.to_vec()))
}

pub(crate) fn name_object(input: &[u8]) -> IResult<&[u8], Object> {
    combinator::map(name, Object::from)(input)
}

pub(crate) fn dictionary_entry(input: &[u8]) -> IResult<&[u8], (Name, Object)> {
    let (remainder, name) = name(input)?;
    let (remainder, obj) = object(remainder)?;

    Ok((remainder, (name, obj)))
}

pub(crate) fn dictionary_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, map) = sequence::delimited(
        sequence::terminated(
            bytes::complete::tag(b"<<"),
            character::complete::multispace1,
        ),
        multi::fold_many0(dictionary_entry, Dictionary::new, |mut acc, (name, obj)| {
            acc.insert(name, obj);
            acc
        }),
        sequence::terminated(
            bytes::complete::tag(b">>"),
            character::complete::multispace1,
        ),
    )(input)?;

    Ok((remainder, Object::Dictionary(map)))
}

pub(crate) fn indirect_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, index) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, generation) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, object) = sequence::delimited(
        sequence::terminated(
            bytes::complete::tag(b"obj"),
            character::complete::multispace1,
        ),
        // TODO: handle special case `R` for reference
        object,
        sequence::terminated(
            bytes::complete::tag(b"endobj"),
            character::complete::multispace1,
        ),
    )(remainder)?;

    Ok((
        remainder,
        Object::IndirectObject(IndirectObject {
            index: index,
            generation: generation,
            object: Box::new(object),
        }),
    ))
}

pub(crate) fn object(input: &[u8]) -> IResult<&[u8], Object> {
    branch::alt((
        string_object,
        bool_object,
        number_object,
        null_object,
        indirect_object,
        name_object,
        dictionary_object,
    ))(input)
}

pub(crate) fn object0(input: &[u8]) -> IResult<&[u8], Vec<Object>> {
    multi::many0(object)(input)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    pub fn test_bool_object() {
        let empty = &b""[..];
        assert_eq!(bool_object(b"true "), Ok((empty, Object::Bool(true))));
        assert_eq!(bool_object(b"false "), Ok((empty, Object::Bool(false))));
        assert!(bool_object(b"falsee").is_err());
        assert!(bool_object(b"afalse").is_err());
    }

    #[test]
    pub fn test_integer_object() {
        let empty = &b""[..];
        assert_eq!(number_object(b"123 "), Ok((empty, Object::Integer(123))));
        assert_eq!(number_object(b"-123 "), Ok((empty, Object::Integer(-123))));
    }

    #[test]
    pub fn test_float_object() {
        let empty = &b""[..];
        assert_eq!(
            number_object(b"123.123 "),
            Ok((empty, Object::Float(123.123)))
        );
        assert_eq!(
            number_object(b"-123.123 "),
            Ok((empty, Object::Float(-123.123)))
        );
        assert!(number_object(b"d123.123 ").is_err());
        assert!(number_object(b"-1c23.123 ").is_err());
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
    pub fn test_string_object() {
        let empty = &b""[..];
        assert_eq!(
            string_object("()\n".as_bytes()),
            Ok((empty, Object::String("".to_owned())))
        );
        assert_eq!(
            string_object("(a) ".as_bytes()),
            Ok((empty, Object::String("a".to_owned())))
        );
        assert_eq!(
            string_object("((a)) ".as_bytes()),
            Ok((empty, Object::String("(a)".to_owned())))
        );
        assert_eq!(
            string_object(r"((\(a)) ".as_bytes()),
            Ok((empty, Object::String(r"(\(a)".to_owned())))
        );
        assert_eq!(
            string_object(r"(a\)\)\)) ".as_bytes()),
            Ok((empty, Object::String(r"a\)\)\)".to_owned())))
        );
        assert_eq!(
            string_object("(123\\nmnbvcx)\n".as_bytes()),
            Ok((empty, Object::String("123\\nmnbvcx".to_owned())))
        );
    }

    #[test]
    pub fn test_null_object() {
        let empty = &b""[..];
        assert_eq!(null_object("null\n".as_bytes()), Ok((empty, Object::Null)));
    }

    #[test]
    pub fn test_name_object() {
        assert!(name_object(b"/Name1 ").is_ok());
        assert!(name_object(b"/ASomewhatLongerName ").is_ok());
        assert!(name_object(b"/A;Name_With-Various***Characters? ").is_ok());
        assert!(name_object(b"/1.2 ").is_ok());
        assert!(name_object(b"/$$ ").is_ok());
        assert!(name_object(b"/@pattern ").is_ok());
        assert!(name_object(b"/.notdef ").is_ok());
        assert!(name_object(b"/lime#20Green ").is_ok());
        assert!(name_object(b"/paired#28#29parentheses ").is_ok());
        assert!(name_object(b"/The_Key_of_F#23_Minor ").is_ok());
        assert!(name_object(b"/A#42 ").is_ok());
    }

    #[test]
    pub fn test_dictionary() {
        let empty = &b""[..];

        let obj = Object::Dictionary(HashMap::from([(b"Length".to_vec(), Object::Integer(93))]));
        assert_eq!(dictionary_object(b"<< /Length 93 >> "), Ok((empty, obj)));

        let obj = Object::Dictionary(HashMap::from([
            (b"Type".to_vec(), Object::Name(b"Example".to_vec())),
            (
                b"Subtype".to_vec(),
                Object::Name(b"DictionaryExample".to_vec()),
            ),
            (b"Version".to_vec(), Object::Float(0.01)),
            (b"IntegerItem".to_vec(), Object::Integer(12)),
            (
                b"StringItem".to_vec(),
                Object::String("a string".to_owned()),
            ),
            (
                b"Subdictionary".to_vec(),
                Object::Dictionary(HashMap::from([
                    (b"Item2".to_vec(), Object::Bool(true)),
                    (b"Item2".to_vec(), Object::Bool(true)),
                ])),
            ),
        ]));
        assert_eq!(
            dictionary_object(
                b"<< /Type /Example
        /Subtype /DictionaryExample
        /Version 0.01
        /IntegerItem 12
        /StringItem (a string)
        /Subdictionary <<
        /Item2 true
        >>
        >>
        "
            ),
            Ok((empty, obj))
        );
    }

    #[test]
    pub fn test_indirect_object() {
        let empty = &b""[..];
        assert_eq!(
            indirect_object("0 0 obj null endobj ".as_bytes()),
            Ok((
                empty,
                Object::IndirectObject(IndirectObject {
                    index: 0,
                    generation: 0,
                    object: Box::new(Object::Null)
                })
            ))
        );
    }
}
