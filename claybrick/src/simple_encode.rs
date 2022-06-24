//! Simple serialiation of a [RawPdf].
//!
//! The implementation is as simple as possible and will result in an
//! unoptimized PDF file (i.e using more bytes than necessary).
use crate::{
    pdf::RawPdf,
    writer::{Encoder, Writer},
};

mod object;
pub mod section;

/// Encode a [RawPdf] in the most simple way.
///
/// * Use cross-reference streams and no xref or trailer (no support for old
///   (pre 1.5) PDF reader)
/// * Nothing is compressed
/// * multiple redundant whitespaces
/// * unoptimized flat object structure
/// * if the [RawPdf] contains multiple sections, they will get merged into a
///   single section
pub struct SimpleEncoder;

impl Encoder<RawPdf> for SimpleEncoder {
    fn write_to(pdf: &RawPdf, writer: &mut dyn Writer) {
        log::trace!("write PDF version and binary indicator");
        // ignore the version and binary fiels in the RawPdf. The fields indicate what
        // was read. We write something different here
        writer.write(b"%PDF-1.7\n");
        writer.write(b"%\0\0\0\0\n");
        for sec in pdf.sections.iter() {
            Self::write_to(sec, writer);
        }
        writer.write(b"%%EOF\n");
    }
}
