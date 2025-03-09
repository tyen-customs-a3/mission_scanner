use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, none_of, space0, space1},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult
};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EquipmentReference {
    pub class_name: String,
    pub context: String,
}

fn parse_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((
        map(take_while1(|c: char| c.is_whitespace()), |_| ()),
        map(preceded(tag("//"), take_while1(|c| c != '\n' && c != '\r')), |_| ()),
        map(delimited(tag("/*"), take_until("*/"), tag("*/")), |_| ()),
    )))(input)?;
    Ok((input, ()))
}

fn parse_quoted_string(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(
        char('"'),
        recognize(many0(alt((
            preceded(char('\\'), none_of("")),
            none_of("\"")
        )))),
        char('"')
    )(input)?;
    
    Ok((input, content.replace("\\\"", "\"")))
}

fn parse_double_quoted_string(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(
        tag("\"\""),
        recognize(many0(none_of("\""))),
        tag("\"\"")
    )(input)?;
    
    Ok((input, content.to_string()))
}

fn parse_variable_assignment(input: &str) -> IResult<&str, (String, String)> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = opt(tuple((
        tag("private"),
        space1
    )))(input)?;
    let (input, var_name) = recognize(tuple((
        char('_'),
        take_while1(|c: char| c.is_alphanumeric() || c == '_')
    )))(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, class_name) = alt((
        parse_quoted_string,
        delimited(
            tag("\"\""),
            take_while1(|c| c != '"'),
            tag("\"\"")
        )
    ))(input)?;
    
    Ok((input, (var_name.to_string(), class_name)))
}

fn parse_variable_reference(input: &str) -> IResult<&str, String> {
    let (input, var_name) = recognize(tuple((
        char('_'),
        take_while1(|c: char| c.is_alphanumeric() || c == '_')
    )))(input)?;
    Ok((input, var_name.to_string()))
}

fn parse_equipment_command(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, unit_var) = opt(tuple((
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        space1
    )))(input)?;
    let (input, command) = alt((
        tag("addHeadgear"),
        tag("addVest"),
        tag("addBackpack"),
        tag("addWeapon"),
        tag("addGoggles"),
        tag("addMagazine"),
        tag("addItemToBackpack"),
        tag("addItemToVest"),
        tag("addItemToUniform"),
        tag("addWeaponItem"),
        tag("forceAddUniform"),
        tag("createVehicle"),
        tag("call")
    ))(input)?;
    let (input, _) = space0(input)?;
    
    let (input, class_name) = if command == "addWeaponItem" {
        let (input, _) = char('[')(input)?;
        let (input, _) = parse_whitespace_and_comments(input)?;
        let (input, _) = take_until(",")(input)?;
        let (input, _) = char(',')(input)?;
        let (input, _) = space0(input)?;
        let (input, class_name) = alt((
            parse_quoted_string,
            parse_variable_reference
        ))(input)?;
        (input, class_name)
    } else if command == "createVehicle" {
        let (input, _) = char('[')(input)?;
        let (input, _) = parse_whitespace_and_comments(input)?;
        let (input, class_name) = parse_quoted_string(input)?;
        (input, class_name)
    } else if command == "call" {
        let (input, _) = opt(space0)(input)?;
        let (input, first_arg) = delimited(
            char('['),
            alt((parse_quoted_string, parse_variable_reference)),
            tuple((opt(char(',')), space0, opt(take_until("]")), char(']')))
        )(input)?;
        (input, first_arg)
    } else {
        let (input, class_name) = alt((
            parse_quoted_string,
            parse_variable_reference
        ))(input)?;
        (input, class_name)
    };

    Ok((input, EquipmentReference {
        class_name,
        context: format!("Command: {}", command),
    }))
}

fn parse_weapon_item(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = opt(tuple((
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        space1
    )))(input)?;
    let (input, _) = tag("addWeaponItem")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = take_until(",")(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = space0(input)?;
    let (input, class_name) = parse_quoted_string(input)?;
    
    Ok((input, EquipmentReference {
        class_name,
        context: "Command: addWeaponItem".to_string(),
    }))
}

fn parse_vehicle_creation(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = opt(tuple((
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        space0,
        char('='),
        space0
    )))(input)?;
    let (input, _) = tag("createVehicle")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, class_name) = parse_quoted_string(input)?;
    
    Ok((input, EquipmentReference {
        class_name,
        context: "Vehicle creation".to_string(),
    }))
}

