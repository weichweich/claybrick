use crate::pdf::CbString;

use super::{Encode, Writer};

impl Encode for CbString {
    fn encoded_len(&self) -> usize {
        let mut len = self.len();
        let mut open_paranthesis = 0;
        let mut remaining_closing_paranthesis = self.iter().filter(|&c| *c == b')').count();

        // check for characters that we need to escape.
        for char in self.iter() {
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

    fn write_to(&self, writer: &mut dyn Writer) {
        writer.write(&b"("[..]);

        let mut open_paranthesis: usize = 0;
        let mut remaining_closing_paranthesis = self.iter().filter(|&c| *c == b')').count();

        let mut last_written_index = 0;
        // check for characters that we need to escape.
        for (index, char) in self.iter().enumerate() {
            match (char, open_paranthesis, remaining_closing_paranthesis) {
                (b'(', _, 0) => {
                    open_paranthesis += 1;
                    writer.write(&self[last_written_index..index]);
                    writer.write(&br"\"[..]);
                    last_written_index = index;
                }
                (b'(', _, _) => open_paranthesis += 1,
                // unbalanced closing paranthesis need to be escaped, they would otherwise determain the end of the
                // string
                (b')', 0, _) => {
                    writer.write(&self[last_written_index..index]);
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
        writer.write(&self[last_written_index..]);
        writer.write(&b")"[..]);
    }
}

#[cfg(test)]
mod tests {
    use crate::{pdf::CbString, writer::Encode};

    #[test]
    fn test_simple() {
        let simple = CbString::from(b"abcdefg".to_vec());
        assert_eq!(simple.len() + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, b"(abcdefg)".to_vec())
    }

    #[test]
    fn test_end_with_closing_paranthesis() {
        let simple = CbString::from(b"(abcdefg)".to_vec());
        assert_eq!(simple.len() + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, b"((abcdefg))".to_vec())
    }

    #[test]
    fn test_end_with_unmatched_closing_paranthesis() {
        let simple = CbString::from(b"abcdefg)".to_vec());

        // 2 for start and end. One for escaping.
        assert_eq!(simple.len() + 3, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, br"(abcdefg\))".to_vec())
    }

    #[test]
    fn test_many_unmatched_closing_paranthesis() {
        let simple = CbString::from(b")))))))))".to_vec());

        // 2 for start and end. many for escaping.
        assert_eq!(simple.len() * 2 + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, br"(\)\)\)\)\)\)\)\)\))".to_vec())
    }

    #[test]
    fn test_many_unmatched_opening_paranthesis() {
        let simple = CbString::from(b"(((((((((".to_vec());

        // 2 for start and end. many for escaping.
        assert_eq!(simple.len() * 2 + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, br"(\(\(\(\(\(\(\(\(\()".to_vec())
    }

    #[test]
    fn test_many_matched_paranthesis() {
        let simple = CbString::from(b"((((((()))))))".to_vec());

        // 2 for start and end. many for escaping.
        assert_eq!(simple.len() + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, br"(((((((())))))))".to_vec())
    }

    #[test]
    fn test_many_unmatched_paranthesis() {
        let simple = CbString::from(b")))))(((((".to_vec());

        // 2 for start and end. many for escaping.
        assert_eq!(simple.len() * 2 + 2, simple.encoded_len());
        let mut out = Vec::new();
        simple.write_to(&mut out);
        assert_eq!(out, br"(\)\)\)\)\)\(\(\(\(\()".to_vec())
    }
}
