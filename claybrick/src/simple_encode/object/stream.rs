use crate::{
    pdf::{document::K_LENGTH, Name, Object, Stream},
    writer::Encoder,
};

use crate::simple_encode::SimpleEncoder;

const START_STREAM: &[u8] = b"stream\n";
const END_STREAM: &[u8] = b"\nendstream";

impl Encoder<Stream> for SimpleEncoder {
    fn write_to(s: &Stream, writer: &mut dyn crate::writer::Writer) {
        // update the dictionary to fit the new layout
        let mut updated_dict = s.dictionary.clone();
        updated_dict.insert(
            Name::from(K_LENGTH),
            Object::from(i32::try_from(s.data.len()).expect("FIXME: don't panic")),
        );
        Self::write_to(&updated_dict, writer);
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
