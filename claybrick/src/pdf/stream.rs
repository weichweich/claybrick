use super::{Bytes, Dictionary};

#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    pub dictionary: Dictionary,
    pub data: Bytes,
}
