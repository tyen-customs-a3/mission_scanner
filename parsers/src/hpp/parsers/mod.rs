use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, space0, space1, digit1, alpha1, alphanumeric1},
    sequence::{delimited, preceded, terminated},
    branch::alt,
    combinator::{map, map_res, opt},
    multi::separated_list0,
    error::Error,
    Parser,
};

use crate::hpp::models::{ItemReference, MedicalItemProperties};
use std::num::NonZeroU32;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse a LIST_X macro, e.g. LIST_2("ACE_fieldDressing")
pub fn list_macro(input: &str) -> IResult<&str, ItemReference> {
    let (input, _) = tag("LIST_").parse(input)?;
    let (input, count_str) = digit1.parse(input)?;
    let (input, item_name) = delimited(
        preceded(space0::<&str, Error<&str>>, char('(')),
        preceded(space0::<&str, Error<&str>>, string_literal),
        preceded(space0::<&str, Error<&str>>, char(')'))
    ).parse(input)?;
    
    let count = count_str.parse::<u32>().unwrap_or(1);
    Ok((input, ItemReference {
        item_name: item_name.to_string(),
        count: NonZeroU32::new(count),
    }))
}

/// Parse an array of items, e.g. { "Item1", LIST_2("Item2"), "Item3" }
pub fn items_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    delimited(
        char('{'),
        separated_list0(
            preceded(space0::<&str, Error<&str>>, char(',')),
            preceded(
                space0::<&str, Error<&str>>,
                alt((
                    map(string_literal, |s| ItemReference {
                        item_name: s.to_string(),
                        count: NonZeroU32::new(1),
                    }),
                    list_macro
                ))
            )
        ),
        preceded(space0::<&str, Error<&str>>, char('}'))
    ).parse(input)
}

/// Parse magazines array, same format as items_array
pub fn magazines_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    items_array(input)
}

/// Parse backpack items array, same format as items_array
pub fn backpack_items_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    items_array(input)
}

