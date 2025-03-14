use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1, none_of},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone)]
pub struct ClassReference {
    pub name: String,
    pub class_type: ClassType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassType {
    Weapon,
    Uniform,
    Vest,
    Backpack,
    Item,
    Magazine,
}

#[derive(Debug, PartialEq)]
pub struct InventoryClass {
    pub parent_class: String,
    pub references: Vec<ClassReference>,
}

fn parse_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((
        map(multispace1, |_| ()),
        map(preceded(tag("//"), take_until("\n")), |_| ()),
        map(delimited(tag("/*"), take_until("*/"), tag("*/")), |_| ()),
    )))(input)?;
    Ok((input, ()))
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    preceded(
        parse_whitespace_and_comments,
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"))))
        ))
    )(input)
}

fn parse_string_value(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(
        char('"'),
        recognize(many0(none_of("\""))),
        char('"')
    )(input)?;
    Ok((input, content.to_string()))
}

fn parse_class_type(name: &str) -> Option<ClassType> {
    if name.contains("weap_") {
        Some(ClassType::Weapon)
    } else if name.starts_with("U_") || name.contains("uniform") {
        Some(ClassType::Uniform)
    } else if name.contains("vest") {
        Some(ClassType::Vest)
    } else if name.contains("backpack") || name.ends_with("pack") {
        Some(ClassType::Backpack)
    } else if name.contains("mag_") || name.ends_with("magazine") {
        Some(ClassType::Magazine)
    } else {
        Some(ClassType::Item)
    }
}

fn parse_name_or_value_property(input: &str) -> IResult<&str, String> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = alt((
        tag("name"),
        tag("typeName"),
        tag("headgear"),
        tag("uniform"),
        tag("vest"),
        tag("backpack")
    ))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = parse_string_value(input)?;
    let (input, _) = opt(char(';'))(input)?;
    Ok((input, name))
}

fn find_closing_brace(input: &str) -> Option<usize> {
    let mut brace_count = 1;
    let mut chars = input.char_indices();
    
    while let Some((idx, c)) = chars.next() {
        match c {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_class_content(input: &str) -> IResult<&str, Vec<ClassReference>> {
    let mut refs = Vec::new();
    let mut remaining = input;

    while let Some(idx) = remaining.find(|c| c == 'n' || c == 't' || c == 'h' || c == 'u' || c == 'v' || c == 'b') {
        if let Ok((rest, name)) = parse_name_or_value_property(&remaining[idx..]) {
            refs.push(ClassReference {
                name: name.clone(),
                class_type: parse_class_type(&name).unwrap_or(ClassType::Item),
            });
            remaining = rest;
        } else {
            // Skip this character and continue searching
            if idx + 1 < remaining.len() {
                remaining = &remaining[idx + 1..];
            } else {
                break;
            }
        }
    }

    Ok((remaining, refs))
}

fn parse_inventory_section(input: &str) -> IResult<&str, Vec<ClassReference>> {
    let (input, _) = take_until("class Inventory")(input)?;
    let (input, _) = tag("class Inventory")(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = char('{')(input)?;
    
    let Some(end_idx) = find_closing_brace(input) else {
        return Ok((input, Vec::new()));
    };
    
    let content = &input[..end_idx];
    let (_, refs) = parse_class_content(content)?;
    let input = &input[end_idx + 1..];
    
    Ok((input, refs))
}

fn parse_attributes_section(input: &str) -> IResult<&str, Vec<ClassReference>> {
    let (input, _) = take_until("class Attributes")(input)?;
    let (input, _) = tag("class Attributes")(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = char('{')(input)?;
    
    let Some(end_idx) = find_closing_brace(input) else {
        return Ok((input, Vec::new()));
    };
    
    let content = &input[..end_idx];
    if content.contains("class Inventory") {
        let (_, refs) = parse_inventory_section(content)?;
        Ok((&input[end_idx + 1..], refs))
    } else {
        Ok((&input[end_idx + 1..], Vec::new()))
    }
}

fn parse_class_block(input: &str) -> IResult<&str, Vec<(String, Vec<ClassReference>)>> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = tag("class")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, class_name) = parse_identifier(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = char('{')(input)?;
    
    let Some(end_idx) = find_closing_brace(input) else {
        return Ok((input, Vec::new()));
    };
    
    let content = &input[..end_idx];
    let mut results = Vec::new();
    
    // Check if this class has an Attributes section with Inventory
    if content.contains("class Attributes") {
        if let Ok((_, refs)) = parse_attributes_section(content) {
            if !refs.is_empty() {
                results.push((class_name.to_string(), refs));
            }
        }
    }
    
    // Process nested classes
    let mut remaining = content;
    while let Some(idx) = remaining.find("class ") {
        let next_part = &remaining[idx..];
        if let Ok((rest, nested_results)) = parse_class_block(next_part) {
            results.extend(nested_results);
            if rest.len() < remaining.len() {
                remaining = rest;
            } else {
                // Avoid infinite loop
                remaining = &remaining[idx + 6..];
            }
        } else {
            // If parsing fails, skip this class
            remaining = &remaining[idx + 6..];
        }
    }
    
    Ok((&input[end_idx + 1..], results))
}

fn parse_class_blocks(input: &str) -> IResult<&str, Vec<(String, Vec<ClassReference>)>> {
    let mut classes = Vec::new();
    let mut remaining = input;

    while !remaining.trim().is_empty() {
        if let Some(idx) = remaining.find("class ") {
            if let Ok((rest, mut results)) = parse_class_block(&remaining[idx..]) {
                classes.append(&mut results);
                remaining = rest;
            } else {
                // If parsing fails, skip this class
                remaining = &remaining[idx + 6..];
            }
        } else {
            break;
        }
    }

    Ok((remaining, classes))
}

pub fn parse_sqm(input: &str) -> IResult<&str, Vec<InventoryClass>> {
    let (input, classes) = parse_class_blocks(input)?;
    
    let inventory_classes = classes
        .into_iter()
        .map(|(parent_class, references)| InventoryClass {
            parent_class,
            references,
        })
        .collect();
    
    Ok((input, inventory_classes))
}

pub fn extract_class_dependencies(sqm_content: &str) -> HashSet<String> {
    let mut dependencies = HashSet::new();
    
    if let Ok((_, inventory_classes)) = parse_sqm(sqm_content) {
        for class in inventory_classes {
            for reference in class.references {
                dependencies.insert(reference.name);
            }
        }
    }
    
    dependencies
}