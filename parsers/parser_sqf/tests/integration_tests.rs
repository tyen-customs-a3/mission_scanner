use std::path::PathBuf;
use parser_sqf::{scan_sqf_file, ItemKind};

#[test]
fn test_scan_arsenal_file() {
    // Get the path to the example arsenal.sqf file
    let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    file_path.push("tests");
    file_path.push("example_data");
    file_path.push("arsenal.sqf");

    assert!(file_path.exists(), "Test file does not exist at path: {:?}", file_path);

    // Scan the file
    let items = scan_sqf_file(&file_path).expect("Failed to scan arsenal file");

    // Print found items for debugging
    println!("\nFound {} items:", items.len());
    for item in &items {
        println!("Item: {} ({:?})", item.item_id, item.kind);
    }

    // Define all expected items from the arsenal file
    let expected_items = vec![
        // Items from _itemEquipment array
        ("Tarkov_Uniforms_1", ItemKind::Item),
        ("V_PlateCarrier2_blk", ItemKind::Item),
        
        // Items from _itemMod array
        ("rhsusf_acc_eotech_552", ItemKind::Item),
        ("rhsusf_acc_compm4", ItemKind::Item),
        ("rhsusf_acc_grip1", ItemKind::Item),
        ("rhsusf_acc_grip2", ItemKind::Item),
        ("rhsusf_acc_grip3", ItemKind::Item),
        ("rhsusf_acc_grip4", ItemKind::Item),
        ("rhsusf_acc_grip4_bipod", ItemKind::Item),
        ("rhsusf_acc_saw_lw_bipod", ItemKind::Item),
        
        // Items from _itemWeaponRifle array
        ("rhs_weap_hk416d145", ItemKind::Item),
        ("rhs_weap_m16a4_imod", ItemKind::Item),
        ("rhsusf_spcs_ocp_saw", ItemKind::Item),
        ("rhs_weap_m4a1_m320", ItemKind::Item),
        
        // Items from _itemWeaponLAT array
        ("rhs_weap_M136", ItemKind::Item),
        
        // Items from _itemWeaponAmmo array
        ("rhs_mag_30Rnd_556x45_M855A1_Stanag", ItemKind::Item),
        ("greenmag_ammo_556x45_M855A1_60Rnd", ItemKind::Item),
        ("rhsusf_200Rnd_556x45_M855_mixed_soft_pouch", ItemKind::Item),
        ("ACE_HandFlare_Green", ItemKind::Item),
        ("ACE_HandFlare_Red", ItemKind::Item),
        ("ACE_HandFlare_White", ItemKind::Item),
        ("ACE_HandFlare_Yellow", ItemKind::Item),
        ("1Rnd_HE_Grenade_shell", ItemKind::Item),
        ("1Rnd_Smoke_Grenade_shell", ItemKind::Item),
        ("HandGrenade", ItemKind::Item),
        ("SmokeShell", ItemKind::Item),
    ];

    // Verify each expected item is found
    let mut missing_items = Vec::new();
    for (expected_id, expected_kind) in &expected_items {
        if !items.iter().any(|item| &item.item_id == expected_id && std::mem::discriminant(&item.kind) == std::mem::discriminant(expected_kind)) {
            missing_items.push(expected_id);
        }
    }

    assert!(
        missing_items.is_empty(),
        "Failed to find the following expected items: {:?}",
        missing_items
    );

    // Verify we found the correct total number of unique items
    let unique_items: std::collections::HashSet<_> = items.iter()
        .map(|item| (&item.item_id, std::mem::discriminant(&item.kind)))
        .collect();
    
    assert_eq!(
        unique_items.len(),
        expected_items.len(),
        "Found {} unique items but expected {}. Extra or duplicate items were found.",
        unique_items.len(),
        expected_items.len()
    );
}

