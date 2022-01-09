use self::filter::FilterError;

use super::{Bytes, Dictionary, Name, Object};

const FILTER: &[u8] = b"Filter";
const FILTER_PARAM: &[u8] = b"DecodeParms";

#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    pub dictionary: Dictionary,
    pub data: Bytes,
}

impl Stream {
    pub fn filters(&self) -> Result<Vec<&Name>, FilterError> {
        match self.dictionary.get(FILTER) {
            Some(Object::Array(a)) => a
                .iter()
                .map(|n| n.name())
                .collect::<Option<Vec<_>>>()
                .ok_or(FilterError::InvalidFilter),
            Some(Object::Name(n)) => Ok(vec![n]),
            None => Ok(vec![]),
            Some(..) => Err(FilterError::InvalidFilter),
        }
    }

    pub fn filtered_data(&self) -> Result<Bytes, FilterError> {
        let mut out_data = self.data.clone();
        for f in self.filters()? {
            out_data = filter::filter(
                f,
                self.dictionary.get(FILTER_PARAM).and_then(Object::dictionary),
                &out_data,
            )?;
        }
        Ok(out_data)
    }
}

pub mod filter {
    use std::borrow::Borrow;

    use flate2::{Decompress, FlushDecompress, Status};

    use crate::pdf::{Bytes, Dictionary, Name};

    const FILTER_ASCII_HEX: &[u8] = b"ASCIIHexDecode";
    const FILTER_ASCII_85: &[u8] = b"ASCII85Decode";
    const FILTER_LZW: &[u8] = b"LZWDecode";
    const FILTER_FLATE: &[u8] = b"FlateDecode";
    const FILTER_RUN_LENGTH: &[u8] = b"RunLengthDecode";
    const FILTER_CCITT_FAX: &[u8] = b"CCITTFaxDecode";
    const FILTER_JBIG2: &[u8] = b"JBIG2Decode";
    const FILTER_DCT: &[u8] = b"DCTDecode";
    const FILTER_JPX: &[u8] = b"JPXDecode";
    const FILTER_CRYPT: &[u8] = b"Crypt";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum FilterError {
        UnknownFilter(Name),
        UnsupportedFilter(Name),
        InvalidData,
        InvalidFilter,
    }

    pub fn filter(filter_name: &Name, _p: Option<&Dictionary>, data: &Bytes) -> Result<Bytes, FilterError> {
        match filter_name.borrow() {
            FILTER_ASCII_HEX => decode_ascii_hex(data.borrow()),
            FILTER_ASCII_85 => Err(FilterError::UnsupportedFilter(FILTER_ASCII_85.to_vec().into())),
            FILTER_LZW => Err(FilterError::UnsupportedFilter(FILTER_LZW.to_vec().into())),
            FILTER_FLATE => decode_flate(data),
            FILTER_RUN_LENGTH => Err(FilterError::UnsupportedFilter(FILTER_RUN_LENGTH.to_vec().into())),
            FILTER_CCITT_FAX => Err(FilterError::UnsupportedFilter(FILTER_CCITT_FAX.to_vec().into())),
            FILTER_JBIG2 => Err(FilterError::UnsupportedFilter(FILTER_JBIG2.to_vec().into())),
            FILTER_DCT => Err(FilterError::UnsupportedFilter(FILTER_DCT.to_vec().into())),
            FILTER_JPX => Err(FilterError::UnsupportedFilter(FILTER_JPX.to_vec().into())),
            FILTER_CRYPT => Err(FilterError::UnsupportedFilter(FILTER_CRYPT.to_vec().into())),
            name => Err(FilterError::UnknownFilter(name.to_vec().into())),
        }
    }

    fn decode_ascii_hex(data: &[u8]) -> Result<Bytes, FilterError> {
        let mut buffer = Vec::<u8>::with_capacity(data.len() / 2 + 1);
        // TODO: replace with group_by once it's stable
        let mut acc = None;
        for b in data.iter().filter(|b| !b.is_ascii_whitespace()) {
            let nibble = match b {
                b @ b'0'..=b'9' => Some(b - b'0'),
                b @ b'A'..=b'F' => Some(b - b'A' + 10),
                b @ b'a'..=b'f' => Some(b - b'a' + 10),
                b'>' => None,
                _ => return Err(FilterError::InvalidData),
            };

            // always clear acc
            match (acc.take(), nibble) {
                // Push new byte
                (Some(acc), Some(b)) => buffer.push((acc << 4) + b),

                // Store nibble for next iteration.
                (None, Some(b)) => acc = Some(b),

                // reached end of data, but nibble needs to be padded with 0
                (Some(acc), None) => {
                    buffer.push(acc << 4);
                    break;
                }

                // Reached end of data, no action required
                (None, None) => break,
            }
        }
        Ok(buffer.into())
    }

    fn decode_flate(data: &Bytes) -> Result<Bytes, FilterError> {
        let mut d = Decompress::new(true);
        let mut out = Vec::<u8>::with_capacity(2 * 1024 * 1024);
        let into_invalid_data_err = |err| {
            log::error!(
                "Error while applying {} filter: {:?}",
                String::from_utf8_lossy(FILTER_FLATE),
                err
            );
            FilterError::InvalidData
        };

        while Status::StreamEnd
            != d.decompress_vec(&data[..], &mut out, FlushDecompress::None)
                .map_err(into_invalid_data_err)?
        {
            out.reserve(2 * 1024 * 1024);
        }

        Ok(out.into())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_decode_ascii_hex() {
            assert_eq!(
                Ok(b"Hello world!".to_vec().into()),
                decode_ascii_hex(&b"48656c6c6f20776f726c6421"[..])
            );
            assert_eq!(
                Ok(b"Hello world!".to_vec().into()),
                decode_ascii_hex(&b" 48656c6c6f20776f726c6421 "[..])
            );
            assert_eq!(
                Ok(b"Hello world!".to_vec().into()),
                decode_ascii_hex(&b"4 8 6 5 6 c 6 c 6 f 2 0 7 7 6 f 7 2 6 c 6 4 2 1"[..])
            );
        }
    }
}
