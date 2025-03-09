use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0, space1, digit1},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, opt, recognize},
    error::Error,
    Parser,
};

use crate::sqf::models::RandomRange;

/// Parse a floating point number
pub fn float_value(input: &str) -> IResult<&str, f32> {
    map(
        recognize((
            opt(alt((char('+'), char('-')))),
            alt((
                recognize((
                    opt(digit1),
                    char('.'),
                    opt(digit1),
                )),
                digit1,
            )),
            opt((
                alt((char('e'), char('E'))),
                opt(alt((char('+'), char('-')))),
                digit1,
            )),
        )),
        |s: &str| s.parse().unwrap()
    ).parse(input)
}

/// Parse a random range expression
/// Example: random [0,1,4]
pub fn random_range(input: &str) -> IResult<&str, RandomRange> {
    let (input, _) = tag("random").parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char('[')).parse(input)?;
    let (input, min) = preceded(space0::<&str, Error<&str>>, float_value).parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char(',')).parse(input)?;
    let (input, mid) = preceded(space0::<&str, Error<&str>>, float_value).parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char(',')).parse(input)?;
    let (input, max) = preceded(space0::<&str, Error<&str>>, float_value).parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char(']')).parse(input)?;
    
    Ok((input, RandomRange { min, mid, max }))
}

