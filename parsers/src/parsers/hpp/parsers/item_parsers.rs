use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, space0, space1, multispace0},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, map_res, opt},
    multi::{separated_list0, separated_list1},
    Parser,
    error::{ParseError, ErrorKind},
};
use crate::parsers::hpp::models::{ItemReference, MedicalItemProperties};
use std::num::NonZeroU32;

/// Result type for our parsers
pub type ParseResult<'a, T> = IResult<&'a str, T>;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> ParseResult<&str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse a LIST macro, which defines an item with a count
/// Example: LIST_2("ACE_fieldDressing")
pub fn list_macro(input: &str) -> ParseResult<ItemReference> {
    let (input, _) = tag("LIST_").parse(input)?;
    let (input, count_str) = digit1.parse(input)?;
    let (input, _) = char('(').parse(input)?;
    let (input, item_name) = string_literal(input)?;
    let (input, _) = char(')').parse(input)?;
    
    let count = count_str.parse::<u32>().unwrap_or(1);
    Ok((input, ItemReference::new(item_name, count).unwrap_or_else(|| ItemReference::single(item_name))))
}

/// Parse a single item (just a string literal)
pub fn single_item(input: &str) -> ParseResult<ItemReference> {
    let (input, item_name) = string_literal(input)?;
    Ok((input, ItemReference::single(item_name)))
}

/// Parse an item entry, which can be either a LIST macro or a single item
pub fn item_entry(input: &str) -> IResult<&str, ItemReference> {
    let (input, _) = space0.parse(input)?;
    alt((
        list_macro,
        single_item
    )).parse(input)
}

/// Parse an array of items
pub fn item_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    // Use separated_list0 for a more robust parsing of comma-separated items
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('{').parse(input)?;
    let (input, items) = separated_list0(
        preceded(space0, char(',')),
        preceded(space0, item_entry)
    ).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('}').parse(input)?;
    
    Ok((input, items))
}

/// Parse an items array from an HPP file
pub fn items_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    let (input, _) = space0.parse(input)?;
    let (input, _) = tag("items[]").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, items) = item_array(input)?;
    
    Ok((input, items))
}

/// Parse a magazines array from an HPP file
pub fn magazines_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    let (input, _) = space0.parse(input)?;
    let (input, _) = tag("magazines[]").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, items) = item_array(input)?;
    
    Ok((input, items))
}

/// Parse a backpack items array from an HPP file
pub fn backpack_items_array(input: &str) -> IResult<&str, Vec<ItemReference>> {
    let (input, _) = space0.parse(input)?;
    let (input, _) = tag("backpackItems[]").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, items) = item_array(input)?;
    
    Ok((input, items))
}

/// Parse a floating point number
pub fn float_value(input: &str) -> IResult<&str, f32> {
    map_res(
        take_while1(|c: char| c.is_numeric() || c == '.' || c == '-'),
        |s: &str| s.parse::<f32>()
    ).parse(input)
}

/// Parse an array of integers
pub fn int_array(input: &str) -> IResult<&str, Vec<i32>> {
    delimited(
        char('{'),
        separated_list0(
            preceded(space0, char(',')),
            preceded(
                space0,
                map_res(
                    take_while1(|c: char| c.is_numeric() || c == '-'),
                    |s: &str| s.parse::<i32>()
                )
            )
        ),
        preceded(space0, char('}'))
    ).parse(input)
}

/// Parse a numeric value (float or int)
fn numeric_value(input: &str) -> IResult<&str, Value> {
    let (input, _) = multispace0.parse(input)?;
    let (input, value_str) = take_while1(|c: char| c.is_numeric() || c == '.' || c == '-').parse(input)?;
    let value_str = value_str.trim();
    
    // Try parsing as int first
    if let Ok(int_val) = value_str.parse::<i32>() {
        Ok((input, Value::Int(int_val)))
    } else if let Ok(float_val) = value_str.parse::<f32>() {
        Ok((input, Value::Float(float_val)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Float
        )))
    }
}

/// Parse a key-value pair
fn parse_key_value_pair(input: &str) -> IResult<&str, (&str, Value)> {
    let (input, _) = multispace0.parse(input)?;
    let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;
    let key = key.trim();
    let (input, _) = multispace0.parse(input)?;
    let (input, has_array) = opt(tag("[]")).parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    let (input, value) = if has_array.is_some() {
        let (input, values) = int_array(input)?;
        (input, Value::Array(values))
    } else {
        numeric_value(input)?
    };
    
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char(';').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    Ok((input, (key, value)))
}

