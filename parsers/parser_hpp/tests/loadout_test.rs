use parser_hpp::{HppParser, HppValue};
use std::fs;

#[test]
fn test_loadout_parsing() {
    let content = fs::read_to_string("tests/fixtures/loadout.hpp").unwrap();
    let parser = HppParser::new(&content).unwrap();
    let classes = parser.parse_classes();

    // Test base class
    let base_man = classes.iter().find(|c| c.name == "baseMan").unwrap();
    assert!(base_man.parent.is_none());
    assert!(base_man.properties.iter().any(|p| p.name == "displayName"));

    // Test inheritance
    let rifleman = classes.iter().find(|c| c.name == "rm").unwrap();
    assert_eq!(rifleman.parent.as_deref(), Some("baseMan"));

    // Debug output
    for (i, prop) in rifleman.properties.iter().enumerate() {
        if prop.name == "uniform" {
            println!("Found uniform property at index {}", i);
            if let HppValue::Array(values) = &prop.value {
                println!("Uniform values: {:?}", values);
            }
        }
    }

    // Test array properties
    let uniform_prop = rifleman.properties.iter().find(|p| p.name == "uniform").unwrap();
    if let HppValue::Array(uniforms) = &uniform_prop.value {
        // The quoted string is returned from the parser since LIST macros are preserved as strings
        assert!(uniforms.iter().any(|u| u.contains("usp_g3c_kp_mx_aor2")), 
                "Could not find usp_g3c_kp_mx_aor2 in: {:?}", uniforms);
    } else {
        panic!("Expected uniform to be an array");
    }

    // Test nested inheritance - look for missing class
    match classes.iter().find(|c| c.name == "cls") {
        Some(cls) => {
            assert_eq!(cls.parent.as_deref(), Some("rm_fa"));
        },
        none => {
            let available_classes = classes.iter().map(|c| &c.name).collect::<Vec<_>>();
            panic!("Class 'cls' not found. Available classes: {:?}", available_classes);
        }
    };

    // Verify all expected classes are present
    let expected_classes = vec![
        "baseMan", "rm", "ar", "aar", "rm_lat", "gren", 
        "tl", "sl", "co", "rm_fa", "cls"
    ];
    
    for class_name in expected_classes {
        assert!(
            classes.iter().any(|c| c.name == class_name),
            "Missing class: {}", class_name
        );
    }
} 