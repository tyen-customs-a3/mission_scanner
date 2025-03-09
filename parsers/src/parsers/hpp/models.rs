use std::num::NonZeroU32;
use serde::{Serialize, Deserialize};

/// Represents an item reference in a loadout or inventory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemReference {
    /// Name/class of the item
    pub item_name: String,
    /// Quantity of the item, must be at least 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<NonZeroU32>,
}

impl ItemReference {
    /// Creates a new ItemReference with the given name and count
    /// 
    /// # Arguments
    /// * `name` - The name/class of the item
    /// * `count` - The quantity of the item (must be > 0)
    /// 
    /// # Returns
    /// * `Some(ItemReference)` if count > 0
    /// * `None` if count == 0
    pub fn new(name: impl Into<String>, count: u32) -> Option<Self> {
        NonZeroU32::new(count).map(|c| Self {
            item_name: name.into(),
            count: Some(c),
        })
    }

    /// Creates a new ItemReference with a count of 1
    pub fn single(name: impl Into<String>) -> Self {
        Self {
            item_name: name.into(),
            count: NonZeroU32::new(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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