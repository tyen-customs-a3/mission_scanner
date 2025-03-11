use parser_sqf::{parser, scanner};
use chumsky::Parser;

#[test]
fn test_equipment_assignment() {
    let input = r#"
        _unit addHeadgear "rhs_tsh4";
    "#;

    let ast = parser::parser().parse(input).unwrap();
    let items = scanner::scan_single(&ast.expressions[0]);
    
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].item_id, "rhs_tsh4");
}

#[test]
fn test_weapon_and_ammo_assignment() {
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
    "#;

    let ast = parser::parser().parse(input).unwrap();
    let mut all_items = Vec::new();
    for expr in &ast.expressions {
        all_items.extend(scanner::scan_single(expr));
    }
    
    // Check that we found all three items
    assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg_empty"));
    assert!(all_items.iter().any(|item| item.item_id == "rhs_weap_rpg7"));
    assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
}

#[test]
fn test_weighted_selection() {
    let input = r#"
        private _facewearPoolWeighted = selectRandomWeighted [
            "goggles1", 4,
            "goggles2", 1
        ];
        _unit addGoggles _facewearPoolWeighted;
    "#;

    let ast = parser::parser().parse(input).unwrap();
    let mut all_items = Vec::new();
    for expr in &ast.expressions {
        all_items.extend(scanner::scan_single(expr));
    }
    
    // Check that we found both goggles
    assert!(all_items.iter().any(|item| item.item_id == "goggles1"));
    assert!(all_items.iter().any(|item| item.item_id == "goggles2"));
}

#[test]
fn test_array_access_and_foreach() {
    let input = r#"
        private _array = ["item1", "item2", "item3"];
        private _index = _array # 2;
        {
            _unit addItem _x;
        } forEach _array;
    "#;

    let ast = parser::parser().parse(input).unwrap();
    let mut all_items = Vec::new();
    for expr in &ast.expressions {
        all_items.extend(scanner::scan_single(expr));
    }
    
    assert!(all_items.iter().any(|item| item.item_id == "item1"));
    assert!(all_items.iter().any(|item| item.item_id == "item2"));
    assert!(all_items.iter().any(|item| item.item_id == "item3"));
}

#[test]
fn test_multiline_comments_and_strings() {
    let input = r#"
        /* This is a multiline comment
           with multiple lines */
        private _str = "This is a string";
        // This is a single line comment
        private _multiline = "Line 1
            Line 2
            Line 3";
        _unit addItem _str;
    "#;

    let ast = parser::parser().parse(input).unwrap();
    assert!(ast.expressions.len() > 0);
}

#[test]
fn test_complex_function_calls() {
    let input = r#"
        private _result = [
            ["Land_CanisterFuel_Red_F", _vehicle] call ace_cargo_fnc_loadItem,
            (missionNamespace getVariable ["RATSNAKE_MotorPatrolSpawns", []]),
            ["I_C_Offroad_02_LMG_F", getPos _trigger, [], 10, "NONE"] call BIS_fnc_spawnVehicle
        ];
    "#;

    let ast = parser::parser().parse(input).unwrap();
    let mut all_items = Vec::new();
    for expr in &ast.expressions {
        all_items.extend(scanner::scan_single(expr));
    }
    
    assert!(all_items.iter().any(|item| item.item_id == "Land_CanisterFuel_Red_F"));
    assert!(all_items.iter().any(|item| item.item_id == "I_C_Offroad_02_LMG_F"));
}

#[test]
fn test_global_variables() {
    let input = r#"
        force ace_medical_blood_enabledFor = 1;
        force force ace_arsenal_enableIdentityTabs = false;
        missionNamespace setVariable ["RATSNAKE_waypoints", _waypoints];
        profileNamespace setVariable ["ace_arsenal_saved_loadouts", []];
    "#;

    let ast = parser::parser().parse(input).unwrap();
    assert!(ast.expressions.len() > 0);
}