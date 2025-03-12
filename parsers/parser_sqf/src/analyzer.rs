use hemtt_sqf::{Expression, Statement, Statements, UnaryCommand, BinaryCommand};
use crate::models::{ItemKind, ItemReference, ItemContext, AnalysisResult};
use hemtt_sqf::parser::{run as parse_sqf, database::Database};
use hemtt_workspace::reporting::{Processed, Output, Token, Symbol};
use hemtt_workspace::position::{Position, LineCol};
use hemtt_workspace::WorkspacePath;
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use hemtt_common::config::PDriveOption;
use hemtt_workspace::Workspace;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone)]
enum VariableType {
    Item(ItemKind),
    Array(Vec<VariableType>),
    Unknown,
}

// Collection phase data structures
#[derive(Default)]
struct CollectedData {
    variables: HashMap<String, VariableType>,
    pending_items: Vec<(String, ItemKind)>,
    conditions: Vec<Expression>,
}

pub struct Analyzer {
    // Phase 1: Collection
    collected: CollectedData,
    // Phase 2: Analysis results
    items: Vec<ItemContext>,
    current_scope: String,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            collected: CollectedData::default(),
            items: Vec::new(),
            current_scope: String::new(),
        }
    }
}

impl Analyzer {
    pub fn process_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr, _) => self.collect_from_expression(expr),
            Statement::AssignGlobal(name, expr, _) | Statement::AssignLocal(name, expr, _) => {
                let name = name.clone();
                let var_type = Self::infer_type(expr);
                
                // Collection phase
                self.current_scope = name.clone();
                self.collect_from_expression(expr);
                self.current_scope.clear();
                self.collected.variables.insert(name, var_type);
            }
        }
    }

    fn collect_from_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Array(elements, _) => {
                for element in elements {
                    if let Some((name, kind)) = self.analyze_array_element(element) {
                        self.collected.pending_items.push((name, kind));
                    }
                }
            }
            Expression::BinaryCommand(cmd, lhs, rhs, _) => {
                if let Some(items) = self.analyze_command(cmd, lhs, rhs) {
                    self.collected.pending_items.extend(items);
                }
            }
            Expression::Code(code) => {
                for stmt in code.content() {
                    self.process_statement(stmt);
                }
            }
            Expression::String(_, _, _) |
            Expression::Number(_, _) |
            Expression::Boolean(_, _) |
            Expression::Variable(_, _) |
            Expression::NularCommand(_, _) |
            Expression::UnaryCommand(_, _, _) |
            Expression::ConsumeableArray(_, _) => {
                // These expressions don't contain items to collect
            }
        }
    }

    fn analyze_array_element(&self, element: &Expression) -> Option<(String, ItemKind)> {
        match element {
            Expression::String(s, _, _) => Some((s.to_string(), ItemKind::Item)),
            Expression::Variable(var, _) => {
                if let Some(VariableType::Item(kind)) = self.collected.variables.get(var) {
                    Some((var.to_string(), kind.clone()))
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn analyze_command(&self, cmd: &BinaryCommand, lhs: &Expression, rhs: &Expression) -> Option<Vec<(String, ItemKind)>> {
        match cmd {
            BinaryCommand::Named(name) => {
                match name.to_string().to_lowercase().as_str() {
                    "addweapon" => self.extract_item(rhs, ItemKind::Weapon),
                    "addmagazine" => self.extract_item(rhs, ItemKind::Magazine),
                    "addbackpack" => self.extract_item(rhs, ItemKind::Backpack),
                    "addvest" => self.extract_item(rhs, ItemKind::Vest),
                    "adduniform" => self.extract_item(rhs, ItemKind::Uniform),
                    _ => None
                }
            }
            _ => None
        }
    }

    fn extract_item(&self, expr: &Expression, kind: ItemKind) -> Option<Vec<(String, ItemKind)>> {
        match expr {
            Expression::String(s, _, _) => Some(vec![(s.to_string(), kind)]),
            Expression::Variable(var, _) => {
                match self.collected.variables.get(var) {
                    Some(VariableType::Item(_)) => Some(vec![(var.to_string(), kind)]),
                    Some(VariableType::Array(types)) => {
                        let items = types.iter().filter_map(|t| {
                            if let VariableType::Item(_) = t {
                                Some((var.to_string(), kind.clone()))
                            } else {
                                None
                            }
                        }).collect();
                        Some(items)
                    }
                    _ => None
                }
            }
            _ => None
        }
    }

    fn infer_type(expr: &Expression) -> VariableType {
        match expr {
            Expression::String(_, _, _) => VariableType::Item(ItemKind::Item),
            Expression::Array(items, _) => {
                let mut types = Vec::new();
                for item in items {
                    types.push(Self::infer_type(item));
                }
                VariableType::Array(types)
            }
            Expression::BinaryCommand(cmd, _, _, _) => {
                match cmd.as_str() {
                    "addWeapon" => VariableType::Item(ItemKind::Weapon),
                    "addMagazine" => VariableType::Item(ItemKind::Magazine),
                    "addBackpack" => VariableType::Item(ItemKind::Backpack),
                    "addVest" => VariableType::Item(ItemKind::Vest),
                    "addUniform" => VariableType::Item(ItemKind::Uniform),
                    _ => VariableType::Unknown,
                }
            }
            _ => VariableType::Unknown,
        }
    }

    // Phase 2: Analysis
    fn analyze_collected_data(&mut self) {
        // First collect all the items we need to add
        let items_to_add: Vec<_> = self.collected.pending_items.drain(..)
            .map(|(name, kind)| {
                let item_ref = ItemReference { item_id: name, kind };
                ItemContext {
                    item: item_ref,
                    conditions: self.collected.conditions.clone(),
                    scope: self.current_scope.clone(),
                }
            })
            .collect();
        
        // Then add them all at once
        self.items.extend(items_to_add);
    }

    pub fn into_result(mut self) -> AnalysisResult {
        // Run analysis phase before returning result
        self.analyze_collected_data();
        AnalysisResult {
            items: self.items,
        }
    }
}

/// Analyze SQF statements and extract item information
///
/// # Arguments
/// * `statements` - The parsed SQF statements
///
/// # Returns
/// * `Result<AnalysisResult, String>` - Analysis results or error message
pub fn analyze_sqf(statements: &Statements) -> Result<AnalysisResult, String> {
    let mut analyzer = Analyzer::default();
    
    // Phase 1: Collection
    for statement in statements.content() {
        analyzer.process_statement(statement);
    }
    
    // Phase 2: Analysis happens in into_result()
    Ok(analyzer.into_result())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemKind, ItemContext};

    fn process_code(code: &str) -> Vec<ItemContext> {
        let database = Database::a3(false);
        let workspace = Workspace::builder()
            .memory()
            .finish(None, false, &PDriveOption::Disallow)
            .unwrap();
        let test_file = workspace.join("test.sqf").unwrap();
        test_file.create_file().unwrap().write_all(code.as_bytes()).unwrap();
        
        let processed = Processed::new(
            vec![Output::Direct(Arc::new(Token::new(
                Symbol::Word(code.to_string()),
                Position::new(
                    LineCol(0, (1, 0)),
                    LineCol(code.len(), (1, code.len())),
                    test_file.clone(),
                )
            )))],
            HashMap::new(),
            vec![],
            false,
        ).unwrap();
        
        let statements = parse_sqf(&database, &processed).unwrap();
        let mut analyzer = Analyzer::default();
        
        for stmt in statements.content() {
            analyzer.process_statement(stmt);
        }
        
        analyzer.into_result().items
    }

    #[test]
    fn test_basic_item_tracking() {
        let code = r#"_unit addWeapon "rhs_weap_m4a1";"#;
        let items = process_code(code);
        println!("Final items count: {}", items.len());
        
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item.item_id, "rhs_weap_m4a1");
        assert_eq!(items[0].item.kind, ItemKind::Weapon);
    }

    #[test]
    fn test_array_call_function() {
        let code = r#"
            _weapons = ["rhs_weap_m4a1", "rhs_weap_m249"];
            _items = ["item1", "item2"];
            [_weapons + _items] call ace_arsenal_fnc_initBox;
        "#;
        let items = process_code(code);
        println!("Final items count: {}", items.len());
        
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].item.item_id, "rhs_weap_m4a1");
        assert_eq!(items[0].item.kind, ItemKind::Item);
        assert_eq!(items[1].item.item_id, "rhs_weap_m249");
        assert_eq!(items[1].item.kind, ItemKind::Item);
        assert_eq!(items[2].item.item_id, "item1");
        assert_eq!(items[2].item.kind, ItemKind::Item);
        assert_eq!(items[3].item.item_id, "item2");
        assert_eq!(items[3].item.kind, ItemKind::Item);
    }

    fn test_array_add_select() {
        let code = r#"
            _weapons = ["rhs_weap_m4a1", "rhs_weap_ak74m"];
            _unit addWeapon (_weapons select 0);
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item.item_id, "rhs_weap_m4a1");
        assert_eq!(items[0].item.kind, ItemKind::Weapon);
    }
}
