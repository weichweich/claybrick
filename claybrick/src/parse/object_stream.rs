use nom::{bytes, character};

use crate::pdf::{
    document::{dict_types::OBJECT_STREAM, K_FIRST, K_LENGTH, K_STREAM_OBJECT_COUNT, K_TYPE},
    Object, Stream,
};

use super::{error::CbParseError, object::object, CbParseResult, Span};

fn parse_content(
    _length: usize,
    obj_count: usize,
    first_offset: usize,
    input: Span,
) -> CbParseResult<Vec<(usize, Object)>> {
    let mut remainder = input;
    let mut objects = Vec::with_capacity(obj_count);
    for _ in 0..obj_count {
        // Next object number and byte offset.
        let (r, obj_number) = character::complete::u32(remainder)?;
        let obj_number: usize = obj_number.try_into().expect("TODO: handle error");
        let (r, _) = character::complete::multispace1(r)?;
        let (r, byte_offset) = character::complete::u32(r)?;
        let byte_offset: usize = byte_offset.try_into().expect("TODO: handle error");
        // the last pair might not be followed by a whitespace
        let (r, _) = character::complete::multispace0(r)?;
        remainder = r;

        // parse object with number `obj_number` at position `first_offset +
        // byte_offset`.
        let (obj_bytes, _) = bytes::complete::take(first_offset + byte_offset)(input)?;
        let (_, obj) = object(obj_bytes)?;

        // add object to the output vector.
        objects.push((obj_number, obj));
    }

    Ok((remainder, objects))
}

pub(crate) fn object_stream(stream: &Stream) -> Result<Vec<(usize, Object)>, CbParseError<()>> {
    let dict = &stream.dictionary;
    dict.get(K_TYPE)
        .and_then(Object::name)
        .filter(|name| &name[..] == OBJECT_STREAM)
        .expect("FIXME: error for wrong type");
    let length: usize = dict
        .get(K_LENGTH)
        .and_then(Object::integer)
        .expect("FIXME: error for wrong length")
        .try_into()
        .expect("FIXME: error for invalid length");
    let obj_count: usize = dict
        .get(K_STREAM_OBJECT_COUNT)
        .and_then(Object::integer)
        .expect("FIXME: error for wrong count")
        .try_into()
        .expect("FIXME: error for invalid count");
    let first_offset: usize = dict
        .get(K_FIRST)
        .and_then(Object::integer)
        .expect("FIXME: error for wrong count")
        .try_into()
        .expect("FIXME: error for invalid count");

    let data = stream.filtered_data().expect("FIXME: error handling");

    let (_, objs) = parse_content(length, obj_count, first_offset, data[..].into()).expect("TODO: error handling");
    Ok(objs)
}

#[cfg(test)]
mod tests {
    use crate::pdf::{Bytes, Name};

    use super::*;

    #[test]
    fn test_object_stream_empty() {
        let input_stream = Stream {
            dictionary: [
                (Name::new(K_TYPE.into()), Object::from(Name::new(OBJECT_STREAM.into()))),
                (Name::new(K_STREAM_OBJECT_COUNT.into()), Object::Integer(0)),
                (Name::new(K_FIRST.into()), Object::Integer(0)),
                (Name::new(K_LENGTH.into()), Object::Integer(0)),
            ]
            .into(),
            data: b"".to_vec().into(),
        };

        assert_eq!(object_stream(&input_stream), Ok(vec![]))
    }

    #[test]
    fn test_object_stream_single() {
        let data: Bytes = b"123 0 999".to_vec().into();
        let input_stream = Stream {
            dictionary: [
                (Name::new(K_TYPE.into()), Object::from(Name::new(OBJECT_STREAM.into()))),
                (Name::new(K_STREAM_OBJECT_COUNT.into()), Object::Integer(1)),
                (Name::new(K_FIRST.into()), Object::Integer(6)),
                (
                    Name::new(K_LENGTH.into()),
                    Object::Integer(data.len().try_into().unwrap()),
                ),
            ]
            .into(),
            data,
        };

        assert_eq!(object_stream(&input_stream), Ok(vec![(123, Object::Integer(999))]))
    }
}
