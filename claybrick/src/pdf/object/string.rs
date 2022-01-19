use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CbString(Vec<u8>);

impl From<Vec<u8>> for CbString {
    fn from(v: Vec<u8>) -> Self {
        CbString(v)
    }
}

impl Deref for CbString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for CbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CbString")
            .field(&String::from_utf8_lossy(&self.0[..]))
            .finish()
    }
}

impl std::fmt::Display for CbString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &String::from_utf8_lossy(&self.0[..]))
    }
}
