use nom::{bytes, character, sequence, IResult};

use crate::pdf::Pdf;

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
        let empty = &[0u8; 0][..];
        assert_eq!(Ok((empty, (1, 8))), version("%PDF-1.8".as_bytes()));
        assert_eq!(
            Ok((empty, (1, 8))),
            version("   \t\n   %PDF-1.8".as_bytes())
        );
        assert_eq!(
            Ok((empty, (1, 8))),
            version("   \t\n   %PDF-1.8 \t   ".as_bytes())
        );
    }

    #[test]
    fn test_parse() {
        let empty = &[0u8; 0][..];
        assert_eq!(
            parse(
                "%PDF-1.7
%\x01\x01\x01\x01\x01
1 0 obj
<< /Type /Catalog
   /Pages 2 0 R
>>
endobj
2 0 obj
<< /Kids [3 0 R]
   /Type /Pages
   /Count 1
>>
endobj".as_bytes()
            ),
            Ok((
                empty,
                Pdf {
                    version: (1, 7),
                    announced_binary: true,
                    objects: vec![]
                }
            ))
        )
    }
}
