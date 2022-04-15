use crate::{pdf::Array, writer::Encoder};

use crate::simple_encode::SimpleEncoder;

impl Encoder<Array> for SimpleEncoder {
    fn write_to(array: &Array, writer: &mut dyn crate::writer::Writer) {
        writer.write(b"[");
        for (i, item) in array.iter().enumerate() {
            if i != 0 {
                writer.write(b" ");
            }
            Self::write_to(item, writer);
        }
        writer.write(b"]");
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::Object;

    use super::*;

    #[test]
    fn empty_array() {
        let array = Array::from(vec![]);
        let encoded_len = SimpleEncoder::encoded_len(&array);
        assert_eq!(encoded_len, 2);

        let mut out = Vec::new();
        SimpleEncoder::write_to(&array, &mut out);
        let expected = b"[]";
        assert_eq!(expected, &out[..]);
        assert_eq!(encoded_len, out.len())
    }

    #[test]
    fn array_with_numbers() {
        let array = Array::from(vec![Object::Integer(0), Object::Integer(1), Object::Integer(2)]);
        let encoded_len = SimpleEncoder::encoded_len(&array);
        assert_eq!(encoded_len, 7);

        let mut out = Vec::new();
        SimpleEncoder::write_to(&array, &mut out);
        let expected = b"[0 1 2]";
        assert_eq!(expected, &out[..]);
        assert_eq!(encoded_len, out.len())
    }
}