/// Parse medical item properties class
pub fn medical_item_properties(input: &str) -> IResult<&str, MedicalItemProperties> {
    let (input, _) = preceded(space0::<&str, Error<&str>>, tag("class")).parse(input)?;
    let (input, class_name) = preceded(space1, take_while1(|c: char| c.is_alphanumeric() || c == '_')).parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char('{')).parse(input)?;
    
    let mut result = MedicalItemProperties {
        class_name: class_name.to_string(),
        pain_reduce: None,
        hr_increase_low: None,
        hr_increase_normal: None,
        hr_increase_high: None,
        time_in_system: None,
        time_till_max_effect: None,
        max_dose: None,
        viscosity_change: None,
    };
    
    let (input, _) = preceded(space0::<&str, Error<&str>>, terminated(
        separated_list0(
            preceded(space0::<&str, Error<&str>>, char(';')),
            preceded(space0::<&str, Error<&str>>, alt((
                map(
                    preceded(tag("painReduce"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric() || c == '.' || c == '-'))))),
                    |v| result.pain_reduce = v.parse().ok()
                ),
                map(
                    preceded(tag("hrIncreaseLow[]"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, delimited(char('{'), separated_list0(preceded(space0::<&str, Error<&str>>, char(',')), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric() || c == '-'))), char('}')))))),
                    |v| result.hr_increase_low = Some(v.iter().filter_map(|s| s.parse().ok()).collect())
                ),
                map(
                    preceded(tag("hrIncreaseNormal[]"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, delimited(char('{'), separated_list0(preceded(space0::<&str, Error<&str>>, char(',')), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric() || c == '-'))), char('}')))))),
                    |v| result.hr_increase_normal = Some(v.iter().filter_map(|s| s.parse().ok()).collect())
                ),
                map(
                    preceded(tag("hrIncreaseHigh[]"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, delimited(char('{'), separated_list0(preceded(space0::<&str, Error<&str>>, char(',')), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric() || c == '-'))), char('}')))))),
                    |v| result.hr_increase_high = Some(v.iter().filter_map(|s| s.parse().ok()).collect())
                ),
                map(
                    preceded(tag("timeInSystem"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric()))))),
                    |v| result.time_in_system = v.parse().ok()
                ),
                map(
                    preceded(tag("timeTillMaxEffect"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric()))))),
                    |v| result.time_till_max_effect = v.parse().ok()
                ),
                map(
                    preceded(tag("maxDose"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric()))))),
                    |v| result.max_dose = v.parse().ok()
                ),
                map(
                    preceded(tag("viscosityChange"), preceded(space0::<&str, Error<&str>>, preceded(char('='), preceded(space0::<&str, Error<&str>>, take_while1(|c: char| c.is_numeric() || c == '-'))))),
                    |v| result.viscosity_change = v.parse().ok()
                ),
            )))
        ),
        preceded(space0::<&str, Error<&str>>, char(';'))
    )).parse(input)?;
    
    let (input, _) = preceded(space0::<&str, Error<&str>>, char('}')).parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    #[test]
    fn test_string_literal() {
        // Basic cases
        assert_eq!(string_literal(r#""ACE_fieldDressing""#).unwrap(), ("", "ACE_fieldDressing"));
        assert_eq!(string_literal(r#""ACRE_PRC343""#).unwrap(), ("", "ACRE_PRC343"));

        // With special characters
        assert_eq!(string_literal(r#""ACE_field-Dressing_2.0""#).unwrap(), ("", "ACE_field-Dressing_2.0"));
        assert_eq!(string_literal(r#""Item With Spaces""#).unwrap(), ("", "Item With Spaces"));
        assert_eq!(string_literal(r#""Item/With/Slashes""#).unwrap(), ("", "Item/With/Slashes"));

        // With trailing content
        assert_eq!(string_literal(r#""ACE_fieldDressing"rest"#).unwrap(), ("rest", "ACE_fieldDressing"));

        // Special characters and edge cases
        assert_eq!(string_literal(r#""Item with \"quotes\" inside""#).unwrap(), ("", r#"Item with \"quotes\" inside"#));
        assert_eq!(string_literal(r#""Item with \n newline""#).unwrap(), ("", r#"Item with \n newline"#));
        assert_eq!(string_literal(r#""!@#$%^&*()""#).unwrap(), ("", "!@#$%^&*()"));
        assert_eq!(string_literal(r#""١٢٣""#).unwrap(), ("", "١٢٣")); // Non-ASCII characters
        assert_eq!(string_literal(r#""🎮🎲""#).unwrap(), ("", "🎮🎲")); // Emojis

        // Whitespace handling
        assert_eq!(string_literal(r#""   ""#).unwrap(), ("", "   ")); // Only spaces
        assert_eq!(string_literal(r#""Item with tabs	and spaces""#).unwrap(), ("", "Item with tabs	and spaces"));
        assert_eq!(string_literal(r#""Item with
            newlines""#).unwrap(), ("", "Item with\n            newlines"));

        // Error cases
        assert!(string_literal(r#""Unclosed string"#).is_err());
        assert!(string_literal(r#"Not a string"#).is_err());
        assert!(string_literal(r#""""#).unwrap() == ("", ""));  // Empty string is valid
        assert!(string_literal(r#""\"#).is_err()); // Single backslash
        assert!(string_literal("").is_err()); // Empty input
    }

    #[test]
    fn test_list_macro() {
        // Basic cases
        let (rest, result) = list_macro(r#"LIST_2("ACE_fieldDressing")"#).unwrap();
        assert_eq!(result.item_name, "ACE_fieldDressing");
        assert_eq!(result.count, NonZeroU32::new(2));
        assert_eq!(rest, "");

        // Edge cases for numbers
        let test_cases = vec![
            ("LIST_1", 1),
            ("LIST_0", 1),  // Should default to 1
            ("LIST_4294967295", 4294967295),  // u32::MAX
            ("LIST_00001", 1),  // Leading zeros
            ("LIST_042", 42),   // Octal-looking number
        ];

        for (prefix, expected) in test_cases {
            let input = format!(r#"{}("ACE_fieldDressing")"#, prefix);
            let (_, result) = list_macro(&input).unwrap();
            assert_eq!(result.count, NonZeroU32::new(expected), "Failed for input: {}", input);
        }

        // Whitespace variations
        let whitespace_cases = vec![
            r#"LIST_2 ("ACE_fieldDressing")"#,
            r#"LIST_2( "ACE_fieldDressing")"#,
            r#"LIST_2("ACE_fieldDressing" )"#,
            r#"LIST_2 ( "ACE_fieldDressing" ) "#,
            r#"LIST_2	("ACE_fieldDressing")"#,  // Tab
        ];

        for input in whitespace_cases {
            let result = list_macro(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let (_, item) = result.unwrap();
            assert_eq!(item.count, NonZeroU32::new(2));
        }

        // Error cases
        let error_cases = vec![
            r#"LIST("ACE_fieldDressing")"#,         // Missing number
            r#"LIST_("ACE_fieldDressing")"#,        // Missing number
            r#"LIST_2"ACE_fieldDressing")"#,        // Missing opening parenthesis
            r#"LIST_2("ACE_fieldDressing"#,         // Missing closing parenthesis
            r#"LIST_-1("ACE_fieldDressing")"#,      // Negative number
            r#"LIST_2.5("ACE_fieldDressing")"#,     // Decimal number
            r#"LIST_4294967296("ACE_fieldDressing")"#, // Beyond u32::MAX
            r#"LIST_2(ACE_fieldDressing)"#,         // Missing quotes
            r#"LIST_2()"#,                          // Empty parentheses
            r#"LIST_a("ACE_fieldDressing")"#,       // Non-numeric count
        ];

        for input in error_cases {
            assert!(list_macro(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_item_array() {
        // Basic case
        let input = r#"{
            "ACRE_PRC343",
            LIST_2("ACE_fieldDressing"),
            "ACE_morphine"
        }"#;
        let (_, result) = items_array(input).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].item_name, "ACRE_PRC343");
        assert_eq!(result[0].count, NonZeroU32::new(1));
        assert_eq!(result[1].item_name, "ACE_fieldDressing");
        assert_eq!(result[1].count, NonZeroU32::new(2));

        // Empty array
        assert_eq!(items_array("{}").unwrap().1.len(), 0);
        assert_eq!(items_array("{ }").unwrap().1.len(), 0);

        // Single item array
        let (_, result) = items_array(r#"{"AFCE_fieldDressing"}"#).unwrap();
        assert_eq!(result.len(), 1);

        // Array with various whitespace and comma patterns
        let inputs = vec![
            r#"{,"ACE_fieldDressing",}"#,  // Leading/trailing comma
            r#"{"ACE_fieldDressing"    }"#,  // Extra whitespace
            r#"{    "ACE_fieldDressing"}"#,  // Leading whitespace
            r#"{"ACE_fieldDressing",}"#,     // Trailing comma
        ];
        for input in inputs {
            let (_, result) = items_array(input).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].item_name, "ACE_fieldDressing");
        }

        // Complex nested cases
        let input = r#"{
            LIST_2("Item with, comma"),
            "Item with } brace",
            LIST_1("Item with { brace"),
            "Item with []",
            LIST_5("Item with ()"),
            "Last Item"
        }"#;
        let (_, result) = items_array(input).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].item_name, "Item with, comma");
        assert_eq!(result[1].item_name, "Item with } brace");

        // Array formatting variations
        let format_cases = vec![
            r#"{}"#,                              // Empty array
            r#"{,}"#,                             // Empty with comma
            r#"{"Item"}"#,                        // Single item
            r#"{"Item",}"#,                       // Trailing comma
            r#"{,"Item"}"#,                       // Leading comma
            r#"{,,"Item",,"Another",}"#,          // Multiple commas
            r#"{    "Item"    }"#,               // Extra spaces
            r#"{"Item"
            }"#,                                  // Newlines
            r#"{LIST_1("Item"),LIST_2("Other")}"#, // No spaces
        ];

        for input in format_cases {
            assert!(items_array(input).is_ok(), "Failed to parse: {}", input);
        }

        // Error cases
        let error_cases = vec![
            r#"{"Unclosed"#,           // Unclosed array
            r#""Not an array""#,       // Not an array
            r#"[Wrong brackets]"#,     // Wrong brackets
            r#"{"Missing quote}"#,     // Missing quotes
            r#"{"Invalid" "Comma" }"#, // Missing comma
            r#"{ LIST_2 }"#,          // Invalid LIST macro
            r#"{{Nested}}"#,          // Nested arrays
        ];

        for input in error_cases {
            assert!(items_array(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_medical_item_properties() {
        // Complete case
        let input = r#"
        class Morphine {
            painReduce = 0.8;
            hrIncreaseLow[] = {-10, -20};
            hrIncreaseNormal[] = {-10, -30};
            hrIncreaseHigh[] = {-10, -35};
            timeInSystem = 1800;
            timeTillMaxEffect = 30;
            maxDose = 4;
            viscosityChange = -10;
        };"#;
        
        let (_, result) = medical_item_properties(input).unwrap();
        assert_eq!(result.class_name, "Morphine");
        assert_eq!(result.pain_reduce, Some(0.8));
        assert_eq!(result.hr_increase_low, Some(vec![-10, -20]));
        assert_eq!(result.time_in_system, Some(1800));

        // Minimal case (only required fields)
        let input = r#"
        class Bandage {
            timeInSystem = 0;
        };"#;
        
        let (_, result) = medical_item_properties(input).unwrap();
        assert_eq!(result.class_name, "Bandage");
        assert_eq!(result.time_in_system, Some(0));
        assert_eq!(result.pain_reduce, None);

        // With extra whitespace and comments
        let input = r#"
        class    Morphine    {
            // Comment
            painReduce    =     0.8   ;
            timeInSystem=1800;  // Inline comment
        };"#;
        
        let (_, result) = medical_item_properties(input).unwrap();
        assert_eq!(result.class_name, "Morphine");
        assert_eq!(result.pain_reduce, Some(0.8));
        assert_eq!(result.time_in_system, Some(1800));

        // Test all possible combinations of optional fields
        let test_cases = vec![
            // Minimal valid case
            r#"class Test { timeInSystem = 0; };"#,
            
            // All fields present
            r#"class Full {
                painReduce = 0.8;
                hrIncreaseLow[] = {-10, -20};
                hrIncreaseNormal[] = {-10, -30};
                hrIncreaseHigh[] = {-10, -35};
                timeInSystem = 1800;
                timeTillMaxEffect = 30;
                maxDose = 4;
                viscosityChange = -10;
            };"#,
            
            // Different numeric formats
            r#"class Numbers {
                painReduce = 0.0000001;
                timeInSystem = 000123;
                timeTillMaxEffect = +30;
                maxDose = 0004;
                viscosityChange = -0010;
            };"#,
            
            // Array variations
            r#"class Arrays {
                hrIncreaseLow[] = {-10};
                hrIncreaseNormal[] = {-10,-20,-30};
                hrIncreaseHigh[] = {+10,+20,+30};
            };"#,
        ];

        for input in test_cases {
            assert!(medical_item_properties(input).is_ok(), "Failed to parse: {}", input);
        }

        // Test invalid cases
        let error_cases = vec![
            // Missing required fields
            r#"class Empty { };"#,
            
            // Invalid class name
            r#"class 123Invalid { timeInSystem = 0; };"#,
            
            // Invalid numeric values
            r#"class Invalid { timeInSystem = 1.1.1; };"#,
            r#"class Invalid { timeInSystem = -; };"#,
            r#"class Invalid { timeInSystem = 1e6; };"#,
            
            // Invalid array formats
            r#"class Invalid { hrIncreaseLow[] = {1,}; };"#,
            r#"class Invalid { hrIncreaseLow[] = {,1}; };"#,
            r#"class Invalid { hrIncreaseLow[] = {1.1}; };"#,
            
            // Syntax errors
            r#"class Invalid { timeInSystem = 0 }"#,  // Missing semicolon
            r#"class Invalid { timeInSystem = ; };"#, // Missing value
            r#"class { timeInSystem = 0; };"#,       // Missing class name
        ];

        for input in error_cases {
            assert!(medical_item_properties(input).is_err(), "Should fail to parse: {}", input);
        }
    }
} 