/// Parse medical item properties
pub fn medical_item_properties(input: &str) -> IResult<&str, MedicalItemProperties> {
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = tag("class").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, class_name) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('{').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    let mut pain_reduce = None;
    let mut hr_increase_low = None;
    let mut hr_increase_normal = None;
    let mut hr_increase_high = None;
    let mut time_in_system = None;
    let mut time_till_max_effect = None;
    let mut max_dose = None;
    let mut viscosity_change = None;
    
    let mut remaining = input;
    while let Ok((rest, (key, value))) = parse_key_value_pair(remaining) {
        match (key, value) {
            ("painReduce", Value::Float(v)) => pain_reduce = Some(v),
            ("timeInSystem", Value::Int(v)) => time_in_system = Some(v as u32),
            ("timeTillMaxEffect", Value::Int(v)) => time_till_max_effect = Some(v as u32),
            ("maxDose", Value::Int(v)) => max_dose = Some(v as u32),
            ("viscosityChange", Value::Int(v)) => viscosity_change = Some(v),
            ("hrIncreaseLow", Value::Array(v)) => hr_increase_low = Some(v),
            ("hrIncreaseNormal", Value::Array(v)) => hr_increase_normal = Some(v),
            ("hrIncreaseHigh", Value::Array(v)) => hr_increase_high = Some(v),
            _ => {}
        }
        remaining = rest;
    }
    
    let (input, _) = multispace0.parse(remaining)?;
    let (input, _) = char('}').parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, MedicalItemProperties {
        class_name: class_name.to_string(),
        pain_reduce,
        hr_increase_low,
        hr_increase_normal,
        hr_increase_high,
        time_in_system,
        time_till_max_effect,
        max_dose,
        viscosity_change,
    }))
}

#[derive(Debug)]
enum Value {
    Float(f32),
    Int(i32),
    Array(Vec<i32>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_literal() {
        // Basic case
        let input = r#""ACE_fieldDressing""#;
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(result, "ACE_fieldDressing");
        assert_eq!(rest, "");

        // With spaces
        let input = r#""ACE field Dressing""#;
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(result, "ACE field Dressing");
        assert_eq!(rest, "");

        // With special characters
        let input = r#""ACE_field-Dressing_2.0""#;
        let (rest, result) = string_literal(input).unwrap();
        assert_eq!(result, "ACE_field-Dressing_2.0");
        assert_eq!(rest, "");

        // Should fail on unclosed quotes
        assert!(string_literal(r#""ACE_fieldDressing"#).is_err());
    }

    #[test]
    fn test_list_macro() {
        // Basic case
        let input = r#"LIST_2("ACE_fieldDressing")"#;
        let (rest, result) = list_macro(input).unwrap();
        assert_eq!(result.item_name, "ACE_fieldDressing");
        assert_eq!(result.count, NonZeroU32::new(2));
        assert_eq!(rest, "");

        // Zero count should default to 1
        let input = r#"LIST_0("ACE_fieldDressing")"#;
        let (rest, result) = list_macro(input).unwrap();
        assert_eq!(result.count, NonZeroU32::new(1));

        // Large numbers
        let input = r#"LIST_999("ACE_fieldDressing")"#;
        let (rest, result) = list_macro(input).unwrap();
        assert_eq!(result.count, NonZeroU32::new(999));
    }

    #[test]
    fn test_single_item() {
        let input = r#""ACE_fieldDressing""#;
        let (rest, result) = single_item(input).unwrap();
        assert_eq!(result.item_name, "ACE_fieldDressing");
        assert_eq!(result.count, Some(1));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_item_entry() {
        let input1 = r#""ACE_fieldDressing""#;
        let (rest1, result1) = item_entry(input1).unwrap();
        assert_eq!(result1.item_name, "ACE_fieldDressing");
        assert_eq!(result1.count, Some(1));
        assert_eq!(rest1, "");

        let input2 = r#"LIST_2("ACE_fieldDressing")"#;
        let (rest2, result2) = item_entry(input2).unwrap();
        assert_eq!(result2.item_name, "ACE_fieldDressing");
        assert_eq!(result2.count, Some(2));
        assert_eq!(rest2, "");
    }

    #[test]
    fn test_item_array() {
        let input = r#"{"ACE_fieldDressing", LIST_2("ACE_packingBandage"), "ACE_morphine"}"#;
        let (rest, result) = item_array(input).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].item_name, "ACE_fieldDressing");
        assert_eq!(result[0].count, Some(1));
        assert_eq!(result[1].item_name, "ACE_packingBandage");
        assert_eq!(result[1].count, Some(2));
        assert_eq!(result[2].item_name, "ACE_morphine");
        assert_eq!(result[2].count, Some(1));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_items_array() {
        let input = r#"items[] = {"ACE_fieldDressing", LIST_2("ACE_packingBandage")}"#;
        let (rest, result) = items_array(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].item_name, "ACE_fieldDressing");
        assert_eq!(result[0].count, Some(1));
        assert_eq!(result[1].item_name, "ACE_packingBandage");
        assert_eq!(result[1].count, Some(2));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_medical_item_properties() {
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
        assert_eq!(result.hr_increase_normal, Some(vec![-10, -30]));
        assert_eq!(result.hr_increase_high, Some(vec![-10, -35]));
        assert_eq!(result.time_in_system, Some(1800));
        assert_eq!(result.time_till_max_effect, Some(30));
        assert_eq!(result.max_dose, Some(4));
        assert_eq!(result.viscosity_change, Some(-10));
    }
} 