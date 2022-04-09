use crate::{
    pdf::CbString,
    simple_encode::SimpleEncoder,
    writer::{Encoder, Writer},
};

impl Encoder<CbString> for SimpleEncoder {
    fn encoded_len(str: &CbString) -> usize {
        let mut len = str.len();
        let mut open_paranthesis = 0;
        let mut remaining_closing_paranthesis = str.iter().filter(|&c| *c == b')').count();

        // check for characters that we need to escape.
        for char in str.iter() {
            match (char, open_paranthesis, remaining_closing_paranthesis) {
                (b'(', _, 0) => {
                    len += 1;
                    open_paranthesis += 1;
                }
                (b'(', ..) => open_paranthesis += 1,
                // unbalanced closing paranthesis need to be escaped, they would otherwise determain the end of the
                // string
                (b')', 0, _) => {
                    len += 1;
                    remaining_closing_paranthesis -= 1;
                }
                (b')', ..) => {
                    open_paranthesis -= 1;
                    remaining_closing_paranthesis -= 1;
                }
                _ => {}
            }
        }

        // we need two additional bytes for the opening and closing paranthesis
        len + 2
    }

    fn write_to(str: &CbString, writer: &mut dyn Writer) {
        writer.write(&b"("[..]);

        let mut open_paranthesis: usize = 0;
        let mut remaining_closing_paranthesis = str.iter().filter(|&c| *c == b')').count();

        let mut last_written_index = 0;
        // check for characters that we need to escape.
        for (index, char) in str.iter().enumerate() {
            match (char, open_paranthesis, remaining_closing_paranthesis) {
                (b'(', _, 0) => {
                    open_paranthesis += 1;
                    writer.write(&str[last_written_index..index]);
                    writer.write(&br"\"[..]);
                    last_written_index = index;
                }
                (b'(', _, _) => open_paranthesis += 1,
                // unbalanced closing paranthesis need to be escaped, they would otherwise determain the end of the
                // string
                (b')', 0, _) => {
                    writer.write(&str[last_written_index..index]);
                    writer.write(&br"\"[..]);
                    last_written_index = index;
                    remaining_closing_paranthesis = remaining_closing_paranthesis.saturating_sub(1);
                }
                (b')', _, _) => {
                    open_paranthesis = open_paranthesis.saturating_sub(1);
                    remaining_closing_paranthesis = remaining_closing_paranthesis.saturating_sub(1);
                }
                // skip all others.
                _ => {}
            }
        }
        writer.write(&str[last_written_index..]);
        writer.write(&b")"[..]);
    }
}

#[cfg(test)]
mod tests {
    use crate::{pdf::CbString, simple_encode::SimpleEncoder, writer::Encoder};

    #[test]
    fn test_simple() {
        let simple = CbString::from(b"abcdefg".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, b"(abcdefg)".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_end_with_closing_paranthesis() {
        let simple = CbString::from(b"(abcdefg)".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, b"((abcdefg))".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_end_with_unmatched_closing_paranthesis() {
        let simple = CbString::from(b"abcdefg)".to_vec());

        // 2 for start and end. One for escaping.
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() + 3);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, br"(abcdefg\))".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_many_unmatched_closing_paranthesis() {
        let simple = CbString::from(b")))))))))".to_vec());

        // 2 for start and end. many for escaping.
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() * 2 + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, br"(\)\)\)\)\)\)\)\)\))".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_many_unmatched_opening_paranthesis() {
        let simple = CbString::from(b"(((((((((".to_vec());

        // 2 for start and end. many for escaping.
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() * 2 + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, br"(\(\(\(\(\(\(\(\(\()".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_many_matched_paranthesis() {
        let simple = CbString::from(b"((((((()))))))".to_vec());

        // 2 for start and end. many for escaping.
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, br"(((((((())))))))".to_vec());
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn test_many_unmatched_paranthesis() {
        let simple = CbString::from(b")))))(((((".to_vec());

        // 2 for start and end. many for escaping.
        let encoded_len = SimpleEncoder::encoded_len(&simple);
        assert_eq!(encoded_len, simple.len() * 2 + 2);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&simple, &mut out);
        assert_eq!(out, br"(\)\)\)\)\)\(\(\(\(\()".to_vec());
        assert_eq!(encoded_len, out.len());
    }
}
