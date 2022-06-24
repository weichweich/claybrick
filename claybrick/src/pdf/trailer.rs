use super::{Bytes, Dictionary, Object, Reference};

pub const TRAILER: &[u8] = b"trailer";
pub const K_SIZE: &[u8] = b"Size";
pub const K_PREVIOUS: &[u8] = b"Prev";
pub const K_ENCRYPT: &[u8] = b"Encrypt";
pub const K_ROOT: &[u8] = b"Root";
pub const K_INFO: &[u8] = b"info";
pub const K_ID: &[u8] = b"ID";
pub const K_X_REF_STM: &[u8] = b"XRefStm";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrailerError {
    InvalidSize,
    MissingSize,
    InvalidRoot,
    MissingRoot,
    InvalidXRefStm,
    MissingXRefStm,
    InvalidPrevious,
    InvalidInfo,
    InvalidId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trailer {
    /// Highest object number used in the PDF document
    pub size: usize,

    /// Byte offset to the previous PDF section
    pub previous: Option<usize>,

    /// Reference to the root object.
    pub root: Reference,

    /// Dictionary containing information for decryption.
    pub encrypt: Option<Dictionary>,

    /// Information for this document.
    pub info: Option<Reference>,

    /// File identifier used for encryption.
    pub id: Option<[Bytes; 2]>,

    /// Start of the XRef table.
    ///
    /// This provides obtional compatibility to readers that don't support XRef
    /// streams.
    pub x_ref_stm: Option<usize>,
}

impl From<Trailer> for Dictionary {
    fn from(trailer: Trailer) -> Self {
        // we now that the trailer struct has 7 fields.
        let mut dict = Dictionary::with_capacity(7);
        dict.insert(
            K_SIZE.to_owned().into(),
            Object::Integer(trailer.size.try_into().expect("FIXME")),
        );
        if let Some(prev) = trailer.previous {
            dict.insert(
                K_PREVIOUS.to_owned().into(),
                Object::Integer(prev.try_into().expect("FIXME")),
            );
        }

        dict.insert(K_ROOT.to_owned().into(), Object::Reference(trailer.root));

        if let Some(enc) = trailer.encrypt {
            dict.insert(K_ENCRYPT.to_owned().into(), Object::Dictionary(enc));
        }

        if let Some(info) = trailer.info {
            dict.insert(K_INFO.to_owned().into(), Object::Reference(info));
        }

        if let Some([id0, id1]) = trailer.id {
            dict.insert(
                K_ID.to_owned().into(),
                Object::Array(vec![Object::HexString(id0), Object::HexString(id1)].into()),
            );
        }

        dict
    }
}

impl TryFrom<Dictionary> for Trailer {
    type Error = TrailerError;

    fn try_from(dict: Dictionary) -> Result<Self, Self::Error> {
        Ok(Trailer {
            size: dict
                .get(K_SIZE)
                .ok_or(TrailerError::MissingSize)?
                .integer()
                .ok_or(TrailerError::InvalidSize)?
                .try_into()
                .map_err(|_| TrailerError::InvalidSize)?,

            previous: dict
                .get(K_PREVIOUS)
                .and_then(Object::integer)
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| TrailerError::InvalidPrevious)?,

            root: dict
                .get(K_ROOT)
                .ok_or(TrailerError::MissingRoot)?
                .reference()
                // TODO: don't clone
                .cloned()
                .ok_or(TrailerError::InvalidRoot)?,

            // TODO: don't clone
            encrypt: dict.get(K_ENCRYPT).and_then(|enc| enc.dictionary()).cloned(),

            // TODO: don't clone
            info: dict
                .get(K_INFO)
                .map(|o| o.reference().ok_or(TrailerError::InvalidInfo))
                .transpose()?
                .cloned(),

            id: dict
                .get(K_ID)
                .map(|o| o.array().ok_or(TrailerError::InvalidId))
                .transpose()?
                .map(|a| {
                    if a.len() == 2 {
                        Ok([
                            // TODO: don't clone
                            a.first()
                                .and_then(Object::hex_string)
                                .ok_or(TrailerError::InvalidId)?
                                .clone(),
                            // TODO: don't clone
                            a.get(1)
                                .and_then(Object::hex_string)
                                .ok_or(TrailerError::InvalidId)?
                                .clone(),
                        ])
                    } else {
                        Err(TrailerError::InvalidId)
                    }
                })
                .transpose()?,

            x_ref_stm: dict
                .get(K_X_REF_STM)
                .map(|obj| obj.integer().ok_or(TrailerError::InvalidXRefStm))
                .transpose()?
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| TrailerError::InvalidXRefStm)?,
        })
    }
}
