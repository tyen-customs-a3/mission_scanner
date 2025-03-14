#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::{PathBuf, Path};
    use std::io::Write;
    use parser_sqf::{parse_file, ItemKind};
    use hemtt_workspace::{Workspace, LayerType, WorkspacePath};
    use hemtt_common::config::PDriveOption;
    use log::debug;
    use std::fs::File;
    use tempfile::tempdir;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_arsenal_file_parsing() {
        init();

        // Get the test file path
        let test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("example_data")
            .join("arsenal.sqf");
        
        debug!("Test file path: {:?}", test_file_path);
        assert!(test_file_path.exists(), "Test file does not exist at {:?}", test_file_path);
        
        // Parse the file using the workspace file
        let result = parse_file(&test_file_path)
            .expect("Failed to parse arsenal.sqf");

        // Verify items that should have specific types based on how they're used
        let weapons = vec![
            "rhs_weap_hk416d145",
            "rhs_weap_m16a4_imod",
            "rhs_weap_m4a1_m320",
            "rhs_weap_M136"
        ];
        for weapon in weapons {
            assert!(
                result.iter().any(|item| item.class_name == weapon && item.kind == ItemKind::Weapon),
                "Expected weapon '{}' not found or has wrong type",
                weapon
            );
        }

        let magazines = vec![
            "rhs_mag_30Rnd_556x45_M855A1_Stanag",
            "rhsusf_200Rnd_556x45_M855_mixed_soft_pouch"
        ];
        for magazine in magazines {
            assert!(
                result.iter().any(|item| item.class_name == magazine && item.kind == ItemKind::Magazine),
                "Expected magazine '{}' not found or has wrong type",
                magazine
            );
        }

        // Check uniform
        assert!(
            result.iter().any(|item| item.class_name == "Tarkov_Uniforms_1" && item.kind == ItemKind::Uniform),
            "Expected uniform not found or has wrong type"
        );

        // Check vest
        assert!(
            result.iter().any(|item| item.class_name == "V_PlateCarrier2_blk" && item.kind == ItemKind::Vest),
            "Expected vest not found or has wrong type"
        );

        // Check backpack
        assert!(
            result.iter().any(|item| item.class_name == "rhsusf_spcs_ocp_saw" && item.kind == ItemKind::Backpack),
            "Expected backpack not found or has wrong type"
        );

        // All other items should be found but with ItemKind::Item
        let generic_items = vec![
            "rhsusf_acc_eotech_552",
            "rhsusf_acc_compm4",
            "rhsusf_acc_grip1",
            "rhsusf_acc_grip2",
            "rhsusf_acc_grip3",
            "rhsusf_acc_grip4",
            "rhsusf_acc_grip4_bipod",
            "rhsusf_acc_saw_lw_bipod",
            "ACE_HandFlare_Green",
            "ACE_HandFlare_Red",
            "ACE_HandFlare_White",
            "ACE_HandFlare_Yellow",
            "1Rnd_HE_Grenade_shell",
            "1Rnd_Smoke_Grenade_shell",
            "HandGrenade",
            "SmokeShell"
        ];
        for item in generic_items {
            assert!(
                result.iter().any(|i| i.class_name == item && i.kind == ItemKind::Item),
                "Expected generic item '{}' not found or has wrong type",
                item
            );
        }
    }

    #[test]
    fn test_getvariable_not_treated_as_item() {
        init();

        // Create a temporary directory for our test file
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_file_path = temp_dir.path().join("test_getvariable.sqf");
        
        // Create test content
        let test_content = r#"
        private _unitRole = _unit getVariable ["tmf_assignGear_role", nil];
        private _unitRoleCommon = ["rm","rm_lat","rm_mat", "medic", "engineer"];
        private _unitRoleForceBackpack = ["rm_mat", "medic", "engineer"];
        "#;
        
        // Write the test content to the file
        let mut file = File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content.as_bytes()).expect("Failed to write test content");
        
        // Parse the file
        let result = parse_file(&test_file_path).expect("Failed to parse test_getvariable.sqf");
        
        // Verify that "tmf_assignGear_role" is not treated as an item
        assert!(
            !result.iter().any(|item| item.class_name == "tmf_assignGear_role"),
            "getVariable parameter 'tmf_assignGear_role' should not be treated as an item"
        );
        
        // These items should NOT be found as they are just in arrays not used in item-related functions
        let unexpected_items = vec!["rm", "rm_lat", "rm_mat", "medic", "engineer"];
        for item in unexpected_items {
            assert!(
                !result.iter().any(|found_item| found_item.class_name == item),
                "Unexpected item '{}' was found but should not be treated as an item",
                item
            );
        }
    }

    #[test]
    fn test_function_parameters_not_treated_as_items() {
        init();

        // Create a temporary directory for our test file
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_file_path = temp_dir.path().join("test_function_params.sqf");
        
        // Create test content with lineIntersectsSurfaces function
        let test_content = r#"
        private _trace = lineIntersectsSurfaces [eyePos _unit, eyePos _unit vectorAdd [0, 0, 10], _unit, objNull, true, -1, "GEOM", "NONE", true];
        private _surfaces = lineIntersectsSurfaces [_start, _end, _ignore, _ignore, true, 1, "FIRE", "VIEW"];
        "#;
        
        // Write the test content to the file
        let mut file = File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content.as_bytes()).expect("Failed to write test content");
        
        // Parse the file
        let result = parse_file(&test_file_path).expect("Failed to parse test_function_params.sqf");
        
        // Verify that string literals used as function parameters are not treated as items
        let unexpected_items = vec!["GEOM", "NONE", "FIRE", "VIEW"];
        for item in unexpected_items {
            assert!(
                !result.iter().any(|found_item| found_item.class_name == item),
                "Function parameter '{}' was incorrectly treated as an item",
                item
            );
        }
    }

    #[test]
    fn test_diary_records_not_treated_as_items() {
        init();

        // Create a temporary directory for our test file
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_file_path = temp_dir.path().join("test_diary.sqf");
        
        // Create test content with diary records
        let test_content = r#"
        /* ===============================================
            GENERAL BRIEFING NOTES
             - Uses HTML style syntax. All supported tags can be found here - https://community.bistudio.com/wiki/createDiaryRecord
             - For images use <img image='FILE'></img> (for those familiar with HTML note it is image rather than src).
             - Note that using the " character inside the briefing block is forbidden use ' instead of ".
        */

        /* ===============================================
            SITUATION
             - Outline of what is going on, where we are we and what has happened before the mission has started? This needs to contain any relevant background information.
             - Draw attention to friendly and enemy forces in the area. The commander will make important decisions based off this information.
             - Outline present weather conditions, players will typically assume that it is daylight with sunny weather.
        */

        private _situation = ["diary", ["Situation","

        <font size='18'>ENEMY FORCES</font>
        <br/>
        Platoon strength infantry guarding the town. motorized, mechanized and heliborne infantry within the area will likely respond.
        <br/><br/>

        <font size='18'>FRIENDLY FORCES</font>
        <br/>
        an upsized squad of enthusiastic and eclectically armed guerrillas.

        "]];

        /* ===============================================
            MISSION
             - Describe any objectives that the team is expected to complete.
             - Summarize(!) the overall task. This MUST be short and clear.
        */

        private _mission = ["diary", ["Mission","
        <br/>
        Destroy or steal all ammo caches in the town of abdera to the south.
        <br/><br/>
        Retreat to the north after mission completion
        "]];

        /* ===============================================
            EXECUTION
             - Provide an outline as to what the commander of the player's command might give.
        */

        private _execution = ["diary", ["Execution","
        <br/>
        <font size='18'>COMMANDER'S INTENT</font>
        <br/>
        Raid the outpost set up within the town of abdera and destroy or steal all weapons and ammo caches found. retreat to the north after mission is acomplished, move quickly after contact is made to avoid getting caught by enemy reinforcements.
        <br/><br/>

        <font size='18'>MOVEMENT PLAN</font>
        <br/>
        Advance on foot at squad leads discretion.
        <br/><br/>


        <font size='18'>SPECIAL TASKS</font>
        <br/>
        due to state of our own weaponry, use of enemy weaponry from caches and soldiers may be recomended.
        "]];

        /* ===============================================
            ADMINISTRATION
             - Outline of logistics: available resources (equipment/vehicles) and ideally a summary of their capabilities.
             - Outline of how to use any mission specific features/scripts.
             - Seating capacities of each vehicle available for use.
        */

        private _administration = ["diary", ["Administration","
        <br/>
        no logistics support, use what you can find.
        "]];

        player createDiaryRecord _administration;
        player createDiaryRecord _execution;
        player createDiaryRecord _mission;
        player createDiaryRecord _situation;
        "#;
        
        // Write the test content to the file
        let mut file = File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content.as_bytes()).expect("Failed to write test content");
        
        // Parse the file
        let result = parse_file(&test_file_path).expect("Failed to parse test_diary.sqf");
        
        // Verify that no items are found in the diary record creation code
        assert!(
            result.is_empty(),
            "Found {} items in diary record code when none were expected: {:?}",
            result.len(),
            result.iter().map(|item| &item.class_name).collect::<Vec<_>>()
        );
    }
} 