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
use log::{debug, info, trace, warn};

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

fn parse_variable_assignment(input: &str) -> IResult<&str, EquipmentReference> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    
    // First check if this is a diary-related assignment
    if let Ok(_) = tuple::<_, _, nom::error::Error<&str>, _>((
        opt(tuple((
            tag("private"),
            space1
        ))),
        alt((
            tag("_briefing"),
            tag("_situation"),
            tag("_mission"),
            tag("_execution"),
            tag("_administration")
        )),
        space0,
        alt((
            tag("="),
            tag("+")
        ))
    ))(input) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        )));
    }
    
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
        map(
            delimited(
                tag("\"\""),
                take_while1(|c| c != '"'),
                tag("\"\"")
            ),
            |s: &str| s.to_string()
        )
    ))(input)?;
    
    Ok((input, EquipmentReference {
        class_name: class_name.to_string(),
        context: format!("Variable assignment: {}", var_name),
    }))
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
    let (input, _unit_var) = opt(tuple((
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

fn parse_diary_section(input: &str) -> IResult<&str, ()> {
    let (input, _) = parse_whitespace_and_comments(input)?;
    
    // Match diary variable assignment or concatenation
    if let Ok((remaining, _)) = tuple::<_, _, nom::error::Error<&str>, _>((
        opt(tuple((
            tag("private"),
            space1
        ))),
        alt((
            tag("_briefing"),
            tag("_situation"),
            tag("_mission"),
            tag("_execution"),
            tag("_administration")
        )),
        space0,
        tag("=")
    ))(input) {
        // Skip until we find a semicolon that's not inside a string
        let mut depth = 0;
        let mut pos = 0;
        let chars: Vec<char> = remaining.chars().collect();
        
        while pos < chars.len() {
            match chars[pos] {
                '"' => depth = 1 - depth,
                ';' if depth == 0 => {
                    trace!("Diary section skipped - assignment/concat. Input starts with: {}", &input[..std::cmp::min(40, input.len())]);
                    return Ok((&remaining[pos + 1..], ()));
                }
                _ => {}
            }
            pos += 1;
        }
        trace!("Diary section partially skipped - no semicolon found. Input starts with: {}", &input[..std::cmp::min(40, input.len())]);
        return Ok((remaining, ()));
    }
    
    // Match diary record creation
    if let Ok((remaining, _)) = tuple::<_, _, nom::error::Error<&str>, _>((
        opt(take_while1(|c: char| c.is_alphanumeric() || c == '_')),
        space0,
        tag("createDiaryRecord")
    ))(input) {
        // Skip until we find a semicolon that's not inside a string
        let mut depth = 0;
        let mut pos = 0;
        let chars: Vec<char> = remaining.chars().collect();
        
        while pos < chars.len() {
            match chars[pos] {
                '"' => depth = 1 - depth,
                '[' if depth == 0 => depth += 1,
                ']' if depth == 0 => depth -= 1,
                ';' if depth == 0 => {
                    trace!("Diary section skipped - createDiaryRecord. Input starts with: {}", &input[..std::cmp::min(40, input.len())]);
                    return Ok((&remaining[pos + 1..], ()));
                }
                _ => {}
            }
            pos += 1;
        }
        trace!("Diary section partially skipped - no semicolon found. Input starts with: {}", &input[..std::cmp::min(40, input.len())]);
        return Ok((remaining, ()));
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
    // First try to skip any diary sections
    if let Ok((remaining, _)) = parse_diary_section(input) {
        trace!("Equipment reference skipped - diary section detected. Input starts with: {}", &input[..std::cmp::min(40, input.len())]);
        return Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::Tag
        )));
    }

    let result = alt((
        parse_variable_assignment,
        parse_weapon_item,
        parse_vehicle_creation,
        parse_equipment_command,
        parse_unit_creation,
        parse_array_item,
    ))(input);
    
    if let Ok((_, reference)) = &result {
        trace!("Equipment reference parsed: {} ({})", reference.class_name, reference.context);
    }
    
    result
}

fn is_equipment_class(class_name: &str) -> bool {
    class_name.starts_with("rhs_") || 
    class_name.starts_with("I_") || 
    class_name.starts_with("Land_")
}

fn is_inside_diary_context(input: &str) -> bool {
    // Check if we're inside a diary-related context
    if let Ok(_) = tuple::<_, _, nom::error::Error<&str>, _>((
        opt(tuple((
            tag("private"),
            space1
        ))),
        alt((
            tag("_briefing"),
            tag("_situation"),
            tag("_mission"),
            tag("_execution"),
            tag("_administration")
        )),
        space0,
        alt((
            tag("="),
            tag("+")
        ))
    ))(input) {
        return true;
    }

    // Check for diary record creation
    if let Ok(_) = tuple::<_, _, nom::error::Error<&str>, _>((
        opt(take_while1(|c: char| c.is_alphanumeric() || c == '_')),
        space0,
        tag("createDiaryRecord")
    ))(input) {
        return true;
    }
    
    false
}

fn is_used_in_command(input: &str, var_name: &str) -> bool {
    // Check if the variable is used in an equipment command
    let command_patterns = [
        "addHeadgear", "addVest", "addBackpack", "addWeapon", "addGoggles",
        "addMagazine", "addItemToBackpack", "addItemToVest", "addItemToUniform",
        "addWeaponItem", "forceAddUniform", "createVehicle", "createUnit"
    ];
    
    for pattern in command_patterns {
        if input.contains(&format!("{} {}", pattern, var_name)) {
            return true;
        }
    }
    
    false
}

/// Scans an SQF file for equipment class references
pub fn scan_equipment_references(input: &str) -> HashSet<EquipmentReference> {
    let mut references = HashSet::new();
    let mut variables = std::collections::HashMap::new();
    let mut diary_variables = HashSet::new();
    let mut in_diary_section = false;

    // First pass: collect all variable assignments and mark diary variables
    debug!("First pass - variable collection");
    let mut temp_pos = input;
    while !temp_pos.is_empty() {
        // Check if we're entering a diary section
        if is_inside_diary_context(temp_pos) {
            in_diary_section = true;
            trace!("Entering diary section at: {}", &temp_pos[..std::cmp::min(40, temp_pos.len())]);
            
            // If a line contains "+" and a variable, mark that variable as used in a diary
            if temp_pos.contains("+") {
                // Extract any variable references
                let mut pos = 0;
                while let Some(idx) = temp_pos[pos..].find('_') {
                    pos += idx;
                    if pos > 0 && temp_pos.as_bytes()[pos-1].is_ascii_alphanumeric() {
                        pos += 1;
                        continue;
                    }
                    
                    let end = temp_pos[pos..].find(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .map_or(temp_pos.len(), |i| pos + i);
                    
                    if end > pos {
                        let var_ref = &temp_pos[pos..end];
                        trace!("Variable used in diary: {}", var_ref);
                        diary_variables.insert(var_ref.to_string());
                    }
                    
                    pos = end;
                }
            }
            
            // Skip until semicolon
            if let Some(idx) = temp_pos.find(';') {
                temp_pos = &temp_pos[idx+1..];
            } else if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
            continue;
        }
        
        // Reset diary section flag if we encounter a semicolon
        if in_diary_section && temp_pos.starts_with(';') {
            in_diary_section = false;
            trace!("Exiting diary section");
            if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
            continue;
        }
        
        // Skip processing while in diary section
        if in_diary_section {
            if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
            continue;
        }

        match parse_variable_assignment(temp_pos) {
            Ok((remaining, reference)) => {
                // Store all variables, not just equipment-like ones
                let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                trace!("Variable stored: {} = {}", var_name, reference.class_name);
                variables.insert(var_name, reference.class_name);
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
    debug!("Second pass - resolving references");
    in_diary_section = false;
    let mut current_pos = input;
    
    while !current_pos.is_empty() {
        // Check if we're entering a diary section
        if is_inside_diary_context(current_pos) {
            in_diary_section = true;
            trace!("Entering diary section at: {}", &current_pos[..std::cmp::min(40, current_pos.len())]);
            
            // Skip until semicolon
            if let Some(idx) = current_pos.find(';') {
                current_pos = &current_pos[idx+1..];
            } else if let Some(new_pos) = current_pos.get(1..) {
                current_pos = new_pos;
            } else {
                break;
            }
            continue;
        }
        
        // Reset diary section flag if we encounter a semicolon
        if in_diary_section && current_pos.starts_with(';') {
            in_diary_section = false;
            trace!("Exiting diary section");
            if let Some(new_pos) = current_pos.get(1..) {
                current_pos = new_pos;
            } else {
                break;
            }
            continue;
        }
        
        // Skip processing while in diary section
        if in_diary_section {
            if let Some(new_pos) = current_pos.get(1..) {
                current_pos = new_pos;
            } else {
                break;
            }
            continue;
        }

        // Try equipment commands and resolve any variables
        match parse_equipment_reference(current_pos) {
            Ok((remaining, mut reference)) => {
                // If the class_name is a variable reference, resolve it
                if reference.class_name.starts_with('_') {
                    trace!("Variable reference: {}", reference.class_name);
                    
                    // Skip variables only used in diary sections
                    if diary_variables.contains(&reference.class_name) && !is_used_in_command(input, &reference.class_name) {
                        trace!("Skipping diary-only variable: {}", reference.class_name);
                        current_pos = remaining;
                        continue;
                    }
                    
                    if let Some(actual_class_name) = variables.get(&reference.class_name) {
                        trace!("Variable resolved: {} -> {}", reference.class_name, actual_class_name);
                        reference.class_name = actual_class_name.clone();
                        if is_equipment_class(&reference.class_name) {
                            trace!("Adding resolved reference: {} ({})", reference.class_name, reference.context);
                            references.insert(reference);
                        } else {
                            trace!("Resolved reference not an equipment class: {}", reference.class_name);
                        }
                    } else {
                        warn!("Variable not found: {}", reference.class_name);
                    }
                } else if is_equipment_class(&reference.class_name) {
                    // Skip variables only defined for diary use
                    if reference.context.contains("Variable assignment") {
                        let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                        if diary_variables.contains(&var_name) && !is_used_in_command(input, &var_name) {
                            trace!("Skipping diary-related variable assignment: {}", var_name);
                            current_pos = remaining;
                            continue;
                        }
                    }
                    
                    trace!("Adding direct reference: {} ({})", reference.class_name, reference.context);
                    references.insert(reference);
                } else {
                    trace!("Reference not an equipment class: {}", reference.class_name);
                }
                current_pos = remaining;
            }
            Err(_) => {
                // Try parsing double-quoted strings that look like equipment references
                if let Ok((remaining, class_name)) = parse_double_quoted_string(current_pos) {
                    if is_equipment_class(&class_name) {
                        trace!("Adding string literal: {}", class_name);
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

    info!("Final references count: {}", references.len());
    references
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;

    // Add a setup function for enabling logging in tests
    fn setup() {
        // Initialize logging for tests with proper output
        let _ = env_logger::try_init();
    }

    #[test]
    fn test_variable_assignment() {
        setup();
        let input = r#"private _bp = "rhs_rpg_empty";"#;
        let (_, reference) = parse_variable_assignment(input).unwrap();
        assert_eq!(reference.class_name, "rhs_rpg_empty");
        assert_eq!(reference.context, "Variable assignment: _bp");
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
        setup();
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
        
        // Debug info: Show what sections are being detected as diary sections
        debug!("--- test_diary_record_parsing ---");
        debug!("Input: {}", input);
        let mut current_pos = input;
        while !current_pos.is_empty() {
            if let Ok((remaining, _)) = parse_diary_section(current_pos) {
                let skipped_len = current_pos.len() - remaining.len();
                debug!("Diary section detected, skipped {} chars: {}", 
                        skipped_len, 
                        &current_pos[..std::cmp::min(skipped_len, 40)]);
                current_pos = remaining;
            } else if let Some(new_pos) = current_pos.get(1..) {
                current_pos = new_pos;
            } else {
                break;
            }
        }
        
        // Show what the scan_equipment_references function is finding
        let references = scan_equipment_references(input);
        debug!("Found {} references:", references.len());
        for reference in &references {
            debug!("  - {} ({})", reference.class_name, reference.context);
        }
        
        // Should not detect any references since none are used in equipment commands
        assert_eq!(references.len(), 0);
    }

    #[test]
    fn test_multi_assignment_and_concat() {
        setup();
        let input = r#"
        private _loadout = "rhs_weap_m4a1";
        private _briefing = "Loadout: " + _loadout + "<br/>";
        _briefing = _briefing + "
            Additional equipment: ""rhs_mag_30Rnd_556x45_M855A1_Stanag""
        ";
        "#;
        
        // Debug info
        debug!("--- test_multi_assignment_and_concat ---");
        debug!("Input: {}", input);
        let mut current_pos = input;
        while !current_pos.is_empty() {
            if let Ok((remaining, _)) = parse_diary_section(current_pos) {
                let skipped_len = current_pos.len() - remaining.len();
                debug!("Diary section detected, skipped {} chars: {}", 
                        skipped_len, 
                        &current_pos[..std::cmp::min(skipped_len, 40)]);
                current_pos = remaining;
            } else if let Ok((remaining, reference)) = parse_variable_assignment(current_pos) {
                debug!("Variable assignment: {} = {}", 
                      reference.context.split(": ").nth(1).unwrap_or(""),
                      reference.class_name);
                current_pos = remaining;
            } else if let Some(new_pos) = current_pos.get(1..) {
                current_pos = new_pos;
            } else {
                break;
            }
        }
        
        // Show what the scan_equipment_references function is finding
        let references = scan_equipment_references(input);
        debug!("Found {} references:", references.len());
        for reference in &references {
            debug!("  - {} ({})", reference.class_name, reference.context);
        }
        
        // Should not detect any references since none are used in equipment commands
        assert_eq!(references.len(), 0);
    }

    #[test] 
    fn test_variable_reference_in_command() {
        setup();
        let input = r#"
        private _weapon = "rhs_weap_m4a1";
        _unit addWeapon _weapon; // This SHOULD be detected via variable resolution
        "#;
        
        // Debug info
        debug!("--- test_variable_reference_in_command ---");
        debug!("Input: {}", input);
        
        // First track variables
        let mut temp_pos = input;
        let mut variables = std::collections::HashMap::new();
        debug!("Variables found:");
        while !temp_pos.is_empty() {
            if let Ok((remaining, reference)) = parse_variable_assignment(temp_pos) {
                let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                debug!("  - {} = {}", var_name, reference.class_name);
                variables.insert(var_name, reference.class_name);
                temp_pos = remaining;
            } else if let Some(new_pos) = temp_pos.get(1..) {
                temp_pos = new_pos;
            } else {
                break;
            }
        }
        
        // Then track equipment references
        let mut current_pos = input;
        debug!("Equipment references found:");
        while !current_pos.is_empty() {
            match parse_equipment_reference(current_pos) {
                Ok((remaining, reference)) => {
                    debug!("  - Direct: {} ({})", reference.class_name, reference.context);
                    current_pos = remaining;
                }
                Err(_) => {
                    if let Some(new_pos) = current_pos.get(1..) {
                        current_pos = new_pos;
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Show what the scan_equipment_references function is finding
        let references = scan_equipment_references(input);
        debug!("Final references from scan_equipment_references:");
        for reference in &references {
            debug!("  - {} ({})", reference.class_name, reference.context);
        }
        
        // Updated expectation: We find both the variable assignment and the variable reference
        assert_eq!(references.len(), 2);
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
            if let Ok((remaining, reference)) = parse_variable_assignment(temp_pos) {
                let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                variables.insert(var_name, reference.class_name);
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
            if let Ok((remaining, reference)) = parse_variable_assignment(temp_pos) {
                let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                variables.insert(var_name, reference.class_name);
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
            if let Ok((remaining, reference)) = parse_variable_assignment(temp_pos) {
                let var_name = reference.context.split(": ").nth(1).unwrap_or("").to_string();
                variables.insert(var_name, reference.class_name);
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
