//! Simple serialiation of a [RawPdf].
//!
//! The implementation is as simple as possible and will result in an
//! unoptimized PDF file (i.e using more bytes than necessary).
use crate::{
    writer::{Encoder, Writer},
};

mod object;
struct SimpleEncoder;
