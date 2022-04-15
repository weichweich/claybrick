use crate::{pdf::Stream, writer::Encoder};

use crate::simple_encode::SimpleEncoder;

const START_STREAM: &[u8] = b"stream";
const END_STREAM: &[u8] = b"endstream";

impl Encoder<Stream> for SimpleEncoder {
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
