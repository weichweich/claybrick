pub trait Writer {
    fn write(&mut self, buf: &[u8]);
}

impl Writer for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend(buf);
    }
}

pub trait Encoder<T> {
    fn encoded_len(o: &T) -> usize;
    fn write_to(o: &T, writer: &mut dyn Writer);
}
