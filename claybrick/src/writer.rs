pub mod string;

pub trait Writer {
    fn write(&mut self, buf: &[u8]);
}

impl Writer for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend(buf);
    }
}

pub trait Encode {
    fn encoded_len(&self) -> usize;
    fn write_to(&self, writer: &mut dyn Writer);
}
