use crate::{parse::object::is_regular, pdf::Name, writer::Encoder};

use super::SimpleEncoder;

impl Encoder<Name> for SimpleEncoder {
    fn encoded_len(n: &Name) -> usize {
        n.iter().map(|c| if is_regular(*c) { 1 } else { 3 }).sum::<usize>() + 1
    }

    fn write_to(n: &Name, writer: &mut dyn crate::writer::Writer) {
        let mut last_write = 0;
        writer.write(br"\");
        for (index, &c) in n.iter().enumerate() {
            if !is_regular(c) {
                writer.write(&n[last_write..index]);
                last_write = index + 1;
                writer.write(b"#");
                writer.write(hex::encode(c.to_be_bytes()).as_bytes())
            }
        }
        writer.write(&n[last_write..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiter_in_the_middle() {
        let name = Name::from(b"Hello World!".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&name);
        assert_eq!(encoded_len, 15);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&name, &mut out);
        let expected = br"\Hello#20World!";
        assert_eq!(
            out,
            expected,
            "Expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&out)
        );
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn delimiter_start() {
        let name = Name::from(b" HelloWorld!".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&name);
        assert_eq!(encoded_len, 15);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&name, &mut out);
        let expected = br"\#20HelloWorld!";
        assert_eq!(
            out,
            expected,
            "Expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&out)
        );
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn delimiter_end() {
        let name = Name::from(b"HelloWorld! ".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&name);
        assert_eq!(encoded_len, 15);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&name, &mut out);
        let expected = br"\HelloWorld!#20";
        assert_eq!(
            out,
            expected,
            "Expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&out)
        );
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn only_delimiters() {
        let name = Name::from(b"   ".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&name);
        assert_eq!(encoded_len, 10);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&name, &mut out);
        let expected = br"\#20#20#20";
        assert_eq!(
            out,
            expected,
            "Expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&out)
        );
        assert_eq!(encoded_len, out.len());
    }

    #[test]
    fn no_delimiters() {
        let name = Name::from(b"HelloWorld!".to_vec());
        let encoded_len = SimpleEncoder::encoded_len(&name);
        assert_eq!(encoded_len, 12);
        let mut out = Vec::new();
        SimpleEncoder::write_to(&name, &mut out);
        let expected = br"\HelloWorld!";
        assert_eq!(
            out,
            expected,
            "Expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&out)
        );
        assert_eq!(encoded_len, out.len());
    }
}
