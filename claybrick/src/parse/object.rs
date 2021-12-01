use nom::{branch, bytes, character, combinator, multi, number, sequence, IResult};

use crate::pdf::{Object, IndirectObject};

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
    let (remainder, (f, _)) =
        sequence::pair(number::complete::double, character::complete::multispace1)(input)?;

    Ok((remainder, f.into()))
}

pub(crate) fn null_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, _) = bytes::complete::tag(b"null")(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;

    Ok((remainder, Object::Null))
}

pub(crate) fn indirect_object(input: &[u8]) -> IResult<&[u8], Object> {
    let (remainder, index) = character::complete::u32(input)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, generation) = character::complete::u32(remainder)?;
    let (remainder, _) = character::complete::multispace1(remainder)?;
    let (remainder, object) = sequence::delimited(
        sequence::pair(
            bytes::complete::tag(b"obj"),
            character::complete::multispace1,
        ),
        object,
        sequence::pair(
            bytes::complete::tag(b"endobj"),
            character::complete::multispace1,
        ),
    )(remainder)?;

    Ok((remainder, Object::IndirectObject(IndirectObject {
        index: index,
        generation: generation,
        object: Box::new(object),
    })))
}

pub(crate) fn object(input: &[u8]) -> IResult<&[u8], Object> {
    branch::alt((
        string_object,
        bool_object,
        number_object,
        null_object,
        indirect_object,
    ))(input)
}

pub(crate) fn object0(input: &[u8]) -> IResult<&[u8], Vec<Object>> {
    multi::many0(object)(input)
}

#[cfg(test)]
mod tests {
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
