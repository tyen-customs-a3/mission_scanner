#[cfg(test)]
mod hpp_parser_tests {
    use parsers::hpp::parsers;
    use std::num::NonZeroU32;

    const LOADOUT_HPP: &str = r#"
    class rm : baseMan
    {
        displayName = "Rifleman";
        items[] = 
        {
            "ACRE_PRC343",
            LIST_10("ACE_fieldDressing"),
            LIST_10("ACE_packingBandage"),
            LIST_4("ACE_tourniquet"),
            LIST_2("ACE_epinephrine"),
            LIST_2("ACE_morphine"),
            LIST_2("ACE_splint")
        };
        magazines[] = 
        {
            LIST_2("SmokeShell"),
            LIST_2("rhs_mag_m67"),
            LIST_13("rhs_mag_30Rnd_556x45_M855A1_PMAG")
        };
    };
    "#;

    const MEDICAL_HPP: &str = r#"
    class Morphine {
        painReduce = 0.8;
        hrIncreaseLow[] = {-10, -20};
        hrIncreaseNormal[] = {-10, -30};
        hrIncreaseHigh[] = {-10, -35};
        timeInSystem = 1800;
        timeTillMaxEffect = 30;
        maxDose = 4;
        incompatibleMedication[] = {};
        viscosityChange = -10;
    };
    "#;

    #[test]
    fn test_loadout_items_array() {
        let items_section = r#"items[] = {"ACRE_PRC343", LIST_10("ACE_fieldDressing"), LIST_10("ACE_packingBandage"), LIST_4("ACE_tourniquet"), LIST_2("ACE_epinephrine"), LIST_2("ACE_morphine"), LIST_2("ACE_splint")}"#;
        
        let (_, items) = parsers::items_array(items_section).unwrap();
        assert_eq!(items.len(), 7);
        assert_eq!(items[0].item_name, "ACRE_PRC343");
        assert_eq!(items[0].count, NonZeroU32::new(1));
        assert_eq!(items[1].item_name, "ACE_fieldDressing");
        assert_eq!(items[1].count, NonZeroU32::new(10));
    }

    #[test]
    fn test_loadout_magazines_array() {
        let magazines_section = r#"magazines[] = {"SmokeShell", LIST_2("rhs_mag_m67"), LIST_13("rhs_mag_30Rnd_556x45_M855A1_PMAG")}"#;
        
        let (_, magazines) = parsers::magazines_array(magazines_section).unwrap();
        assert_eq!(magazines.len(), 3);
        assert_eq!(magazines[0].item_name, "SmokeShell");
        assert_eq!(magazines[0].count, NonZeroU32::new(1));
        assert_eq!(magazines[2].item_name, "rhs_mag_30Rnd_556x45_M855A1_PMAG");
        assert_eq!(magazines[2].count, NonZeroU32::new(13));
    }

    #[test]
    fn test_medical_empty_array() {
        let empty_array = r#"incompatibleMedication[] = {}"#;
        let (_, items) = parsers::items_array("{}").unwrap();
        assert_eq!(items.len(), 0);
    }
} 