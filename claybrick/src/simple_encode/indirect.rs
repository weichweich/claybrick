use crate::{pdf::IndirectObject, writer::Encoder};

use super::SimpleEncoder;

impl Encoder<IndirectObject> for SimpleEncoder {
    fn encoded_len(o: &IndirectObject) -> usize {
        o.generation.to_string().len() + o.index.to_string().len() + 2 + Self::encoded_len(&*o.object)
    }

    fn write_to(o: &IndirectObject, writer: &mut dyn crate::writer::Writer) {
        writer.write(o.generation.to_string().as_bytes());
        writer.write(b" ");
        writer.write(o.index.to_string().as_bytes());
        writer.write(b" ");
        Self::write_to(&*o.object, writer);
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests
}
