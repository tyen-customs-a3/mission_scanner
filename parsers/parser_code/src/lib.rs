use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1, none_of, space0},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    branch::alt,
    IResult,
    error::{Error, ErrorKind},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Equipment {
    pub class_name: String,
    pub parent_class: Option<String>,
    pub properties: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq)]
enum ListItem {
    Single(String),
    Repeated(usize, String),
}

#[derive(Debug, PartialEq)]
enum PropertyValue {
    String(String),
    List(Vec<String>),
}

#[derive(Debug, PartialEq)]
enum PropertyOperation {
    Set(PropertyValue),
    Append(Vec<String>),
}

/// Parses whitespace and comments
fn parse_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((
        map(take_while1(|c: char| c.is_whitespace()), |_| ()),
        map(preceded(tag("//"), take_while1(|c| c != '\n' && c != '\r')), |_| ()),
        map(delimited(tag("/*"), take_until("*/"), tag("*/")), |_| ()),
    )))(input)?;
    Ok((input, ()))
}

/// Parses an identifier that can contain alphanumeric characters and underscores
fn parse_identifier(input: &str) -> IResult<&str, &str> {
    preceded(
        parse_whitespace_and_comments,
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"))))
        ))
    )(input)
}

/// Parses array brackets with optional whitespace
fn parse_array_brackets(input: &str) -> IResult<&str, ()> {
    value(
        (),
        delimited(
            parse_whitespace_and_comments,
            char('['),
            delimited(
                parse_whitespace_and_comments,
                char(']'),
                parse_whitespace_and_comments,
            )
        )
    )(input)
}

/// Parses a LIST macro of the form LIST_n("item")
fn parse_list_macro(input: &str) -> IResult<&str, ListItem> {
    let (input, _) = tag("LIST_")(input)?;
    let (input, count_str) = digit1(input)?;
    let count = count_str.parse::<usize>().map_err(|_| nom::Err::Error(Error::new(input, ErrorKind::Digit)))?;
    let (input, item) = delimited(
        char('('),
        delimited(char('"'), take_until("\""), char('"')),
        char(')'),
    )(input)?;
    Ok((input, ListItem::Repeated(count, item.to_string())))
}

/// Parses a quoted string, handling escaped quotes
fn parse_quoted_string(input: &str) -> IResult<&str, ListItem> {
    let (input, content) = delimited(
        char('"'),
        recognize(many0(alt((
            preceded(char('\\'), none_of("")),
            none_of("\"")
        )))),
        char('"')
    )(input)?;
    
    Ok((input, ListItem::Single(content.replace("\\\"", "\""))))
}

/// Parses either a LIST macro or a quoted string
fn parse_list_item(input: &str) -> IResult<&str, ListItem> {
    preceded(
        parse_whitespace_and_comments,
        alt((parse_list_macro, parse_quoted_string))
    )(input)
}

/// Parses a list of strings
fn parse_list(input: &str) -> IResult<&str, Vec<String>> {
    let (input, items) = delimited(
        char('{'),
        map(
            tuple((
                separated_list0(
                    delimited(
                        parse_whitespace_and_comments,
                        char(','),
                        parse_whitespace_and_comments
                    ),
                    parse_list_item,
                ),
                opt(preceded(
                    delimited(
                        parse_whitespace_and_comments,
                        char(','),
                        parse_whitespace_and_comments
                    ),
                    value((), parse_whitespace_and_comments)
                ))
            )),
            |(items, _)| items
        ),
        preceded(parse_whitespace_and_comments, char('}')),
    )(input)?;

    let expanded: Vec<String> = items
        .into_iter()
        .flat_map(|item| match item {
            ListItem::Single(s) => vec![s],
            ListItem::Repeated(count, s) => vec![s; count],
        })
        .collect();

    Ok((input, expanded))
}

