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
                let var_type = self.infer_type(expr, &name);
                
                // Collection phase
                self.current_scope = name.clone();
                self.collect_from_expression(expr);
                self.current_scope.clear();
                self.collected.variables.insert(name, var_type);
            }
        }
    }

    fn infer_type(&self, expr: &Expression, name: &str) -> VariableType {
        match expr {
            Expression::Array(elements, _) => {
                let mut types = Vec::new();
                for element in elements {
                    match element {
                        Expression::String(s, _, _) => {
                            // Store the actual string value in the array type
                            types.push(VariableType::Item(ItemKind::Item));
                        }
                        Expression::Variable(var, _) => {
                            if let Some(var_type) = self.collected.variables.get(var) {
                                types.push(var_type.clone());
                            }
                        }
                        Expression::Array(nested_elements, _) => {
                            // Handle nested arrays
                            let mut nested_types = Vec::new();
                            for nested in nested_elements {
                                if let Expression::String(_, _, _) = nested {
                                    nested_types.push(VariableType::Item(ItemKind::Item));
                                }
                            }
                            if !nested_types.is_empty() {
                                types.push(VariableType::Array(nested_types));
                            }
                        }
                        _ => {}
                    }
                }
                if !types.is_empty() {
                    VariableType::Array(types)
                } else {
                    VariableType::Unknown
                }
            }
            Expression::String(_, _, _) => VariableType::Item(ItemKind::Item),
            Expression::Variable(var, _) => {
                if let Some(var_type) = self.collected.variables.get(var) {
                    var_type.clone()
                } else {
                    VariableType::Unknown
                }
            }
            Expression::BinaryCommand(cmd, lhs, rhs, _) => {
                if let BinaryCommand::Named(name) = cmd {
                    if name == "select" {
                        if let Expression::Variable(var, _) = &**lhs {
                            if let Some(VariableType::Array(types)) = self.collected.variables.get(var) {
                                if let Some(first_type) = types.first() {
                                    return first_type.clone();
                                }
                            }
                        }
                    }
                }
                VariableType::Unknown
            }
            _ => VariableType::Unknown
        }
    }

    fn handle_function_call(&mut self, function_name: &str, args: &[Expression]) -> bool {
        match function_name {
            "ace_cargo_fnc_loaditem" | "ace_cargo_fnc_loadItem" => {
                // Expects ["Land_CanisterFuel_Red_F", _vehicle]
                if args.len() >= 2 {
                    if let Expression::String(s, _, _) = &args[1] {
                        self.collected.pending_items.push((s.to_string(), ItemKind::Item));
                        return true;
                    }
                }
            }
            "ace_arsenal_fnc_initbox" => {
                // Process all items in the array
                if args.len() >= 1 {
                    self.collect_from_expression(&args[0]);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn collect_from_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Array(elements, _) => {
                // Only process arrays in specific contexts
                if !self.current_scope.is_empty() {
                    // Process arrays in assignments
                    for element in elements {
                        match element {
                            Expression::String(s, _, _) => {
                                self.collected.pending_items.push((s.to_string(), ItemKind::Item));
                            }
                            Expression::Array(nested_elements, _) => {
                                // Process nested arrays
                                for nested in nested_elements {
                                    if let Expression::String(s, _, _) = nested {
                                        self.collected.pending_items.push((s.to_string(), ItemKind::Item));
                                    }
                                }
                            }
                            Expression::Variable(var, _) => {
                                if let Some(var_type) = self.collected.variables.get(var) {
                                    match var_type {
                                        VariableType::Item(kind) => {
                                            self.collected.pending_items.push((var.to_string(), kind.clone()));
                                        }
                                        VariableType::Array(types) => {
                                            // Only add array items in specific contexts
                                            if let Some(VariableType::Item(kind)) = types.first() {
                                                self.collected.pending_items.push((var.to_string(), kind.clone()));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Expression::BinaryCommand(cmd, lhs, rhs, _) => {
                if let BinaryCommand::Named(name) = cmd {
                    match name.as_str() {
                        "then" | "else" => {
                            // Process both branches for conditionals
                            self.collect_from_expression(lhs);
                            self.collect_from_expression(rhs);
                        }
                        "select" => {
                            // For select operations, process the array and use the index
                            if let Expression::Variable(var, _) = &**lhs {
                                if let Some(VariableType::Array(types)) = self.collected.variables.get(var) {
                                    if let Expression::Number(_, _) = &**rhs {
                                        // For now, we'll treat any select operation as accessing the first element
                                        if let Some(parent) = self.get_parent_command(expr) {
                                            if let BinaryCommand::Named(name) = parent {
                                                match name.as_str() {
                                                    "addweapon" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Weapon));
                                                        }
                                                    }
                                                    "addmagazine" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Magazine));
                                                        }
                                                    }
                                                    "addbackpack" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Backpack));
                                                        }
                                                    }
                                                    "addvest" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Vest));
                                                        }
                                                    }
                                                    "adduniform" | "forceadduniform" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Uniform));
                                                        }
                                                    }
                                                    "addheadgear" | "addgoggles" => {
                                                        if let Some(first_type) = types.first() {
                                                            self.collected.pending_items.push((var.to_string(), ItemKind::Item));
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "call" => {
                            // Handle function calls
                            if let Expression::Array(args, _) = &**lhs {
                                if let Expression::String(func_name, _, _) = &args[0] {
                                    match func_name.to_lowercase().as_str() {
                                        "ace_cargo_fnc_loaditem" | "ace_cargo_fnc_loadItem" => {
                                            // Expects ["Land_CanisterFuel_Red_F", _vehicle]
                                            if args.len() >= 2 {
                                                if let Expression::String(s, _, _) = &args[0] {
                                                    self.collected.pending_items.push((s.to_string(), ItemKind::Item));
                                                }
                                            }
                                        }
                                        "ace_arsenal_fnc_initbox" => {
                                            // Process all items in the array
                                            if args.len() >= 2 {
                                                self.collect_from_expression(&args[1]);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Some(items) = self.analyze_command(cmd, lhs, rhs) {
                                self.collected.pending_items.extend(items);
                            }
                        }
                    }
                }
            }
            Expression::Code(code) => {
                for stmt in code.content() {
                    self.process_statement(stmt);
                }
            }
            Expression::UnaryCommand(cmd, operand, _) => {
                // Handle select operations
                if let UnaryCommand::Named(name) = cmd {
                    if name == "select" {
                        if let Expression::Variable(var, _) = &**operand {
                            if let Some(VariableType::Array(types)) = self.collected.variables.get(var) {
                                if let Some(VariableType::Item(kind)) = types.first() {
                                    // Only add the item if it's being used in a command
                                    if let Some(parent) = self.get_parent_command(expr) {
                                        if let BinaryCommand::Named(name) = parent {
                                            match name.as_str() {
                                                "addweapon" | "addmagazine" | "addbackpack" | "addvest" |
                                                "adduniform" | "forceadduniform" | "addheadgear" | "addgoggles" => {
                                                    self.collected.pending_items.push((var.to_string(), kind.clone()));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_parent_command<'a>(&'a self, expr: &'a Expression) -> Option<&'a BinaryCommand> {
        match expr {
            Expression::BinaryCommand(cmd, _, _, _) => Some(cmd),
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
                    "adduniform" | "forceadduniform" => self.extract_item(rhs, ItemKind::Uniform),
                    "addheadgear" => self.extract_item(rhs, ItemKind::Item),
                    "addgoggles" => self.extract_item(rhs, ItemKind::Item),
                    "pushbackunique" => {
                        // Process the right-hand side for pushBackUnique
                        if let Expression::String(s, _, _) = rhs {
                            Some(vec![(s.to_string(), ItemKind::Item)])
                        } else if let Expression::Variable(var, _) = rhs {
                            if let Some(VariableType::Item(kind)) = self.collected.variables.get(var) {
                                Some(vec![(var.to_string(), kind.clone())])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    "then" | "else" => {
                        // Process both sides of conditional statements
                        let mut items = Vec::new();
                        if let Some(mut lhs_items) = self.analyze_command_expr(lhs) {
                            items.append(&mut lhs_items);
                        }
                        if let Some(mut rhs_items) = self.analyze_command_expr(rhs) {
                            items.append(&mut rhs_items);
                        }
                        if !items.is_empty() {
                            Some(items)
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
            _ => None
        }
    }

    fn analyze_command_expr(&self, expr: &Expression) -> Option<Vec<(String, ItemKind)>> {
        match expr {
            Expression::BinaryCommand(cmd, lhs, rhs, _) => self.analyze_command(cmd, lhs, rhs),
            Expression::Code(code) => {
                let mut items = Vec::new();
                for stmt in code.content() {
                    if let Statement::Expression(expr, _) = stmt {
                        if let Some(mut stmt_items) = self.analyze_command_expr(expr) {
                            items.append(&mut stmt_items);
                        }
                    }
                }
                if !items.is_empty() {
                    Some(items)
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn extract_item(&self, expr: &Expression, kind: ItemKind) -> Option<Vec<(String, ItemKind)>> {
        match expr {
            Expression::String(s, _, _) => Some(vec![(s.to_string(), kind)]),
            Expression::Variable(var, _) => {
                if let Some(VariableType::Array(types)) = self.collected.variables.get(var) {
                    if let Some(VariableType::Item(_)) = types.first() {
                        Some(vec![(var.to_string(), kind)])
                    } else {
                        None
                    }
                } else {
                    Some(vec![(var.to_string(), kind)])
                }
            }
            Expression::Array(elements, _) => {
                let mut items = Vec::new();
                for element in elements {
                    if let Some(mut extracted) = self.extract_item(element, kind.clone()) {
                        items.append(&mut extracted);
                    }
                }
                if !items.is_empty() {
                    Some(items)
                } else {
                    None
                }
            }
            Expression::BinaryCommand(cmd, lhs, rhs, _) => {
                if let BinaryCommand::Named(name) = cmd {
                    if name == "select" {
                        if let Expression::Variable(var, _) = &**lhs {
                            if let Some(VariableType::Array(types)) = self.collected.variables.get(var) {
                                if let Some(VariableType::Item(_)) = types.first() {
                                    return Some(vec![(var.to_string(), kind)]);
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None
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

    #[test]
    fn test_array_select() {
        // In this case we don't want to use the selected item to limit what we find
        // We want to find all the items in the array
        let code = r#"
            _weapons = ["rhs_weap_m4a1", "rhs_weap_ak74m"];
            _unit addWeapon (_weapons select 0);
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item.item_id, "rhs_weap_m4a1");
        assert_eq!(items[0].item.kind, ItemKind::Weapon);
        assert_eq!(items[1].item.item_id, "rhs_weap_ak74m");
        assert_eq!(items[1].item.kind, ItemKind::Weapon);
    }

    #[test]
    fn test_nested_arrays() {
        let code = r#"
            private _itemEquipment = [
                ["Tarkov_Uniforms_1", "V_PlateCarrier2_blk"],
                ["rhsusf_acc_eotech_552", "rhsusf_acc_compm4"]
            ];
            [_itemEquipment] call ace_arsenal_fnc_initBox;
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 4);
        assert!(items.iter().any(|i| i.item.item_id == "Tarkov_Uniforms_1"));
        assert!(items.iter().any(|i| i.item.item_id == "V_PlateCarrier2_blk"));
        assert!(items.iter().any(|i| i.item.item_id == "rhsusf_acc_eotech_552"));
        assert!(items.iter().any(|i| i.item.item_id == "rhsusf_acc_compm4"));
    }

    #[test]
    fn test_pushback_unique() {
        let code = r#"
            _itemEquipment = ["uniform1"];
            {
                _itemEquipment pushBackUnique _x;
            } forEach (primaryWeaponItems player);
            [_itemEquipment] call ace_arsenal_fnc_initBox;
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item.item_id, "uniform1");
    }

    #[test]
    fn test_conditional_item_assignment() {
        let code = r#"
            if (_unitRole == "rm_mat") then {
                _unit addBackpack "rhs_rpg_empty";
                _unit addWeapon "rhs_weap_rpg7";
            } else {
                _unit addBackpack "default_backpack";
            };
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|i| i.item.item_id == "rhs_rpg_empty" && i.item.kind == ItemKind::Backpack));
        assert!(items.iter().any(|i| i.item.item_id == "rhs_weap_rpg7" && i.item.kind == ItemKind::Weapon));
        assert!(items.iter().any(|i| i.item.item_id == "default_backpack" && i.item.kind == ItemKind::Backpack));
    }

    #[test]
    fn test_multiple_item_types() {
        let code = r#"
            _unit forceAddUniform "uniform1";
            _unit addVest "vest1";
            _unit addHeadgear "headgear1";
            _unit addGoggles "facewear1";
            _unit addBackpack "backpack1";
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 5);
        assert!(items.iter().any(|i| i.item.item_id == "uniform1" && i.item.kind == ItemKind::Uniform));
        assert!(items.iter().any(|i| i.item.item_id == "vest1" && i.item.kind == ItemKind::Vest));
        assert!(items.iter().any(|i| i.item.item_id == "headgear1"));
        assert!(items.iter().any(|i| i.item.item_id == "facewear1"));
        assert!(items.iter().any(|i| i.item.item_id == "backpack1" && i.item.kind == ItemKind::Backpack));
    }

    #[test]
    fn test_cargo_operations() {
        let code = r#"
            clearWeaponCargoGlobal _vehicle;
            clearMagazineCargoGlobal _vehicle;
            clearItemCargoGlobal _vehicle;
            clearBackpackCargoGlobal _vehicle;
            ["Land_CanisterFuel_Red_F", _vehicle] call ace_cargo_fnc_loadItem;
        "#;
        let items = process_code(code);
        
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item.item_id, "Land_CanisterFuel_Red_F");
    }
}