/// Parse a line that might contain a random range expression
pub fn parse_random_line(input: &str) -> Option<RandomRange> {
    let trimmed = input.trim();
    
    // Skip empty lines or comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }
    
    // Find the random expression
    if let Some(pos) = trimmed.find("random [") {
        if let Ok((_, range)) = random_range(&trimmed[pos..]) {
            return Some(range);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn test_float_value() {
        // Basic cases
        let test_cases = vec![
            ("1.0", 1.0),
            ("0.5", 0.5),
            ("10", 10.0),
            ("0", 0.0),
            ("0.0", 0.0),
            ("1000.001", 1000.001),
            (".5", 0.5),        // Leading decimal
            ("5.", 5.0),        // Trailing decimal
            ("0000.5000", 0.5), // Leading/trailing zeros
            ("+1.5", 1.5),      // Explicit positive
            ("-1.5", -1.5),     // Negative
            ("1e2", 100.0),     // Scientific notation
            ("1.5e-2", 0.015),  // Scientific with decimal
            ("1E+2", 100.0),    // Capital E
        ];

        for (input, expected) in test_cases {
            let (rest, value) = float_value(input).unwrap();
            assert_float_eq!(value, expected, abs <= 0.0001);
            assert_eq!(rest, "");
        }

        // With trailing content
        let test_cases = vec![
            ("1.5rest", (1.5, "rest")),
            ("0.5;", (0.5, ";")),
            ("10.0,", (10.0, ",")),
            ("1.5 + 2.0", (1.5, " + 2.0")),
        ];

        for (input, (expected_value, expected_rest)) in test_cases {
            let (rest, value) = float_value(input).unwrap();
            assert_float_eq!(value, expected_value, abs <= 0.0001);
            assert_eq!(rest, expected_rest);
        }

        // Error cases
        let error_cases = vec![
            "",             // Empty string
            "abc",         // Non-numeric
            ".",          // Just decimal
            "1.2.3",      // Multiple decimals
            "--1.0",      // Multiple signs
            "+-1.0",      // Multiple signs
            "1.0e",       // Incomplete scientific
            "1.0e+",      // Incomplete scientific
            "1.0ee2",     // Multiple e
            "0x1.5",      // Hex notation
            "_1.5",       // Leading underscore
        ];

        for input in error_cases {
            assert!(float_value(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_random_range() {
        // Basic cases
        let test_cases = vec![
            ("random [0,1,4]", (0.0, 1.0, 4.0)),
            ("random [0.5,1.5,2.5]", (0.5, 1.5, 2.5)),
            ("random [1,1,1]", (1.0, 1.0, 1.0)),
            ("random [-1,0,1]", (-1.0, 0.0, 1.0)),
            ("random [0.001,0.002,0.003]", (0.001, 0.002, 0.003)),
            ("random [1e2,2e2,3e2]", (100.0, 200.0, 300.0)),
        ];

        for (input, (exp_min, exp_mid, exp_max)) in test_cases {
            let (_, range) = random_range(input).unwrap();
            assert_float_eq!(range.min, exp_min, abs <= 0.0001);
            assert_float_eq!(range.mid, exp_mid, abs <= 0.0001);
            assert_float_eq!(range.max, exp_max, abs <= 0.0001);
        }

        // Whitespace variations
        let whitespace_cases = vec![
            "random[0,1,4]",                    // No spaces
            "random [0, 1, 4]",                 // Normal spaces
            "random [ 0 , 1 , 4 ]",            // Extra spaces
            "random     [0,1,4]",              // Multiple spaces
            "random\t[0,1,4]",                 // Tab
            "random [\n0,\n1,\n4\n]",         // Newlines
            "random [ 0 ,1,4]",                // Mixed spacing
        ];

        for input in whitespace_cases {
            let (_, range) = random_range(input).unwrap();
            assert_float_eq!(range.min, 0.0, abs <= 0.0001);
            assert_float_eq!(range.mid, 1.0, abs <= 0.0001);
            assert_float_eq!(range.max, 4.0, abs <= 0.0001);
        }

        // Error cases
        let error_cases = vec![
            // Syntax errors
            "random 0,1,4]",                    // Missing opening bracket
            "random [0,1,4",                    // Missing closing bracket
            "random (0,1,4)",                   // Wrong brackets
            "random [0:1:4]",                   // Wrong separator
            
            // Wrong number of values
            "random []",                        // Empty
            "random [0]",                       // One value
            "random [0,1]",                     // Two values
            "random [0,1,2,3]",                // Four values
            
            // Invalid values
            "random [a,b,c]",                  // Non-numeric
            "random [,1,2]",                   // Missing value
            "random [1,,2]",                   // Missing middle
            "random [1,2,]",                   // Missing last
            "random [.,.,.}",                  // Just decimals
            
            // Invalid keywords/format
            "[0,1,2]",                         // Missing random
            "Random [0,1,2]",                  // Wrong case
            "random[0,1,2",                    // Unclosed
            "random 0,1,2",                    // No brackets
        ];

        for input in error_cases {
            assert!(random_range(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_random_line() {
        // Test valid cases with context
        let valid_cases = vec![
            // Basic cases
            "random [0,1,4]",
            "for \"_i\" from 1 to (ceil (random [0,1,4])) do {",
            "_damage = random [0.1,0.5,0.9];",
            
            // With calculations
            "random [0,1,2] + random [3,4,5]",  // Should parse first one
            "_var = 1 + random [0,1,2] - 3",
            "call compile format [\"random [%1,%2,%3]\", 0,1,2]",
            
            // With comments
            "random [0,1,2] // Comment",
            "// Comment before\nrandom [0,1,2]",
            
            // Complex contexts
            "if (random [0,1,2] > 0.5) then { hint \"High\"; };",
            "[random [0,0.5,1], random [1,2,3]] call _fnc;",
        ];

        for input in valid_cases {
            let range = parse_random_line(input).unwrap();
            assert!(range.min <= range.mid && range.mid <= range.max,
                   "Invalid range order for input: {}", input);
        }

        // Test cases that should return None
        let none_cases = vec![
            "",                                 // Empty line
            "    ",                            // Whitespace only
            "// random [0,1,2]",               // Comment
            "/* random [0,1,2] */",            // Multi-line comment
            "\"random [0,1,2]\"",              // String literal
            "_random = [0,1,2];",              // Not a random command
            "randomize;",                      // Different command
            "Random[0,1,2]",                   // Wrong case
        ];

        for input in none_cases {
            assert_eq!(parse_random_line(input), None,
                      "Should return None for: {}", input);
        }

        // Test invalid formats (should return None)
        let invalid_cases = vec![
            "random(0,1,2)",                   // Wrong brackets
            "random [0,1]",                    // Too few values
            "random [0,1,2,3]",                // Too many values
            "random [a,b,c]",                  // Non-numeric
            "random [0, 1, max _x]",           // Invalid expression
            "random [.,.,.}",                  // Invalid syntax
        ];

        for input in invalid_cases {
            assert_eq!(parse_random_line(input), None,
                      "Should return None for invalid input: {}", input);
        }
    }
} 