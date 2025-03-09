#[cfg(test)]
mod sqf_parser_tests {
    use parsers::sqf::parsers;

    const ASSIGN_GEAR_SQF: &str = r#"
    for "_i" from 1 to (ceil (random [0,1,4])) do {_unit addItemToUniform "ACE_fieldDressing";};
    for "_i" from 1 to (ceil (random [0,1,4])) do {_unit addItemToUniform "ACE_packingBandage";};
    for "_i" from 1 to (ceil (random [0,1,1])) do {_unit addItemToUniform "ACE_epinephrine";};
    for "_i" from 1 to (ceil (random [0,1,1])) do {_unit addItemToUniform "ACE_morphine";};
    for "_i" from 1 to (ceil (random [0,1,2])) do {_unit addItemToUniform "ACE_tourniquet";};
    for "_i" from 1 to (ceil (random [0,1,2])) do {_unit addItemToUniform "ACE_splint";};
    for "_i" from 1 to (ceil (random [0,1,2])) do {_unit addMagazine "rhs_mag_rgd5";};
    for "_i" from 1 to (ceil (random [0,0,2])) do {_unit addMagazine "rhs_mag_rdg2_white";};
    "#;

    const MOTOR_PATROL_SQF: &str = r#"
    clearWeaponCargoGlobal _hmmvw;
    clearMagazineCargoGlobal _hmmvw;
    clearItemCargoGlobal _hmmvw;
    clearBackpackCargoGlobal _hmmvw;
    ["Land_CanisterFuel_Red_F", _hmmvw] call ace_cargo_fnc_loadItem;
    "#;

    #[test]
    fn test_parse_for_loop_item_additions() {
        let items = parsers::parse_sqf_content(ASSIGN_GEAR_SQF);
        assert_eq!(items.len(), 8);
        
        // Check first item
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
        
        // Check magazine items
        assert_eq!(items[6].item_name, "rhs_mag_rgd5");
        assert_eq!(items[6].container, None);
        assert_eq!(items[7].item_name, "rhs_mag_rdg2_white");
        assert_eq!(items[7].container, None);
    }

    #[test]
    fn test_parse_complex_for_loop() {
        let input = r#"for "_i" from 1 to (ceil (random [0,1,4])) do {_unit addItemToUniform "ACE_fieldDressing"};"#;
        let items = parsers::parse_sqf_content(input);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
    }

    #[test]
    fn test_parse_simple_item_addition() {
        let input = r#"_unit addItemToUniform "ACE_fieldDressing";"#;
        let items = parsers::parse_sqf_content(input);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
    }

    #[test]
    fn test_parse_multiple_item_additions() {
        let input = r#"
        _unit addItemToUniform "ACE_fieldDressing";
        _unit addItemToVest "ACE_morphine";
        _unit addItemToBackpack "ACE_bloodIV";
        _unit addMagazine "rhs_mag_rgd5";
        "#;
        
        let items = parsers::parse_sqf_content(input);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].item_name, "ACE_fieldDressing");
        assert_eq!(items[0].container, Some("uniform".to_string()));
        assert_eq!(items[1].item_name, "ACE_morphine");
        assert_eq!(items[1].container, Some("vest".to_string()));
        assert_eq!(items[2].item_name, "ACE_bloodIV");
        assert_eq!(items[2].container, Some("backpack".to_string()));
        assert_eq!(items[3].item_name, "rhs_mag_rgd5");
        assert_eq!(items[3].container, None);
    }
} 