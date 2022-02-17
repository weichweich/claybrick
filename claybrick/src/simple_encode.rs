use crate::{
    parse::object::{FALSE_OBJECT, TRUE_OBJECT},
    pdf::Object,
    writer::{Encoder, Writer},
};

pub mod string;

struct SimpleEncode;
impl Encoder<Object> for SimpleEncode {
    fn encoded_len(obj: &Object) -> usize {
        match obj {
            Object::String(str) => Self::encoded_len(str),
            Object::HexString(_) => todo!(),
            Object::Float(_) => todo!(),
            Object::Integer(_) => todo!(),
            Object::Bool(true) => TRUE_OBJECT.len(),
            Object::Bool(false) => FALSE_OBJECT.len(),
            Object::Name(_) => todo!(),
            Object::Array(_) => todo!(),
            Object::Dictionary(_) => todo!(),
            Object::Stream(_) => todo!(),
            Object::Null => todo!(),
            Object::Indirect(_) => todo!(),
            Object::Reference(_) => todo!(),
        }
    }

    fn write_to(obj: &Object, writer: &mut dyn Writer) {
        match obj {
            Object::String(str) => Self::write_to(str, writer),
            Object::HexString(_) => todo!(),
            Object::Float(_) => todo!(),
            Object::Integer(_) => todo!(),
            Object::Bool(true) => writer.write(TRUE_OBJECT.as_bytes()),
            Object::Bool(false) => writer.write(FALSE_OBJECT.as_bytes()),
            Object::Name(_) => todo!(),
            Object::Array(_) => todo!(),
            Object::Dictionary(_) => todo!(),
            Object::Stream(_) => todo!(),
            Object::Null => todo!(),
            Object::Indirect(_) => todo!(),
            Object::Reference(_) => todo!(),
        }
    }
}
