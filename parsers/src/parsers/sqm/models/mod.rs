use serde::{Serialize, Deserialize};

/// Represents a weapon definition in an SQM file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponDefinition {
    /// The name of the weapon
    pub weapon_name: String,
    /// The weapon's fire mode, if specified
    pub firemode: Option<String>,
    /// Any magazines associated with this weapon
    pub magazines: Vec<MagazineDefinition>,
}

/// Represents a magazine definition in an SQM file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MagazineDefinition {
    /// The name of the magazine
    pub name: String,
    /// How much ammo is left in the magazine, if specified
    pub ammo_left: Option<u32>,
}

/// Represents an item entry in cargo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemEntry {
    pub name: String,
    pub count: u32,
}

/// Represents cargo in a vehicle or container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoDefinition {
    pub items: Vec<ItemEntry>,
}

/// Represents a class definition in an SQM file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub class_type: String,
    pub type_name: Option<String>,
    pub is_backpack: Option<bool>,
    pub cargo: Option<CargoDefinition>,
    pub nested_classes: Vec<ClassDefinition>,
}

/// Represents weapon information in an SQM file (simplified version)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponInfo {
    pub name: String,
    pub fire_modes: Vec<String>,
    pub ammo_left: Option<u32>,
} 