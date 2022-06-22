use crate::{pdf::IndirectObject, writer::Encoder};

use crate::simple_encode::SimpleEncoder;

impl Encoder<IndirectObject> for SimpleEncoder {
    fn write_to(o: &IndirectObject, writer: &mut dyn crate::writer::Writer) {
        writer.write(o.index.to_string().as_bytes());
        writer.write(b" ");
        writer.write(o.generation.to_string().as_bytes());
        writer.write(b" obj\n");
        Self::write_to(&*o.object, writer);
        writer.write(b"endobj\n");
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests
}
