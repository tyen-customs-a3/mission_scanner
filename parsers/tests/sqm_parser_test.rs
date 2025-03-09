#[cfg(test)]
mod sqm_parser_tests {
    // TODO: Implement SQM parser module
    // use parsers::sqm::parsers;

    const EXAMPLE_MISSION_SQM: &str = r#"
    class Mission
    {
        class Entities
        {
            class Item2
            {
                class Entities
                {
                    class Item1
                    {
                        class Inventory
                        {
                            class primaryWeapon
                            {
                                name="rhs_weap_mg42";
                                firemode="rhs_weap_mg42:manual";
                                class primaryMuzzleMag
                                {
                                    name="rhsgref_50Rnd_792x57_SmE_drum";
                                    ammoLeft=50;
                                };
                            };
                            class handgun
                            {
                                name="rhsusf_weap_glock17g4";
                                class primaryMuzzleMag
                                {
                                    name="rhsusf_mag_17Rnd_9x19_JHP";
                                    ammoLeft=17;
                                };
                            };
                        };
                    };
                };
            };
        };
    };
    "#;

    // Tests are commented out until SQM parser is implemented
    /*
    #[test]
    fn test_extract_weapons_from_mission() {
        let weapons = sqm::extract_weapons_simple(EXAMPLE_MISSION_SQM);
        assert_eq!(weapons.len(), 2);
        
        // Check primary weapon
        assert_eq!(weapons[0].name, "rhs_weap_mg42");
        assert_eq!(weapons[0].fire_modes, vec!["rhs_weap_mg42:manual"]);
        assert_eq!(weapons[0].ammo_left, Some(50));
        
        // Check handgun
        assert_eq!(weapons[1].name, "rhsusf_weap_glock17g4");
        assert_eq!(weapons[1].ammo_left, Some(17));
    }

    #[test]
    fn test_extract_weapons_simple() {
        let input = r#"
        class primaryWeapon
        {
            name="rhs_weap_mg42";
            fireMode="rhs_weap_mg42:manual";
            class primaryMuzzleMag
            {
                name="rhsgref_50Rnd_792x57_SmE_drum";
                ammoLeft=50;
            };
        };
        "#;
        
        let weapons = sqm::extract_weapons_simple(input);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].name, "rhs_weap_mg42");
        assert_eq!(weapons[0].fire_modes, vec!["rhs_weap_mg42:manual"]);
        assert_eq!(weapons[0].ammo_left, Some(50));
    }

    #[test]
    fn test_extract_weapons_with_multiple_mags() {
        let input = r#"
        class primaryWeapon
        {
            name="rhs_weap_m4a1";
            fireMode="Single";
            class primaryMuzzleMag
            {
                name="rhs_mag_30Rnd_556x45_M855A1_PMAG";
                ammoLeft=30;
            };
            class secondaryMuzzleMag
            {
                name="rhs_mag_30Rnd_556x45_M855A1_PMAG_Tracer_Red";
                ammoLeft=30;
            };
        };
        "#;
        
        let weapons = sqm::extract_weapons_simple(input);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].name, "rhs_weap_m4a1");
        assert_eq!(weapons[0].fire_modes, vec!["Single"]);
        assert_eq!(weapons[0].ammo_left, Some(30));
    }
    */
} 