fn parse_unit_creation(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, _) = opt(tuple((
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        space1
    )))(input)?;
    let (input, _command) = alt((
        tag("createUnit"),
        tag("createVehicle")
    ))(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, class_name) = parse_quoted_string(input)?;
    
    Ok((input, EquipmentReference {
        class_name,
        context: "Unit/Vehicle creation".to_string(),
    }))
}

fn is_diary_variable(name: &str) -> bool {
    name == "_briefing"
}

fn parse_diary_record(input: &str) -> IResult<&str, Vec<EquipmentReference>> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    
    // Handle diary variable assignment with proper type annotations
    if let Ok((remaining, var_name)) = recognize::<_, _, nom::error::Error<&str>, _>(tuple((
        char::<&str, nom::error::Error<&str>>('_'),
        take_while1(|c: char| c.is_alphanumeric() || c == '_')
    )))(input) {
        if is_diary_variable(var_name) {
            // Skip until semicolon for assignments
            if let Ok((after_semi, _)) = take_until::<&str, &str, nom::error::Error<&str>>(";")(input) {
                let (after_semi, _) = tag(";")(after_semi)?;
                return Ok((after_semi, Vec::new()));
            }
        }
    }

    // Handle diary record creation
    if input.contains("createDiaryRecord") {
        let (input, _) = opt(tuple((
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            space1
        )))(input)?;
        let (input, _) = tag("createDiaryRecord")(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = take_until("];")(input)?;
        let (input, _) = tag("];")(input)?;
        return Ok((input, Vec::new()));
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag
    )))
}

fn parse_array_item(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = char('[')(input)?;
    let (input, _) = parse_whitespace_and_comments(input)?;
    let (input, class_name) = parse_quoted_string(input)?;
    
    Ok((input, EquipmentReference {
        class_name,
        context: "Array item".to_string(),
    }))
}

fn parse_equipment_reference(input: &str) -> IResult<&str, EquipmentReference> {
    // First check if we're in a diary record context by checking for diary patterns
    let diary_patterns = [
        "private _briefing =",
        "_briefing = _briefing +",
        "createDiaryRecord"
    ];
    
    if diary_patterns.iter().any(|pattern| input.trim().starts_with(pattern)) ||
       input.trim().starts_with("_briefing") {
        // Skip parsing equipment references in diary content
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        )));
    }

    alt((
        parse_variable_assignment,
        parse_weapon_item,
        parse_vehicle_creation,
        parse_equipment_command,
        parse_unit_creation,
        parse_array_item,
    ))(input)
}

fn is_equipment_class(class_name: &str) -> bool {
    class_name.starts_with("rhs_") || 
    class_name.starts_with("I_") || 
    class_name.starts_with("Land_")
}

