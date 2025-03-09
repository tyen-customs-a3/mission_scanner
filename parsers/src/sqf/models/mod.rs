use serde::{Serialize, Deserialize};

/// Represents an item being added to a unit's inventory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemAddition {
    /// Name/class of the item being added
    pub item_name: String,
    /// Container the item is being added to (uniform, vest, backpack)
    pub container: Option<String>,
    /// Quantity of items being added
    pub count: Option<u32>,
}

/// Represents a cargo operation on a vehicle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CargoOperation {
    /// Clear all cargo of a specific type
    Clear {
        /// Vehicle the cargo is being cleared from
        vehicle: String,
        /// Type of cargo being cleared (weapon, magazine, item, backpack)
        cargo_type: String,
    },
    /// Load an item into vehicle cargo
    Load {
        /// Item being loaded
        item: String,
        /// Vehicle the item is being loaded into
        vehicle: String,
        /// Function used to load the item (e.g. ace_cargo_fnc_loadItem)
        function: String,
    },
}

/// Represents a random number range in SQF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomRange {
    /// Minimum value
    pub min: f32,
    /// Most likely value
    pub mid: f32,
    /// Maximum value
    pub max: f32,
} 