#[test]
fn test_arsenal_array_concatenation() {
    let code = r#"
        private _weapons = ["weapon1", "weapon2"];
        private _items = ["item1", "item2"];
        [box1, (_weapons + _items)] call ace_arsenal_fnc_initBox;
    "#;
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.sqf");
    std::fs::write(&file_path, code).expect("Failed to write test file");
    
    let items = scan_sqf_file(&file_path).expect("Failed to scan test file");
    
    let expected_items = vec![
        "weapon1", "weapon2", "item1", "item2"
    ];
    
    for expected_item in expected_items {
        assert!(
            items.iter().any(|item| item.item_id == expected_item),
            "Failed to find expected item: {}",
            expected_item
        );
    }
}

#[test]
fn test_direct_item_additions() {
    let code = r#"
        _unit addItemToUniform "ACE_fieldDressing";
        _unit addItemToVest "rhs_mag_30Rnd_556x45_M855A1_Stanag";
        _unit addWeapon "rhs_weap_m4a1";
        _unit addWeaponItem ["rhs_weap_m4a1", "rhsusf_acc_grip1", true];
        _unit addMagazine "HandGrenade";
    "#;
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.sqf");
    std::fs::write(&file_path, code).expect("Failed to write test file");
    
    let items = scan_sqf_file(&file_path).expect("Failed to scan test file");
    
    let expected_items = vec![
        "ACE_fieldDressing",
        "rhs_mag_30Rnd_556x45_M855A1_Stanag",
        "rhs_weap_m4a1",
        "rhsusf_acc_grip1",
        "HandGrenade"
    ];
    
    for expected_item in expected_items {
        assert!(
            items.iter().any(|item| item.item_id == expected_item),
            "Failed to find expected item: {}",
            expected_item
        );
    }
}

#[test]
fn test_switch_statement_weapons() {
    let code = r#"
        private _unitRole = "ar";
        switch (_setWeapon) do 
        {
            case (_unitRole == "ar") : 
            {
                private _weapon = "weapon_name_ar";
                private _weaponMag = "weapon_mag_ar";
                
                _unit addWeapon _weapon;
                _unit addWeaponItem [_weapon, _weaponMag, true];
                
                for "_i" from 1 to 3 do {_unit addItemToVest _weaponMag};
            };
            default 
            {
                private _weapon = "weapon_name_default";
                private _weaponMag = "weapon_mag_default";
                
                _unit addWeapon _weapon;
                _unit addWeaponItem [_weapon, _weaponMag, true];
                for "_i" from 1 to 6 do {_unit addItemToVest _weaponMag};
            };
        };
    "#;
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.sqf");
    std::fs::write(&file_path, code).expect("Failed to write test file");
    
    let items = scan_sqf_file(&file_path).expect("Failed to scan test file");
    
    let expected_items = vec![
        "weapon_name_ar",
        "weapon_mag_ar"
    ];
    
    for expected_item in expected_items {
        assert!(
            items.iter().any(|item| item.item_id == expected_item),
            "Failed to find expected item: {}",
            expected_item
        );
    }
}

#[test]
fn test_for_loop_item_additions() {
    let code = r#"
        for "_i" from 1 to 4 do {_unit addItemToUniform "ACE_fieldDressing"};
        for "_i" from 1 to 2 do {_unit addItemToUniform "ACE_morphine"};
        for "_i" from 1 to 2 do {_unit addMagazine "rhs_mag_rgd5"};
    "#;
    
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.sqf");
    std::fs::write(&file_path, code).expect("Failed to write test file");
    
    let items = scan_sqf_file(&file_path).expect("Failed to scan test file");
    
    let expected_items = vec![
        "ACE_fieldDressing",
        "ACE_morphine",
        "rhs_mag_rgd5"
    ];
    
    for expected_item in expected_items {
        assert!(
            items.iter().any(|item| item.item_id == expected_item),
            "Failed to find expected item: {}",
            expected_item
        );
    }
} 