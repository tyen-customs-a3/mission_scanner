use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, space0, space1},
    sequence::{delimited, preceded},
    branch::alt,
    combinator::{map, opt},
    error::Error,
    Parser,
};

use crate::sqf::models::CargoOperation;

/// Parse a string literal (text enclosed in double quotes)
pub fn string_literal(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until("\""),
        char('"')
    ).parse(input)
}

/// Parse a variable name (starts with underscore)
pub fn variable_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = char('_').parse(input)?;
    let (input, name) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;
    Ok((input, name))
}

/// Parse a clear cargo operation
/// Example: clearWeaponCargoGlobal _hmmvw;
pub fn clear_cargo(input: &str) -> IResult<&str, CargoOperation> {
    let (input, _) = tag("clear").parse(input)?;
    let (input, cargo_type) = alt((
        map(tag("WeaponCargoGlobal"), |_| "weapon"),
        map(tag("MagazineCargoGlobal"), |_| "magazine"),
        map(tag("ItemCargoGlobal"), |_| "item"),
        map(tag("BackpackCargoGlobal"), |_| "backpack"),
    )).parse(input)?;
    
    let (input, _) = space1.parse(input)?;
    let (input, vehicle) = variable_name(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, CargoOperation::Clear {
        vehicle: vehicle.to_string(),
        cargo_type: cargo_type.to_string(),
    }))
}

/// Parse an ACE cargo load operation
/// Example: ["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;
pub fn ace_cargo_load(input: &str) -> IResult<&str, CargoOperation> {
    let (input, _) = char('[').parse(input)?;
    let (input, item) = preceded(space0::<&str, Error<&str>>, string_literal).parse(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char(',')).parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, vehicle) = variable_name(input)?;
    let (input, _) = preceded(space0::<&str, Error<&str>>, char(']')).parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("call").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, function) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    
    Ok((input, CargoOperation::Load {
        item: item.to_string(),
        vehicle: vehicle.to_string(),
        function: function.to_string(),
    }))
}

