use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name(Vec<u8>);

impl From<Vec<u8>> for Name {
    fn from(v: Vec<u8>) -> Self {
        Name(v)
    }
}

impl Deref for Name {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name")
            .field(&String::from_utf8_lossy(&self.0[..]))
            .finish()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &String::from_utf8_lossy(&self.0[..]))
    }
}