/// Parses a property value (either a string or a list)
fn parse_property_value(input: &str) -> IResult<&str, PropertyOperation> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, operation) = opt(tag("+"))(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    
    let (input, value) = alt((
        map(parse_quoted_string, |item| match item {
            ListItem::Single(s) => PropertyOperation::Set(PropertyValue::String(s)),
            ListItem::Repeated(_, _) => unreachable!(),
        }),
        map(parse_list, |items| match operation {
            Some(_) => PropertyOperation::Append(items),
            None => PropertyOperation::Set(PropertyValue::List(items)),
        }),
    ))(input)?;
    
    Ok((input, value))
}

/// Parses a property with potential operation
fn parse_property(input: &str) -> IResult<&str, (String, PropertyOperation)> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, property_name) = parse_identifier(input)?;
    let (input, array_suffix) = opt(parse_array_brackets)(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, operation) = parse_property_value(input)?;
    let (input, _) = preceded(parse_whitespace_and_comments, opt(char(';')))(input)?;
    
    let operation = match (array_suffix, operation) {
        (Some(_), PropertyOperation::Set(PropertyValue::String(s))) => {
            PropertyOperation::Set(PropertyValue::List(vec![s]))
        },
        (_, op) => op,
    };
    
    Ok((input, (property_name.to_string(), operation)))
}

/// Parses class inheritance
fn parse_class_inheritance(input: &str) -> IResult<&str, Option<&str>> {
    preceded(
        parse_whitespace_and_comments,
        opt(preceded(
            tuple((char(':'), space0)),
            parse_identifier,
        ))
    )(input)
}

