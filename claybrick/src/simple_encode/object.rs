use crate::{
    parse::object::{FALSE_OBJECT, NULL_OBJECT, TRUE_OBJECT},
    pdf::Object,
    writer::{Encoder, Writer},
};

use super::SimpleEncoder;

pub(crate) mod array;
pub(crate) mod dictionary;
pub(crate) mod indirect;
pub(crate) mod name;
pub(crate) mod stream;
pub(crate) mod string;

impl Encoder<Object> for SimpleEncoder {
    fn write_to(obj: &Object, writer: &mut dyn Writer) {
        match obj {
            Object::String(str) => Self::write_to(str, writer),
            Object::HexString(bytes) => {
                writer.write(b"<");
                writer.write(hex::encode(&bytes[..]).as_bytes());
                writer.write(b">");
            }
            Object::Float(f) => writer.write(f.to_string().as_bytes()),
            Object::Integer(i) => writer.write(i.to_string().as_bytes()),
            Object::Bool(true) => writer.write(TRUE_OBJECT.as_bytes()),
            Object::Bool(false) => writer.write(FALSE_OBJECT.as_bytes()),
            Object::Name(n) => Self::write_to(n, writer),
            Object::Array(a) => Self::write_to(a, writer),
            Object::Dictionary(d) => Self::write_to(d, writer),
            Object::Stream(s) => Self::write_to(s, writer),
            Object::Null => writer.write(NULL_OBJECT.as_bytes()),
            Object::Indirect(i) => Self::write_to(i, writer),
            Object::Reference(r) => {
                writer.write(b"R");
                writer.write(b" ");
                writer.write(r.generation.to_string().as_bytes());
                writer.write(b" ");
                writer.write(r.index.to_string().as_bytes());
            }
        }
        writer.write(b"\n");
    }
}
