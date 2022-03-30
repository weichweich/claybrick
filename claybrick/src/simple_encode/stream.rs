use crate::{pdf::Stream, writer::Encoder};

use super::SimpleEncoder;

const START_STREAM: &[u8] = b"stream";
const END_STREAM: &[u8] = b"endstream";

impl Encoder<Stream> for SimpleEncoder {
    fn encoded_len(s: &Stream) -> usize {
        Self::encoded_len(&s.dictionary) + 1 + START_STREAM.len() + END_STREAM.len() + s.data.len()
    }

    fn write_to(s: &Stream, writer: &mut dyn crate::writer::Writer) {
        Self::write_to(&s.dictionary, writer);
        writer.write(b" ");
        writer.write(START_STREAM);
        writer.write(&s.data);
        writer.write(END_STREAM);
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests
}
