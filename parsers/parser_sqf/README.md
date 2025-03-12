# SQF Parser Module

This module provides parsers for Arma 3 SQF (Scripting Format) files, which are used to:
- Parse arsenal and loadout scripts
- Extract item usage and types from function context
- Analyze SQF code structure and item references

## Features

- Parse items from arsenal and loadout scripts
- Infer item types based on function context (e.g., `addWeapon`, `addMagazine`)
- Support for various item kinds:
  - Weapons
  - Magazines
  - Uniforms
  - Vests
  - Backpacks
  - Generic items (attachments, grenades, etc.)

## Usage

```rust
use parser_sqf::analyzer::analyze_sqf;
use parser_sqf::models::ItemKind;
use hemtt_sqf::parser::{run as parse_sqf, database::Database};

// Parse items from an arsenal script
let sqf_code = r#"
_unit addWeapon "rhs_weap_m4a1";
_unit addMagazine "rhs_mag_30Rnd_556x45_M855A1_Stanag";
_unit addUniform "Tarkov_Uniforms_1";
"#;

// Setup and parse
let database = Database::a3(false);
let statements = parse_sqf(&database, &processed).unwrap();
let result = analyze_sqf(&statements).unwrap();

// Access parsed items
for item in result.items {
    println!("Item: {}", item.item.item_id);
    println!("Type: {:?}", item.item.kind);
    
    match item.item.kind {
        ItemKind::Weapon => println!("Found weapon"),
        ItemKind::Magazine => println!("Found magazine"),
        ItemKind::Uniform => println!("Found uniform"),
        ItemKind::Vest => println!("Found vest"),
        ItemKind::Backpack => println!("Found backpack"),
        ItemKind::Item => println!("Found generic item"),
    }
}
```

## Models

### ItemReference

Represents an item with its type and context:

```rust
pub struct ItemReference {
    pub item_id: String,
    pub kind: ItemKind,
}
```

### ItemKind

Represents the type of an item based on its usage:

```rust
pub enum ItemKind {
    Weapon,
    Magazine,
    Uniform,
    Vest,
    Backpack,
    Item,  // Generic/default type
}
```

## Type Inference

The parser infers item types based on the function context in which they appear:

- `addWeapon` → `ItemKind::Weapon`
- `addMagazine` → `ItemKind::Magazine`
- `addUniform` → `ItemKind::Uniform`
- `addVest` → `ItemKind::Vest`
- `addBackpack` → `ItemKind::Backpack`
- Other contexts → `ItemKind::Item`

This approach ensures accurate type identification based on actual usage rather than relying on naming conventions. 