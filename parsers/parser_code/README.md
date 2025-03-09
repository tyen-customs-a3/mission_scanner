# HPP Parser Module

This module provides parsers for Arma 3 HPP (header) files, which are used to define:
- Loadouts (equipment configurations)
- Medical item properties
- Other configuration data

## Features

- Parse item arrays from loadout definitions
- Parse magazine arrays from loadout definitions
- Parse medical item properties from ACE3 medical configs
- Support for LIST_X macros (e.g., LIST_10("ACE_fieldDressing"))

## Usage

```rust
use parser_code::hpp::parsers;
use parser_code::hpp::models::{ItemReference, MedicalItemProperties};

// Parse items from a loadout
let loadout_hpp = r#"
items[] = 
{
    "ACRE_PRC343",
    LIST_10("ACE_fieldDressing"),
    LIST_2("ACE_morphine")
};
"#;

let (_, items) = parsers::items_array(loadout_hpp).unwrap();
println!("Parsed {} items:", items.len());
for item in items {
    println!("  - {} x{}", item.item_name, item.count.map_or(1, |c| c.get()));
}

// Parse medical item properties
let medical_hpp = r#"
class Morphine {
    painReduce = 0.8;
    hrIncreaseLow[] = {-10, -20};
    hrIncreaseNormal[] = {-10, -30};
    hrIncreaseHigh[] = {-10, -35};
    timeInSystem = 1800;
    timeTillMaxEffect = 30;
    maxDose = 4;
    viscosityChange = -10;
};
"#;

let (_, props) = parsers::medical_item_properties(medical_hpp).unwrap();
println!("Medical item: {}", props.class_name);
println!("Pain reduction: {:?}", props.pain_reduce);
```

## Models

### ItemReference

Represents an item with an optional count:

```rust
pub struct ItemReference {
    pub item_name: String,
    pub count: Option<NonZeroU32>,
}
```

### MedicalItemProperties

Represents properties of a medical item in ACE3:

```rust
pub struct MedicalItemProperties {
    pub class_name: String,
    pub pain_reduce: Option<f32>,
    pub hr_increase_low: Option<Vec<i32>>,
    pub hr_increase_normal: Option<Vec<i32>>,
    pub hr_increase_high: Option<Vec<i32>>,
    pub time_in_system: Option<u32>,
    pub time_till_max_effect: Option<u32>,
    pub max_dose: Option<u32>,
    pub viscosity_change: Option<i32>,
}
``` 