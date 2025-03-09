use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0},
    sequence::{delimited, preceded},
    combinator::{map_res},
    Parser,
};
use crate::parsers::sqf::models::RandomRange;

/// Parse a floating point number
pub fn float_value(input: &str) -> IResult<&str, f32> {
    map_res(
        take_while1(|c: char| c.is_numeric() || c == '.'),
        |s: &str| s.parse::<f32>()
    ).parse(input)
}

/// Parse a random range expression
/// Example: random [0,1,4]
pub fn random_range(input: &str) -> IResult<&str, RandomRange> {
    let (input, _) = tag("random").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('[').parse(input)?;
    let (input, min) = preceded(space0, float_value).parse(input)?;
    let (input, _) = preceded(space0, char(',')).parse(input)?;
    let (input, mid) = preceded(space0, float_value).parse(input)?;
    let (input, _) = preceded(space0, char(',')).parse(input)?;
    let (input, max) = preceded(space0, float_value).parse(input)?;
    let (input, _) = preceded(space0, char(']')).parse(input)?;
    
    Ok((input, RandomRange { min, mid, max }))
}

/// Parse a line that might contain a random range expression
pub fn parse_random_line(input: &str) -> Option<RandomRange> {
    let trimmed = input.trim();
    
    // Skip empty lines or comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }
    
    // Try to parse random range expressions
    if let Some(random_start) = trimmed.find("random [") {
        if let Ok((_, range)) = random_range(&trimmed[random_start..]) {
            return Some(range);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_range() {
        let input = "random [0,1,4]";
        let (_, range) = random_range(input).unwrap();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.mid, 1.0);
        assert_eq!(range.max, 4.0);
    }

    #[test]
    fn test_parse_random_line() {
        let input = "for \"_i\" from 1 to (ceil (random [0,1,4])) do {";
        let range = parse_random_line(input).unwrap();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.mid, 1.0);
        assert_eq!(range.max, 4.0);
    }
} 