/// Scans an SQF file for equipment class references
pub fn scan_equipment_references(input: &str) -> HashSet<EquipmentReference> {
    let mut references = HashSet::new();
    let mut variables = std::collections::HashMap::new();
    let mut current_pos = input;

    // First pass: collect all variable assignments
    let mut temp_pos = input;
    while !temp_pos.is_empty() {
        match parse_variable_assignment(temp_pos) {
            Ok((remaining, (var_name, class_name))) => {
                // Only store equipment-like variables
                if is_equipment_class(&class_name) && !is_diary_variable(&var_name) {
                    variables.insert(var_name, class_name);
                }
                temp_pos = remaining;
            }
            Err(_) => {
                if let Some(new_pos) = temp_pos.get(1..) {
                    temp_pos = new_pos;
                } else {
                    break;
                }
            }
        }
    }

    // Second pass: process equipment references and resolve variables
    while !current_pos.is_empty() {
        // Skip diary records
        if let Ok((remaining, _)) = parse_diary_record(current_pos) {
            current_pos = remaining;
            continue;
        }

        // Try equipment commands and resolve any variables
        match parse_equipment_reference(current_pos) {
            Ok((remaining, mut reference)) => {
                // If the class_name is a variable reference, resolve it
                if reference.class_name.starts_with('_') {
                    if let Some(actual_class) = variables.get(&reference.class_name) {
                        reference.class_name = actual_class.clone();
                        references.insert(reference);
                    }
                } else if is_equipment_class(&reference.class_name) {
                    references.insert(reference);
                }
                current_pos = remaining;
            }
            Err(_) => {
                // Try parsing double-quoted strings that look like equipment references
                if let Ok((remaining, class_name)) = parse_double_quoted_string(current_pos) {
                    if class_name.starts_with("rhs_") || 
                       class_name.starts_with("I_") || 
                       class_name.starts_with("Land_") {
                        references.insert(EquipmentReference {
                            class_name,
                            context: "String literal".to_string(),
                        });
                    }
                    current_pos = remaining;
                } else if let Some(new_pos) = current_pos.get(1..) {
                    current_pos = new_pos;
                } else {
                    break;
                }
            }
        }
    }

    references
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_assignment() {
        let input = r#"private _bp = "rhs_rpg_empty";"#;
        let (_, (var_name, class_name)) = parse_variable_assignment(input).unwrap();
        assert_eq!(var_name, "_bp");
        assert_eq!(class_name, "rhs_rpg_empty");
    }

    #[test]
    fn test_equipment_command() {
        let input = r#"_unit addHeadgear "rhs_tsh4";"#;
        let (_, reference) = parse_equipment_command(input).unwrap();
        assert_eq!(reference.class_name, "rhs_tsh4");
    }

    #[test]
    fn test_weapon_item_command() {
        let input = r#"_unit addWeaponItem [_mat, "rhs_rpg7_PG7VL_mag", true];"#;
        let (_, reference) = parse_weapon_item(input).unwrap();
        assert_eq!(reference.class_name, "rhs_rpg7_PG7VL_mag");
    }

    #[test]
    fn test_unit_creation() {
        let input = r#"_group createUnit ["I_G_Soldier_F", getPos player, [], 0, "NONE"];"#;
        let (_, reference) = parse_unit_creation(input).unwrap();
        assert_eq!(reference.class_name, "I_G_Soldier_F");
    }

    #[test]
    fn test_scan_equipment_references() {
        let input = r#"
        private _bp = "rhs_rpg_empty";
        private _mat = "rhs_weap_rpg7";
        private _matAT = "rhs_rpg7_PG7VL_mag";
        
        _unit addBackpack _bp;
        _unit addWeapon _mat;
        _unit addWeaponItem [_mat, _matAT, true];
        for "_i" from 1 to (ceil (random [2,3,5])) do {
            _unit addItemToBackpack _matAT;
        };
        
        _unit addHeadgear "rhs_tsh4";
        _group createUnit ["I_G_Soldier_F", getPos player, [], 0, "NONE"];
        "#;

        let references = scan_equipment_references(input);
        
        assert!(references.iter().any(|r| r.class_name == "rhs_rpg_empty"));
        assert!(references.iter().any(|r| r.class_name == "rhs_weap_rpg7"));
        assert!(references.iter().any(|r| r.class_name == "rhs_rpg7_PG7VL_mag"));
        assert!(references.iter().any(|r| r.class_name == "rhs_tsh4"));
        assert!(references.iter().any(|r| r.class_name == "I_G_Soldier_F"));
    }

    #[test]
    fn test_real_file_parsing() {
        let input = r#"
        if (!isServer) exitWith {};
        private _trigger = selectRandom (missionNamespace getVariable "RATSNAKE_MotorPatrolSpawns");
        private _hmmvw = createVehicle ["I_C_Offroad_02_LMG_F", getPos _trigger];
        _hmmvw setDir (triggerArea _trigger) # 2;
        private _thisPatrol = createGroup independent;
        for "_i" from 1 to 3 do {
            _thisPatrol createUnit ["I_G_Soldier_F", getPos _trigger, [], 10, "NONE"]
        };
        {_x moveInAny _hmmvw} forEach (units _thisPatrol);
        _thisPatrol copyWaypoints (_trigger getVariable "RATSNAKE_waypoints");
        _thisPatrol deleteGroupWhenEmpty true;
        clearWeaponCargoGlobal _hmmvw;
        clearMagazineCargoGlobal _hmmvw;
        clearItemCargoGlobal _hmmvw;
        clearBackpackCargoGlobal _hmmvw;
        ["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;
        "#;

        let references = scan_equipment_references(input);
        assert!(references.iter().any(|r| r.class_name == "I_C_Offroad_02_LMG_F"));
        assert!(references.iter().any(|r| r.class_name == "I_G_Soldier_F"));
        assert!(references.iter().any(|r| r.class_name == "Land_CanisterFuel_Red_F"));
    }

    #[test]
    fn test_diary_record_parsing() {
        let input = r#"
        private _briefing = "ADMIN BRIEFING<br/><br/>";
        _briefing = _briefing + "
            additional enemy reinforcents can be sent after initial mechanized spawn at admin discretion.
            Mission ends after squad retreats to the south.
            Uses weapons like ""rhs_weap_m4a1"" and ""rhs_weap_akm""
        ";
        player createDiaryRecord ["diary", ["Admin",_briefing]];
        private _weapon = "rhs_weap_m4a1"; // This should NOT be detected
        "#;
        
        let references = scan_equipment_references(input);
        // Should not detect any references since none are used in equipment commands
        assert_eq!(references.len(), 0);
    }

    #[test]
    fn test_multi_assignment_and_concat() {
        let input = r#"
        private _loadout = "rhs_weap_m4a1";
        private _briefing = "Loadout: " + _loadout + "<br/>";
        _briefing = _briefing + "
            Additional equipment: ""rhs_mag_30Rnd_556x45_M855A1_Stanag""
        ";
        "#;
        
        let references = scan_equipment_references(input);
        // Should not detect any references since none are used in equipment commands
        assert_eq!(references.len(), 0);
    }

    #[test] 
    fn test_variable_reference_in_command() {
        let input = r#"
        private _weapon = "rhs_weap_m4a1";
        _unit addWeapon _weapon; // This SHOULD be detected via variable resolution
        "#;
        
        let references = scan_equipment_references(input);
        assert_eq!(references.len(), 1);
        assert!(references.iter().any(|r| r.class_name == "rhs_weap_m4a1"));
    }

    #[test]
    fn test_variable_extraction() {
        let input = r#"
        private _weapon = "rhs_weap_m4a1";
        _unit addWeapon _weapon;
        _otherVar = "some_other_value";
        private _bp = "rhs_rpg_empty";
        "#;
        
        let mut temp_pos = input;
        let mut variables = std::collections::HashMap::new();

        while !temp_pos.is_empty() {
            if let Ok((remaining, (var_name, class_name))) = parse_variable_assignment(temp_pos) {
                variables.insert(var_name, class_name);
                temp_pos = remaining;
            } else if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
        }

        assert_eq!(variables.len(), 3); // Should find _weapon, _otherVar, and _bp
        assert_eq!(variables.get("_weapon"), Some(&"rhs_weap_m4a1".to_string()));
        assert_eq!(variables.get("_bp"), Some(&"rhs_rpg_empty".to_string()));
    }

    #[test]
    fn test_variable_extraction_with_diary() {
        let input = r#"
        private _weapon = "rhs_weap_m4a1";
        private _briefing = "ADMIN BRIEFING<br/><br/>";
        _briefing = _briefing + "Some text";
        private _bp = "rhs_rpg_empty";
        "#;
        
        let mut temp_pos = input;
        let mut variables = std::collections::HashMap::new();

        while !temp_pos.is_empty() {
            if let Ok((remaining, (var_name, class_name))) = parse_variable_assignment(temp_pos) {
                variables.insert(var_name, class_name);
                temp_pos = remaining;
            } else if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
        }

        assert_eq!(variables.len(), 2); // Should find _weapon and _bp, but not _briefing
        assert_eq!(variables.get("_weapon"), Some(&"rhs_weap_m4a1".to_string()));
        assert_eq!(variables.get("_bp"), Some(&"rhs_rpg_empty".to_string()));
        assert!(variables.get("_briefing").is_none()); // Should not capture diary variables
    }

    #[test]
    fn test_variable_overwrite() {
        let input = r#"
        private _weapon = "rhs_weap_m4a1";
        _weapon = "rhs_weap_akm"; // Should overwrite previous value
        "#;
        
        let mut temp_pos = input;
        let mut variables = std::collections::HashMap::new();

        while !temp_pos.is_empty() {
            if let Ok((remaining, (var_name, class_name))) = parse_variable_assignment(temp_pos) {
                variables.insert(var_name, class_name);
                temp_pos = remaining;
            } else if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
        }

        assert_eq!(variables.len(), 1);
        assert_eq!(variables.get("_weapon"), Some(&"rhs_weap_akm".to_string()));
    }
}
