use crate::{
    parse::object::{FALSE_OBJECT, NULL_OBJECT, TRUE_OBJECT},
    pdf::Object,
    writer::{Encoder, Writer},
};

mod array;
mod name;
pub mod string;

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
            Object::Dictionary(_) => todo!(),
            Object::Stream(_) => todo!(),
            Object::Null => NULL_OBJECT.len(),
            Object::Indirect(_) => todo!(),
            Object::Reference(_) => todo!(),
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
            Object::Dictionary(_) => todo!(),
            Object::Stream(_) => todo!(),
            Object::Null => writer.write(NULL_OBJECT.as_bytes()),
            Object::Indirect(_) => todo!(),
            Object::Reference(_) => todo!(),
        }
    }
}
