use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, space0, space1, multispace0},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, map_res, opt, value},
    multi::{many0, many_till},
    Parser,
};
use crate::parsers::sqm::models::{ClassDefinition, CargoDefinition, ItemEntry};
use crate::parsers::sqm::parsers::weapon_parsers::{string_literal, numeric_value, boolean_value};

/// Parse an item entry in cargo
pub fn parse_item_entry(input: &str) -> IResult<&str, ItemEntry> {
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = tag("class").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('{').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    let mut name = String::new();
    let mut count = 1;
    
    let (input, _) = map(
        many_till(
            alt((
                map(
                    preceded(
                        (multispace0, tag("name"), space0, char('='), space0),
                        delimited(char('"'), take_until("\""), char('"'))
                    ),
                    |n: &str| name = n.to_string()
                ),
                map(
                    preceded(
                        (multispace0, tag("count"), space0, char('='), space0),
                        numeric_value
                    ),
                    |c| count = c
                ),
                map(
                    preceded(
                        multispace0,
                        take_until(";")
                    ),
                    |_| ()
                )
            )),
            preceded(multispace0, char('}'))
        ),
        |_| ()
    ).parse(input)?;
    
    Ok((input, ItemEntry { name, count }))
}

/// Parse cargo definition
pub fn parse_cargo(input: &str) -> IResult<&str, CargoDefinition> {
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = tag("class").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("TransportItems").parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('{').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    let (input, items) = many0(parse_item_entry).parse(input)?;
    
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('}').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    Ok((input, CargoDefinition { items }))
}

/// Parse a class definition
pub fn parse_class(input: &str) -> IResult<&str, ClassDefinition> {
    let (input, class_type) = preceded(
        (multispace0, tag("class"), space1),
        take_while1(|c: char| c.is_alphanumeric() || c == '_')
    ).parse(input)?;
    
    let (input, _) = preceded(
        multispace0,
        char('{')
    ).parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    
    let mut type_name = None;
    let mut is_backpack = None;
    let mut cargo = None;
    let mut nested_classes = Vec::new();
    
    let (input, _) = map(
        many_till(
            alt((
                map(
                    preceded(
                        (multispace0, tag("type"), space0, char('='), space0),
                        delimited(char('"'), take_until("\""), char('"'))
                    ),
                    |t: &str| type_name = Some(t.to_string())
                ),
                map(
                    preceded(
                        (multispace0, tag("isBackpack"), space0, char('='), space0),
                        boolean_value
                    ),
                    |b| is_backpack = Some(b)
                ),
                map(parse_cargo, |c| cargo = Some(c)),
                map(parse_class, |c| nested_classes.push(c)),
                map(
                    preceded(
                        multispace0,
                        take_until(";")
                    ),
                    |_| ()
                )
            )),
            preceded(multispace0, char('}'))
        ),
        |_| ()
    ).parse(input)?;
    
    Ok((input, ClassDefinition {
        class_type: class_type.to_string(),
        type_name,
        is_backpack,
        cargo,
        nested_classes,
    }))
}

/// Extract items from a class definition
pub fn extract_items(class_def: &ClassDefinition) -> Vec<ItemEntry> {
    let mut items = Vec::new();
    
    // Add items from cargo if present
    if let Some(cargo) = &class_def.cargo {
        items.extend(cargo.items.clone());
    }
    
    // Recursively extract items from nested classes
    for nested in &class_def.nested_classes {
        items.extend(extract_items(nested));
    }
    
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo() {
        // Skip this test for now
    }

    #[test]
    fn test_parse_class() {
        // Skip this test for now
    }
} 