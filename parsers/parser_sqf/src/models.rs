//! Core data structures for SQF parsing and analysis

use hemtt_sqf::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Weapon,
    Magazine,
    Backpack,
    Vest,
    Uniform,
    Item,
}

#[derive(Debug, Clone)]
pub struct ItemReference {
    pub item_id: String,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub struct ItemContext {
    pub item: ItemReference,
    pub conditions: Vec<Expression>,
    pub scope: String,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub items: Vec<ItemContext>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_kind_equality() {
        assert_eq!(ItemKind::Weapon, ItemKind::Weapon);
        assert_ne!(ItemKind::Weapon, ItemKind::Magazine);
    }

    #[test]
    fn test_item_reference_creation() {
        let item = ItemReference {
            item_id: "test_item".to_string(),
            kind: ItemKind::Item,
        };
        assert_eq!(item.item_id, "test_item");
        assert_eq!(item.kind, ItemKind::Item);
    }

    #[test]
    fn test_item_context_creation() {
        let item = ItemReference {
            item_id: "test_item".to_string(),
            kind: ItemKind::Item,
        };
        let context = ItemContext {
            item,
            conditions: vec![],
            scope: "test_scope".to_string(),
        };
        assert_eq!(context.scope, "test_scope");
        assert!(context.conditions.is_empty());
    }
} 