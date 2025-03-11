use std::sync::Arc;
use std::ops::Range;
use hemtt_sqf::Expression;
use hemtt_workspace::position::Position;
use crate::models::{ItemReference, ItemKind};

pub fn extract_string_arg(expr: &Expression) -> Option<String> {
    match expr {
        Expression::String(s, _, _) => Some(s.to_string()),
        Expression::Variable(name, _) => Some(name.clone()),
        _ => None
    }
}

pub fn try_extract_item(command_name: &str, arg: &Expression) -> Option<ItemReference> {
    let item_id = extract_string_arg(arg)?;
    
    let kind = match command_name {
        "addBackpack" => Some(ItemKind::Backpack),
        "addWeapon" => Some(ItemKind::Weapon),
        "addHeadgear" => Some(ItemKind::Headgear),
        "addGoggles" => Some(ItemKind::Goggles),
        "addItem" => Some(ItemKind::Item),
        "ace_arsenal_fnc_initBox" => Some(ItemKind::Item),
        _ => None
    }?;

    Some(ItemReference { item_id, kind })
}

pub fn try_extract_items_from_array(expr: &Expression) -> Vec<ItemReference> {
    match expr {
        Expression::Array(elements, _) => {
            elements.iter()
                .filter_map(|element| {
                    extract_string_arg(element)
                        .map(|item_id| ItemReference {
                            item_id,
                            kind: ItemKind::Item
                        })
                })
                .collect()
        },
        Expression::Variable(name, _) => {
            vec![ItemReference {
                item_id: name.clone(),
                kind: ItemKind::Item
            }]
        },
        _ => Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemtt_sqf::Expression;
    use hemtt_sqf::StringWrapper;
    use std::ops::Range;

    fn create_test_string(value: &str) -> Expression {
        Expression::String(
            Arc::from(value),
            0..value.len(),
            StringWrapper::DoubleQuote
        )
    }

    fn create_test_variable(name: &str) -> Expression {
        Expression::Variable(
            name.to_string(),
            0..name.len()
        )
    }

    #[test]
    fn test_extract_string_literal() {
        let expr = create_test_string("test_item");
        assert_eq!(extract_string_arg(&expr), Some("test_item".to_string()));
    }

    #[test]
    fn test_extract_variable() {
        let expr = create_test_variable("_itemVar");
        assert_eq!(extract_string_arg(&expr), Some("_itemVar".to_string()));
    }

    #[test]
    fn test_extract_backpack() {
        let expr = create_test_string("B_AssaultPack_mcamo");
        let item = try_extract_item("addBackpack", &expr).unwrap();
        assert_eq!(item.item_id, "B_AssaultPack_mcamo");
        assert!(matches!(item.kind, ItemKind::Backpack));
    }
} 