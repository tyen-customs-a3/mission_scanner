use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemAddition {
    pub item_name: String,
    pub container: Option<String>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CargoOperation {
    Clear {
        vehicle: String,
        cargo_type: String,
    },
    Load {
        item: String,
        vehicle: String,
        function: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomRange {
    pub min: f32,
    pub mid: f32,
    pub max: f32,
} 