use nom::{bytes, character};

use super::{
    backward_search,
    error::{CbParseError, CbParseErrorKind},
    CbParseResult, Span,
};

const EOF_MARKER: &[u8] = b"\n%%EOF\n";
const STARTXREF: &[u8] = b"startxref";

pub(crate) fn startxref_tail(input: Span) -> CbParseResult<usize> {
    let (remainder, (trailing, _)) = backward_search::<_, _, _, CbParseError<Span>>(
        STARTXREF.len() + 2048,
        bytes::complete::tag_no_case(STARTXREF),
    )(input)?;
    let (trailing, _) = character::complete::multispace0(trailing)?;
    let (_, xref_pos) = character::complete::i32(trailing)?;
    let xref_pos: usize = xref_pos
        .try_into()
        .map_err(|_| nom::Err::Error(CbParseError::new(input, CbParseErrorKind::StartxrefInvalid)))?;

    Ok((remainder, xref_pos))
}

pub(crate) fn eof_marker_tail(input: Span) -> CbParseResult<()> {
    // trailing bytes that follow the EOF marker are not possible since the limit we
    // provided is the length of the EOF marker
    let (remainder, _trailing) = backward_search::<_, _, _, CbParseError<Span>>(
        EOF_MARKER.len() + 2,
        bytes::complete::tag_no_case(EOF_MARKER),
    )(input)?;

    Ok((remainder, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startxref_tail() {
        let input = &b"         startxref\n2132"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Ok((_, 2132))));

        let input = &b"         startxref\n555\nasdfsadfasdfsadfasdfsadfsadf"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Ok((_, 555))));
    }

    #[test]
    fn test_invalid_startxref_tail() {
        // to big
        let input = &b"         startxref\n9999999999999999999999999999999"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(res, Err(nom::Err::Error(_))));

        let input = &b"         startxref\n-555\nasdfsadfasdfsadfasdfsadfsadf"[..];
        let res = startxref_tail(input.into());
        assert!(matches!(
            res,
            Err(nom::Err::Error(CbParseError {
                input: _,
                kind: CbParseErrorKind::StartxrefInvalid,
                from: None
            }))
        ));
    }
}
