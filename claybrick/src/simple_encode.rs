use crate::{
    parse::object::{FALSE_OBJECT, NULL_OBJECT, TRUE_OBJECT},
    pdf::Object,
    writer::{Encoder, Writer},
};

mod array;
mod dictionary;
mod name;
mod stream;
mod string;

struct SimpleEncoder;
impl Encoder<Object> for SimpleEncoder {
    fn encoded_len(obj: &Object) -> usize {
        match obj {
            Object::String(str) => Self::encoded_len(str),
            Object::HexString(bytes) => 2 + bytes.len() * 2,
            Object::Float(f) => f.to_string().len(),
            Object::Integer(i) => i.to_string().len(),
            Object::Bool(true) => TRUE_OBJECT.len(),
            Object::Bool(false) => FALSE_OBJECT.len(),
            Object::Name(n) => Self::encoded_len(n),
            Object::Array(a) => Self::encoded_len(a),
            Object::Dictionary(d) => Self::encoded_len(d),
            Object::Stream(s) => Self::encoded_len(s),
            Object::Null => NULL_OBJECT.len(),
            Object::Indirect(_) => todo!(),
            Object::Reference(r) => r.index.to_string().len() + r.generation.to_string().len() + 3,
        }
    }

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
            Object::Indirect(_) => todo!(),
            Object::Reference(r) => {
                writer.write(b"R");
                writer.write(b" ");
                writer.write(r.generation.to_string().as_bytes());
                writer.write(b" ");
                writer.write(r.index.to_string().as_bytes());
            }
        }
    }
}
