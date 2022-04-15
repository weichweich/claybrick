pub trait Writer {
    /// Write the buffer.
    fn write(&mut self, buf: &[u8]);

    /// Index of the next byte that will be written.
    fn position(&self) -> usize;
}

impl Writer for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend(buf);
    }

    fn position(&self) -> usize {
        self.len()
    }
}

struct DummyWriter {
    size: usize,
}

impl DummyWriter {
    fn new() -> Self {
        Self { size: 0 }
    }

    fn len(&self) -> usize {
        self.size
    }
}

impl Default for DummyWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for DummyWriter {
    fn write(&mut self, buf: &[u8]) {
        self.size += buf.len();
    }

    fn position(&self) -> usize {
        self.size
    }
}

pub trait Encoder<T> {
    fn encoded_len(o: &T) -> usize {
        let mut out = DummyWriter::new();
        Self::write_to(o, &mut out);
        out.len()
    }

    fn write_to(o: &T, writer: &mut dyn Writer);
}
