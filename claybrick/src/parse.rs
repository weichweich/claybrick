use nom::{bytes, character, IResult};

use crate::pdf::Pdf;

mod object;

fn version(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
    let (remainder, _) = bytes::complete::tag_no_case("%PDF-")(input)?;
    let (remainder, major) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::char('.')(remainder)?;
    let (remainder, minor) = character::complete::u8(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

    Ok((remainder, (major, minor)))
}

fn comment(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, _) = character::complete::char('%')(remainder)?;
    let (remainder, comment) = character::complete::not_line_ending(remainder)?;
    let (remainder, _) = character::complete::line_ending(remainder)?;
    let (remainder, _) = character::complete::multispace0(remainder)?;

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
    let (remainder, _) = character::complete::multispace0(input)?;
    let (remainder, version) = version(remainder)?;
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
        assert_eq!(Ok((empty, (1, 8))), version(b"%PDF-1.8"));
    }

    #[test]
    fn test_parse_binary_indicator() {
        let empty = &[0u8; 0][..];
        assert_eq!(Ok((empty, true)), binary_indicator(b"%\xbf\xbf\xbf\xbf\xbf\n"));
    }

    #[test]
    fn test_parse() {
        let empty = &[0u8; 0][..];
        assert_eq!(
            parse(
                b"%PDF-1.7
%\xbf\xbf\xbf\xbf\xbf
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
endobj"
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
