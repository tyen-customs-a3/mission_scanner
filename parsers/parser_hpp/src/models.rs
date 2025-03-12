//! Core data structures for HPP parsing and analysis

use hemtt_workspace::reporting::Output;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ClassKind {
    Class,
    Struct,
    Enum,
    Typedef,
    Namespace,
    Unknown,
}

impl fmt::Display for ClassKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassKind::Class => write!(f, "class"),
            ClassKind::Struct => write!(f, "struct"),
            ClassKind::Enum => write!(f, "enum"),
            ClassKind::Typedef => write!(f, "typedef"),
            ClassKind::Namespace => write!(f, "namespace"),
            ClassKind::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassReference {
    pub class_id: String,
    pub kind: ClassKind,
    pub items: Vec<ItemReference>,
    pub parent_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Uniform,
    Vest,
    Backpack,
    Weapon,
    Magazine,
    Item,
    Headgear,
    Goggles,
    NVG,
    Binocular,
    Attachment,
    Unknown,
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Uniform => write!(f, "uniform"),
            ItemKind::Vest => write!(f, "vest"),
            ItemKind::Backpack => write!(f, "backpack"),
            ItemKind::Weapon => write!(f, "weapon"),
            ItemKind::Magazine => write!(f, "magazine"),
            ItemKind::Item => write!(f, "item"),
            ItemKind::Headgear => write!(f, "headgear"),
            ItemKind::Goggles => write!(f, "goggles"),
            ItemKind::NVG => write!(f, "nvg"),
            ItemKind::Binocular => write!(f, "binocular"),
            ItemKind::Attachment => write!(f, "attachment"),
            ItemKind::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemReference {
    pub class_id: String,
    pub kind: ItemKind,
    pub count: u32,
}

#[derive(Debug)]
pub struct ClassContext {
    pub class_name: String,
    pub parent_class: Option<String>,
    pub items: Vec<ItemReference>,
    pub scope: String,
}

impl Clone for ClassContext {
    fn clone(&self) -> Self {
        Self {
            class_name: self.class_name.clone(),
            parent_class: self.parent_class.clone(),
            items: self.items.clone(),
            scope: self.scope.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub classes: Vec<ClassContext>,
}

// Helper function to determine item kind from array name
pub fn determine_item_kind(array_name: &str) -> ItemKind {
    match array_name {
        "uniform" => ItemKind::Uniform,
        "vest" => ItemKind::Vest,
        "backpack" => ItemKind::Backpack,
        "primaryWeapon" | "secondaryWeapon" | "sidearmWeapon" => ItemKind::Weapon,
        "magazines" => ItemKind::Magazine,
        "items" | "linkedItems" | "backpackItems" => ItemKind::Item,
        "headgear" => ItemKind::Headgear,
        "goggles" => ItemKind::Goggles,
        "hmd" => ItemKind::NVG,
        "binocular" => ItemKind::Binocular,
        "scope" | "bipod" | "attachment" | "silencer" | 
        "secondaryAttachments" | "sidearmAttachments" => ItemKind::Attachment,
        _ => ItemKind::Unknown,
    }
}

// Helper function to parse LIST_X macro
pub fn parse_list_macro(macro_str: &str) -> Option<(u32, String)> {
    let parts: Vec<&str> = macro_str.split(['(', ')', '"'].as_ref())
        .filter(|s| !s.is_empty())
        .collect();
    
    if parts.len() >= 2 && parts[0].starts_with("LIST_") {
        if let Ok(count) = parts[0].trim_start_matches("LIST_").parse::<u32>() {
            return Some((count, parts[1].to_string()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_kind_equality() {
        assert_eq!(ClassKind::Class, ClassKind::Class);
        assert_ne!(ClassKind::Class, ClassKind::Struct);
    }

    #[test]
    fn test_class_reference_creation() {
        let class = ClassReference {
            class_id: "test_class".to_string(),
            kind: ClassKind::Class,
            items: vec![],
            parent_class: None,
        };
        assert_eq!(class.class_id, "test_class");
        assert_eq!(class.kind, ClassKind::Class);
    }

    #[test]
    fn test_class_context_creation() {
        let class = ClassReference {
            class_id: "test_class".to_string(),
            kind: ClassKind::Class,
            items: vec![],
            parent_class: None,
        };
        let context = ClassContext {
            class_name: "test_class".to_string(),
            parent_class: None,
            items: vec![],
            scope: "test_scope".to_string(),
        };
        assert_eq!(context.scope, "test_scope");
        assert!(context.items.is_empty());
    }

    #[test]
    fn test_determine_item_kind() {
        assert_eq!(determine_item_kind("uniform"), ItemKind::Uniform);
        assert_eq!(determine_item_kind("primaryWeapon"), ItemKind::Weapon);
        assert_eq!(determine_item_kind("magazines"), ItemKind::Magazine);
        assert_eq!(determine_item_kind("unknown"), ItemKind::Unknown);
    }

    #[test]
    fn test_parse_list_macro() {
        assert_eq!(
            parse_list_macro(r#"LIST_2("ACE_fieldDressing")"#),
            Some((2, "ACE_fieldDressing".to_string()))
        );
        assert_eq!(parse_list_macro("not_a_list_macro"), None);
    }
} 