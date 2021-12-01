use nom::{bytes, character, IResult };

use crate::pdf::Pdf;

fn parse_version(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, _) = bytes::complete::tag_no_case("%PDF-")(remainder)?;
    let (remainder, major) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::char('.')(remainder)?;
    let (remainder, minor) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::space0(remainder)?;

    Ok((remainder, (major, minor)))
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Pdf> {
    let (remainder, version) = parse_version(input)?;

    Ok((remainder, Pdf { version }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(Ok((&[0u8; 0][..], (1, 8))), parse_version("%PDF-1.8".as_bytes()));
        assert_eq!(Ok((&[0u8; 0][..], (1, 8))), parse_version("   \t\n   %PDF-1.8".as_bytes()));
        assert_eq!(Ok((&[0u8; 0][..], (1, 8))), parse_version("   \t\n   %PDF-1.8 \t   ".as_bytes()));
    }
}