/// Parses a class definition
pub fn parse_class(input: &str) -> IResult<&str, Equipment> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = tag("class")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, class_name) = parse_identifier(input)?;
    let (input, parent_class) = parse_class_inheritance(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = char('{')(input)?;
    
    let (input, properties) = many0(parse_property)(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = char('}')(input)?;
    let (input, _) = preceded(parse_whitespace_and_comments, opt(char(';')))(input)?;

    let mut properties_map = HashMap::new();
    for (name, operation) in properties {
        match operation {
            PropertyOperation::Set(value) => {
                match value {
                    PropertyValue::String(s) => {
                        properties_map.insert(name, vec![s]);
                    },
                    PropertyValue::List(values) => {
                        properties_map.insert(name, values);
                    }
                }
            },
            PropertyOperation::Append(values) => {
                properties_map
                    .entry(name)
                    .and_modify(|e| e.extend(values.clone()))
                    .or_insert(values);
            }
        }
    }

    Ok((
        input,
        Equipment {
            class_name: class_name.to_string(),
            parent_class: parent_class.map(String::from),
            properties: properties_map,
        },
    ))
}

/// Parses a complete loadout file
pub fn parse_loadout(input: &str) -> IResult<&str, Vec<Equipment>> {
    let (input, classes) = preceded(
        parse_whitespace_and_comments,
        many0(delimited(
            parse_whitespace_and_comments,
            parse_class,
            parse_whitespace_and_comments,
        ))
    )(input)?;
    
    if classes.is_empty() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Many0)));
    }
    
    Ok((input, classes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_class() {
        let input = "class empty { };";
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(equipment.class_name, "empty");
        assert!(equipment.properties.is_empty());
    }

    #[test]
    fn test_parse_class_inheritance() {
        let input = "class derived : base { };";
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(equipment.class_name, "derived");
        assert_eq!(equipment.parent_class, Some("base".to_string()));
    }

    #[test]
    fn test_parse_property_append() {
        let input = r#"class test {
            items[] += {"Item1", "Item2"};
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec!["Item1".to_string(), "Item2".to_string()]
        );
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
        // Comment before class
        class test {
            /* Multi-line
               comment */
            items[] = {"Item"}; // End of line comment
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec!["Item".to_string()]
        );
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let input = r#"class test {
            items[] = {"Item with \"quotes\""};
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec!["Item with \"quotes\"".to_string()]
        );
    }

    #[test]
    fn test_parse_multiple_list_macros() {
        let input = r#"class test {
            items[] = {
                LIST_2("Item1"),
                "Single",
                LIST_3("Item2")
            };
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec![
                "Item1", "Item1",
                "Single",
                "Item2", "Item2", "Item2"
            ]
        );
    }

    #[test]
    fn test_parse_complex_property() {
        let input = r#"class test {
            items[] = {
                LIST_2("First"),
                "Middle",
                LIST_2("Last")
            };
            weapons[] += {
                LIST_3("Gun")
            };
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec!["First", "First", "Middle", "Last", "Last"]
        );
        assert_eq!(
            equipment.properties.get("weapons").unwrap(),
            &vec!["Gun", "Gun", "Gun"]
        );
    }

    #[test]
    fn test_parse_loadout() {
        let input = r#"
        class baseMan {
            linkedItems[] = {"ItemWatch"};
        };
        class rm : baseMan {
            uniform[] = {"uniform1"};
            vest[] = {"vest1"};
        };"#;
        let (_, equipment) = parse_loadout(input).unwrap();
        
        let base_man = equipment.iter().find(|e| e.class_name == "baseMan").unwrap();
        assert!(base_man.parent_class.is_none());
        assert!(base_man.properties.contains_key("linkedItems"));
        
        let rifleman = equipment.iter().find(|e| e.class_name == "rm").unwrap();
        assert_eq!(rifleman.parent_class, Some("baseMan".to_string()));
        assert!(rifleman.properties.contains_key("uniform"));
        assert!(rifleman.properties.contains_key("vest"));
    }

    #[test]
    fn test_parse_with_whitespace_variations() {
        let input = r#"class    test     {
                items[]={"Item"};
                weapons[]    =    {"Gun"};
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert_eq!(equipment.class_name, "test");
        assert_eq!(
            equipment.properties.get("items").unwrap(),
            &vec!["Item".to_string()]
        );
    }

    #[test]
    fn test_parse_empty_arrays() {
        let input = r#"class test {
            empty[] = {};
            trailing_comma[] = {
                "Item",
            };
        };"#;
        let (_, equipment) = parse_class(input).unwrap();
        assert!(equipment.properties.get("empty").unwrap().is_empty());
        assert_eq!(
            equipment.properties.get("trailing_comma").unwrap(),
            &vec!["Item".to_string()]
        );
    }

    #[test]
    fn test_parse_malformed_list_macro() {
        let input = r#"class test {
            items[] = {
                LIST_2("Good"),
                LIST_("Bad"),
                LIST_ABC("Bad")
            };
        };"#;
        let result = parse_class(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_inheritance() {
        let input = r#"
        class base {};
        class middle : base {};
        class derived : middle {
            items[] = {"Item"};
        };"#;
        let (_, classes) = parse_loadout(input).unwrap();
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[2].parent_class, Some("middle".to_string()));
    }

    #[test]
    fn test_parse_property_override() {
        let input = r#"
        class base {
            items[] = {"Base"};
        };
        class derived : base {
            items[] = {"Derived"};
        };"#;
        let (_, classes) = parse_loadout(input).unwrap();
        assert_eq!(
            classes[0].properties.get("items").unwrap(),
            &vec!["Base".to_string()]
        );
        assert_eq!(
            classes[1].properties.get("items").unwrap(),
            &vec!["Derived".to_string()]
        );
    }

    #[test]
    fn test_parse_real_loadout() {
        let loadout_content = std::fs::read_to_string("tests/data/loadout.hpp").expect("Unable to read file");
        let (_, equipment) = parse_loadout(&loadout_content).expect("Failed to parse loadout");
        
        // Verify base class
        let base_man = equipment.iter().find(|e| e.class_name == "baseMan").unwrap();
        assert!(base_man.parent_class.is_none());
        assert!(base_man.properties.contains_key("linkedItems"));
        
        // Verify derived class
        let rifleman = equipment.iter().find(|e| e.class_name == "rm").unwrap();
        assert_eq!(rifleman.parent_class, Some("baseMan".to_string()));
        assert!(rifleman.properties.contains_key("uniform"));
        assert!(rifleman.properties.contains_key("vest"));
    }

    #[test]
    fn test_comprehensive_loadout_patterns() {
        let input = r#"
        // Base class with empty arrays and string properties
        class baseLoadout {
            displayName = "Base Loadout";  // Simple string property
            uniform[] = {};                // Empty array
            vest[] = {};
            backpack[] = {};
            traits[] = {"trait1", "trait2"}; // Array of strings
        };

        // First level inheritance with single items
        class level1 : baseLoadout {
            displayName = "Level 1";
            uniform[] = {"uniform1"};          // Single item array
            vest[] = {"vest1",};              // Array with trailing comma
            backpack[] = {
                "backpack1"
            };
        };

        // Second level inheritance with multiple items and LIST macros
        class level2 : level1 {
            displayName = "Level 2";
            uniform[] = {                      // Multi-item array
                "uniform1",
                "uniform2",
                "uniform3"
            };
            vest[] = {                        // Mixed normal and LIST items
                LIST_2("vest1"),
                "vest2",
                LIST_3("vest3")
            };
            items[] = {                       // Multiple LIST macros
                LIST_10("item1"),
                LIST_5("item2")
            };
            linkedItems[] = {                 // Mixed with trailing comma
                "linked1",
                LIST_2("linked2"),
            };
        };

        // Third level with property appending
        class level3 : level2 {
            displayName = "Level 3";
            backpack[] = {"newbackpack"};     // Override property
            items[] += {                      // Append treated as regular list
                "extraItem1",
                LIST_2("extraItem2")
            };
            linkedItems[] += {"extraLinked"}; // Append treated as regular list
            magazines[] = {                   // New property with nested format
                LIST_2("mag1"),
                LIST_2("mag2"),
                "mag3"
            };
        };"#;

        let (_, equipment) = parse_loadout(input).unwrap();
        
        // Test base class
        let base = equipment.iter().find(|e| e.class_name == "baseLoadout").unwrap();
        assert_eq!(base.properties.get("displayName").unwrap(), &vec!["Base Loadout"]);
        assert!(base.properties.get("uniform").unwrap().is_empty());
        assert_eq!(base.properties.get("traits").unwrap(), &vec!["trait1", "trait2"]);
        
        // Test first level inheritance
        let level1 = equipment.iter().find(|e| e.class_name == "level1").unwrap();
        assert_eq!(level1.parent_class, Some("baseLoadout".to_string()));
        assert_eq!(level1.properties.get("uniform").unwrap(), &vec!["uniform1"]);
        assert_eq!(level1.properties.get("vest").unwrap(), &vec!["vest1"]);
        
        // Test second level inheritance with LIST macros
        let level2 = equipment.iter().find(|e| e.class_name == "level2").unwrap();
        assert_eq!(level2.parent_class, Some("level1".to_string()));
        assert_eq!(
            level2.properties.get("uniform").unwrap(),
            &vec!["uniform1", "uniform2", "uniform3"]
        );
        assert_eq!(
            level2.properties.get("vest").unwrap(),
            &vec!["vest1", "vest1", "vest2", "vest3", "vest3", "vest3"]
        );
        assert_eq!(
            level2.properties.get("items").unwrap(),
            &vec![
                "item1", "item1", "item1", "item1", "item1",
                "item1", "item1", "item1", "item1", "item1",
                "item2", "item2", "item2", "item2", "item2"
            ]
        );
        
        // Test third level with property appending (treated as regular lists)
        let level3 = equipment.iter().find(|e| e.class_name == "level3").unwrap();
        assert_eq!(level3.parent_class, Some("level2".to_string()));
        assert_eq!(level3.properties.get("backpack").unwrap(), &vec!["newbackpack"]);
        
        // Check items[] += is treated as a regular list
        assert_eq!(
            level3.properties.get("items").unwrap(),
            &vec!["extraItem1", "extraItem2", "extraItem2"]
        );
        
        // Check linkedItems[] += is treated as a regular list
        assert_eq!(
            level3.properties.get("linkedItems").unwrap(),
            &vec!["extraLinked"]
        );
        
        // Check regular property
        assert_eq!(
            level3.properties.get("magazines").unwrap(),
            &vec!["mag1", "mag1", "mag2", "mag2", "mag3"]
        );
    }
}
