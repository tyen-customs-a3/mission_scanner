//! Common models for representing mission inventory items
//! 
//! This module provides unified types for representing inventory items
//! found in various mission file formats (SQF, SQM, CPP).

use std::fmt;

/// Represents the source of an inventory item reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    /// Found in a SQF script file
    Script {
        /// Path to the script file
        file_path: String,
    },
    /// Found in mission.sqm
    Mission {
        /// Class or section where item was found
        context: String,
    },
    /// Found in a config file
    Config {
        /// Path to the config file
        file_path: String,
        /// Class or section where item was found
        class: String,
    },
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Script { file_path } => write!(f, "Script: {}", file_path),
            SourceType::Mission { context } => write!(f, "Mission: {}", context),
            SourceType::Config { file_path, class } => write!(f, "Config: {} in {}", class, file_path),
        }
    }
}

/// Type of inventory item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    Weapon,
    Magazine,
    Backpack,
    Uniform,
    Vest,
    Headgear,
    Goggles,
    NVGoggles,
    Binocular,
    Map,
    GPS,
    Radio,
    Compass,
    Watch,
    Generic,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemType::Weapon => write!(f, "Weapon"),
            ItemType::Magazine => write!(f, "Magazine"),
            ItemType::Backpack => write!(f, "Backpack"),
            ItemType::Uniform => write!(f, "Uniform"),
            ItemType::Vest => write!(f, "Vest"),
            ItemType::Headgear => write!(f, "Headgear"),
            ItemType::Goggles => write!(f, "Goggles"),
            ItemType::NVGoggles => write!(f, "NVGoggles"),
            ItemType::Binocular => write!(f, "Binocular"),
            ItemType::Map => write!(f, "Map"),
            ItemType::GPS => write!(f, "GPS"),
            ItemType::Radio => write!(f, "Radio"),
            ItemType::Compass => write!(f, "Compass"),
            ItemType::Watch => write!(f, "Watch"),
            ItemType::Generic => write!(f, "Generic"),
        }
    }
}

/// Represents an inventory item found in mission files
#[derive(Debug, Clone)]
pub struct InventoryItem {
    /// The class name or ID of the item
    pub class_name: String,
    /// The type of item
    pub item_type: ItemType,
    /// Where this item was found
    pub source: SourceType,
    /// Optional count/quantity
    pub count: Option<u32>,
    /// Optional probability of item appearing
    pub probability: Option<f32>,
}

impl InventoryItem {
    /// Creates a new inventory item
    pub fn new(
        class_name: impl Into<String>,
        item_type: ItemType,
        source: SourceType,
    ) -> Self {
        Self {
            class_name: class_name.into(),
            item_type,
            source,
            count: None,
            probability: None,
        }
    }

    /// Sets the count for this item
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Sets the probability for this item
    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = Some(probability);
        self
    }
}

impl fmt::Display for InventoryItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) from {}", self.class_name, self.item_type, self.source)?;
        if let Some(count) = self.count {
            write!(f, " x{}", count)?;
        }
        if let Some(prob) = self.probability {
            write!(f, " @{:.1}%", prob * 100.0)?;
        }
        Ok(())
    }
} 