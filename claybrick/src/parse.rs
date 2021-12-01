use nom::{branch, bytes, character, multi, sequence, IResult};

use crate::pdf::{Object, Pdf};

mod object;

fn version(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, _) = bytes::complete::tag_no_case("%PDF-")(remainder)?;
    let (remainder, major) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::char('.')(remainder)?;
    let (remainder, minor) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::space0(remainder)?;

    Ok((remainder, (major, minor)))
}

fn comment(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, (_, comment)) = sequence::pair(
        character::complete::char('%'),
        character::complete::not_line_ending,
    )(remainder)?;

    Ok((remainder, comment))
}

fn binary_indicator(input: &[u8]) -> IResult<&[u8], bool> {
    if let Ok((r, comment)) = comment(input) {
        if comment.len() > 3 && comment.iter().find(|&d| *d < 128).is_none() {
            Ok((r, true))
        } else {
            Ok((input, false))
        }
    } else {
        Ok((input, false))
    }
}

pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Pdf> {
    let (remainder, version) = version(input)?;
    let (remainder, announced_binary) = binary_indicator(remainder)?;
    let (remainder, objects) = object::object0(remainder)?;

    Ok((
        remainder,
        Pdf {
            version,
            announced_binary,
            objects,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(Ok((&[0u8; 0][..], (1, 8))), version("%PDF-1.8".as_bytes()));
        assert_eq!(
            Ok((&[0u8; 0][..], (1, 8))),
            version("   \t\n   %PDF-1.8".as_bytes())
        );
        assert_eq!(
            Ok((&[0u8; 0][..], (1, 8))),
            version("   \t\n   %PDF-1.8 \t   ".as_bytes())
        );
    }
}