/// Parse a line that might contain a cargo operation
pub fn parse_cargo_line(input: &str) -> Option<CargoOperation> {
    let trimmed = input.trim();
    
    // Skip empty lines or comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }
    
    // Try to parse clear cargo operations
    if trimmed.starts_with("clear") {
        if let Ok((_, operation)) = clear_cargo(trimmed) {
            return Some(operation);
        }
    }
    
    // Try to parse ACE cargo load operations
    if trimmed.starts_with("[") && trimmed.contains("ace_cargo_fnc_loadItem") {
        if let Ok((_, operation)) = ace_cargo_load(trimmed) {
            return Some(operation);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_literal() {
        // Basic cases
        assert_eq!(string_literal(r#""ACE_fieldDressing""#).unwrap(), ("", "ACE_fieldDressing"));
        assert_eq!(string_literal(r#""Land_CanisterFuel_Red_F""#).unwrap(), ("", "Land_CanisterFuel_Red_F"));

        // Special characters
        assert_eq!(string_literal(r#""Item with \"quotes\" inside""#).unwrap(), ("", r#"Item with \"quotes\" inside"#));
        assert_eq!(string_literal(r#""Item with \n newline""#).unwrap(), ("", r#"Item with \n newline"#));
        assert_eq!(string_literal(r#""Path\To\Item""#).unwrap(), ("", r#"Path\To\Item"#));
        assert_eq!(string_literal(r#""Item with spaces and	tabs""#).unwrap(), ("", "Item with spaces and	tabs"));
        
        // Unicode and special characters
        assert_eq!(string_literal(r#""Предмет""#).unwrap(), ("", "Предмет")); // Cyrillic
        assert_eq!(string_literal(r#""アイテム""#).unwrap(), ("", "アイテム")); // Japanese
        assert_eq!(string_literal(r#""Item_with_$pecial_©hars""#).unwrap(), ("", "Item_with_$pecial_©hars"));

        // Error cases
        assert!(string_literal(r#""Unclosed string"#).is_err());
        assert!(string_literal(r#"Not a string"#).is_err());
        assert!(string_literal(r#""""#).unwrap() == ("", "")); // Empty string
        assert!(string_literal(r#""\"#).is_err()); // Single backslash
        assert!(string_literal("").is_err()); // Empty input
    }

    #[test]
    fn test_variable_name() {
        // Basic cases
        assert_eq!(variable_name("_hmmvw").unwrap(), ("", "hmmvw"));
        assert_eq!(variable_name("_vehicle123").unwrap(), ("", "vehicle123"));

        // With trailing content
        assert_eq!(variable_name("_car;").unwrap(), (";", "car"));
        assert_eq!(variable_name("_vehicle rest").unwrap(), (" rest", "vehicle"));
        assert_eq!(variable_name("_var1,_var2").unwrap(), (",_var2", "var1"));

        // Complex variable names
        assert_eq!(variable_name("_camelCaseVar").unwrap(), ("", "camelCaseVar"));
        assert_eq!(variable_name("_UPPERCASE").unwrap(), ("", "UPPERCASE"));
        assert_eq!(variable_name("_mixed123Case456").unwrap(), ("", "mixed123Case456"));
        assert_eq!(variable_name("_x").unwrap(), ("", "x")); // Single letter
        assert_eq!(variable_name("_1").unwrap(), ("", "1")); // Single number

        // Error cases
        let error_cases = vec![
            "hmmvw",         // Missing underscore
            "_",             // Just underscore
            "",             // Empty string
            "var_name",     // No leading underscore
            "1_var",        // Number first
            "_var-name",    // Invalid character
            "_var name",    // Space in name
            "_var.name",    // Dot in name
            "_ name",       // Space after underscore
        ];

        for input in error_cases {
            assert!(variable_name(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_clear_cargo() {
        // Test all cargo types with variations
        let test_cases = vec![
            // Basic cases
            ("clearWeaponCargoGlobal _hmmvw;", ("hmmvw", "weapon")),
            ("clearMagazineCargoGlobal _hmmvw;", ("hmmvw", "magazine")),
            ("clearItemCargoGlobal _hmmvw;", ("hmmvw", "item")),
            ("clearBackpackCargoGlobal _hmmvw;", ("hmmvw", "backpack")),
            
            // With different variable names
            ("clearWeaponCargoGlobal _longVariableName123;", ("longVariableName123", "weapon")),
            ("clearWeaponCargoGlobal _x;", ("x", "weapon")),
            
            // With whitespace variations
            ("clearWeaponCargoGlobal    _hmmvw   ;", ("hmmvw", "weapon")),
            ("clearWeaponCargoGlobal	_hmmvw;", ("hmmvw", "weapon")), // Tab
            ("clearWeaponCargoGlobal _hmmvw     ;", ("hmmvw", "weapon")),
            
            // Without semicolon
            ("clearWeaponCargoGlobal _hmmvw", ("hmmvw", "weapon")),
        ];

        for (input, (expected_vehicle, expected_type)) in test_cases {
            let (_, operation) = clear_cargo(input).unwrap();
            match operation {
                CargoOperation::Clear { vehicle, cargo_type } => {
                    assert_eq!(vehicle, expected_vehicle);
                    assert_eq!(cargo_type, expected_type);
                },
                _ => panic!("Expected Clear operation for input: {}", input),
            }
        }

        // Error cases
        let error_cases = vec![
            "clearCargoGlobal _hmmvw",           // Invalid cargo type
            "clearWeaponCargoGlobal",            // Missing variable
            "clear _hmmvw",                      // Invalid command
            "clearWeaponCargoGlobal hmmvw",      // Missing underscore
            "clearWeaponCargoGlobal _",          // Missing variable name
            "clearWeaponCargoGlobal _ hmmvw",    // Space in variable
            "clearWeaponCargoGlobal_hmmvw",      // Missing space
            "clearWeaponCargoGlobal _hmmvw,",    // Invalid trailing character
        ];

        for input in error_cases {
            assert!(clear_cargo(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_ace_cargo_load() {
        // Test valid cases with variations
        let test_cases = vec![
            // Basic case
            (r#"["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;"#,
             ("Land_CanisterFuel_Red_F", "hmmvw", "ace_cargo_fnc_loadItem")),
            
            // Different item names
            (r#"["Box_NATO_Ammo_F", _vehicle] call ace_cargo_fnc_loadItem;"#,
             ("Box_NATO_Ammo_F", "vehicle", "ace_cargo_fnc_loadItem")),
            
            // With spaces and formatting variations
            (r#"[   "Land_CanisterFuel_Red_F"   ,    _hmmvw   ]    call    ace_cargo_fnc_loadItem   ;"#,
             ("Land_CanisterFuel_Red_F", "hmmvw", "ace_cargo_fnc_loadItem")),
            
            // Without semicolon
            (r#"["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem"#,
             ("Land_CanisterFuel_Red_F", "hmmvw", "ace_cargo_fnc_loadItem")),
            
            // With tabs and newlines
            (r#"["Land_CanisterFuel_Red_F",	_hmmvw]	call	ace_cargo_fnc_loadItem;"#,
             ("Land_CanisterFuel_Red_F", "hmmvw", "ace_cargo_fnc_loadItem")),
        ];

        for (input, (expected_item, expected_vehicle, expected_function)) in test_cases {
            let (_, operation) = ace_cargo_load(input).unwrap();
            match operation {
                CargoOperation::Load { item, vehicle, function } => {
                    assert_eq!(item, expected_item);
                    assert_eq!(vehicle, expected_vehicle);
                    assert_eq!(function, expected_function);
                },
                _ => panic!("Expected Load operation for input: {}", input),
            }
        }

        // Error cases
        let error_cases = vec![
            // Syntax errors
            r#"["Land_CanisterFuel_Red_F" _hmmvw] call ace_cargo_fnc_loadItem;"#,  // Missing comma
            r#"["Land_CanisterFuel_Red_F", _hmmvw call ace_cargo_fnc_loadItem;"#,  // Missing closing bracket
            r#""Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;"#,  // Missing opening bracket
            
            // Invalid values
            r#"[Land_CanisterFuel_Red_F, _hmmvw] call ace_cargo_fnc_loadItem;"#,   // Missing quotes
            r#"["", _hmmvw] call ace_cargo_fnc_loadItem;"#,                        // Empty item name
            r#"["Land_CanisterFuel_Red_F", hmmvw] call ace_cargo_fnc_loadItem;"#,  // Invalid variable
            
            // Invalid function calls
            r#"["Land_CanisterFuel_Red_F", _hmmvw] ace_cargo_fnc_loadItem;"#,      // Missing call
            r#"["Land_CanisterFuel_Red_F", _hmmvw] call;"#,                        // Missing function name
            r#"["Land_CanisterFuel_Red_F", _hmmvw] call invalid_function;"#,        // Invalid function
        ];

        for input in error_cases {
            assert!(ace_cargo_load(input).is_err(), "Should fail to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_cargo_line() {
        // Test valid cases
        let valid_cases = vec![
            // Clear operations
            "clearWeaponCargoGlobal _hmmvw;",
            "clearMagazineCargoGlobal _vehicle;",
            "   clearItemCargoGlobal    _container   ;   ",
            
            // Load operations
            r#"["Box_NATO_Ammo_F", _hmmvw] call ace_cargo_fnc_loadItem;"#,
            r#"   ["Land_CanisterFuel_Red_F", _vehicle] call ace_cargo_fnc_loadItem   ;"#,
        ];

        for input in valid_cases {
            assert!(parse_cargo_line(input).is_some(), "Should parse: {}", input);
        }

        // Test cases that should return None
        let none_cases = vec![
            "",                             // Empty line
            "    ",                         // Whitespace only
            "// Comment",                   // Comment
            "/* Multi-line comment */",     // Multi-line comment
            "_unit addItem 'ACE_fieldDressing';",  // Different command
            "hint 'Cargo cleared';",        // Unrelated command
            "if (true) then { };",         // Control structure
        ];

        for input in none_cases {
            assert_eq!(parse_cargo_line(input), None, "Should return None for: {}", input);
        }

        // Test error cases (should return None instead of error)
        let error_cases = vec![
            "clearCargoGlobal _hmmvw;",    // Invalid clear command
            r#"["Item"] call wrong_function;"#,  // Invalid function
            "clearWeaponCargoGlobal;",     // Missing variable
            r#"["Item", _var] ace_cargo_fnc_loadItem;"#,  // Missing call
        ];

        for input in error_cases {
            assert_eq!(parse_cargo_line(input), None, "Should return None for invalid input: {}", input);
        }
    }
} 