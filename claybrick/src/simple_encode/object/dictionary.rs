use crate::{pdf::Dictionary, writer::Encoder};

use crate::simple_encode::SimpleEncoder;

impl Encoder<Dictionary> for SimpleEncoder {
    fn encoded_len(o: &Dictionary) -> usize {
        // 2 bytes for start and end each.
        let mut size = 4;
        size += o
            .iter()
            .map(|(n, o)| Self::encoded_len(n) + Self::encoded_len(o))
            .sum::<usize>();

        // For N entries we need N delimiters between key and value. We also need one
        // delimiter for each pair (N - 1). This leads to 2 * N - 1.
        size += (o.len() * 2).saturating_sub(1);

        size
    }

    fn write_to(o: &Dictionary, writer: &mut dyn crate::writer::Writer) {
        writer.write(b"<<");
        let mut is_first = true;
        for (key, value) in o.iter() {
            if !is_first {
                writer.write(b" ");
            }
            Self::write_to(key, writer);
            writer.write(b" ");
            Self::write_to(value, writer);
            is_first = false
        }
        writer.write(b">>");
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::Object;

    use super::*;

    #[test]
    fn empty_dict() {
        let d = Dictionary::new();
        let expected_len = SimpleEncoder::encoded_len(&d);
        let expected_output = b"<<>>";
        assert_eq!(expected_len, expected_output.len());

        let mut out = Vec::new();
        SimpleEncoder::write_to(&d, &mut out);
        assert_eq!(expected_output, &out[..]);
        assert_eq!(out.len(), expected_len);
    }

    #[test]
    fn filled_dict() {
        let mut d = Dictionary::new();
        d.insert(b"one".to_vec().into(), Object::Integer(1));
        d.insert(b"two".to_vec().into(), Object::Integer(2));
        d.insert(b"three".to_vec().into(), Object::Integer(3));

        let expected_len = SimpleEncoder::encoded_len(&d);
        let expected_output = br"<<\one 1 \two 2 \three 3>>";
        assert_eq!(expected_len, expected_output.len());

        let mut out = Vec::new();
        SimpleEncoder::write_to(&d, &mut out);
        // TODO: The order of the dictionary is not preserved or defined.
        // assert_eq!(
        //     expected_output,
        //     &out[..],
        //     "expected: {} got: {}",
        //     String::from_utf8_lossy(expected_output),
        //     String::from_utf8_lossy(&out[..])
        // );
        assert_eq!(out.len(), expected_len);
    }
}
