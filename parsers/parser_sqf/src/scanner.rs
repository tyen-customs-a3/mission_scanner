use hemtt_sqf::{Expression, Statement, parser::database::Database};
use hemtt_workspace::reporting::{WorkspaceFiles, Processed, Symbol, Token, Output};
use hemtt_workspace::position::{Position, LineCol};
use hemtt_workspace::WorkspacePath;
use std::sync::Arc;
use std::collections::HashMap;
use crate::models::ItemReference;
use crate::workspace::setup_workspace;
use crate::extractors::try_extract_item;
use std::path::PathBuf;

pub fn scan_sqf(code: &str, file_path: &PathBuf) -> Result<Vec<ItemReference>, String> {
    let (position, _workspace) = setup_workspace(code, file_path)?;
    
    let token = Arc::new(Token::new(Symbol::Word(code.to_string()), position));
    let processed = Processed::new(
        vec![Output::Direct(token)],
        Default::default(),
        Vec::new(),
        false
    ).map_err(|e| format!("Failed to create Processed: {e:?}"))?;
    let database = Database::a3(false);
    
    let parsed = match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => sqf,
        Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
            let files = WorkspaceFiles::new();
            let errors: Vec<_> = e.iter()
                .map(|error| error.diagnostic().unwrap().to_string(&files))
                .collect();
            return Err(errors.join("\n"));
        }
        Err(e) => return Err(format!("Parse error: {e:?}")),
    };

    let mut items = Vec::new();
    let mut variables = HashMap::new();
    for statement in parsed.content() {
        scan_statement(statement, &mut items, &mut variables);
    }
    
    Ok(items)
}

fn scan_statement(statement: &Statement, items: &mut Vec<ItemReference>, variables: &mut HashMap<String, Vec<Expression>>) {
    match statement {
        Statement::Expression(expr, _) => {
            scan_expression(expr, items, variables);
        }
        Statement::AssignGlobal(name, expr, _) | Statement::AssignLocal(name, expr, _) => {
            // Store array assignments for later use
            if let Expression::Array(elements, _) = expr {
                variables.insert(name.clone(), elements.clone());
            }
            scan_expression(expr, items, variables);
        }
    }
}

fn scan_expression(expr: &Expression, items: &mut Vec<ItemReference>, variables: &mut HashMap<String, Vec<Expression>>) {
    match expr {
        Expression::BinaryCommand(cmd, left, right, _) => {
            let command_name = cmd.as_str();
            
            // Handle forEach loops
            if command_name == "forEach" {
                if let Expression::Code(statements) = &**left {
                    if let Expression::Variable(array_name, _) = &**right {
                        // Clone the array elements before iterating to avoid borrow checker issues
                        if let Some(array_elements) = variables.get(array_name).cloned() {
                            // For each element in the array, process the forEach body
                            for element in array_elements {
                                for statement in statements.content() {
                                    scan_statement(statement, items, variables);
                                }
                            }
                        }
                        // Return early to avoid processing the forEach body again
                        return;
                    }
                }
            }
            
            if let Some(item) = try_extract_item(command_name, right) {
                items.push(item);
            }
            
            // Recursively scan both sides
            scan_expression(left, items, variables);
            scan_expression(right, items, variables);
        }
        Expression::Code(statements) => {
            for statement in statements.content() {
                scan_statement(statement, items, variables);
            }
        }
        Expression::Array(elements, _) => {
            for element in elements {
                scan_expression(element, items, variables);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::models::ItemKind;

    fn setup_test_file(code: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.sqf");
        fs::write(&file_path, code).expect("Failed to write test file");
        (temp_dir, file_path)
    }

    #[test]
    fn test_scan_empty_code() {
        let (_temp_dir, file_path) = setup_test_file("");
        let result = scan_sqf("", &file_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_single_item() {
        let code = r#"player addBackpack "B_AssaultPack_mcamo";"#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].item_id, "B_AssaultPack_mcamo");
        assert!(matches!(result[0].kind, ItemKind::Backpack));
    }

    #[test]
    fn test_scan_array_assignment() {
        let code = r#"
            private _itemEquipment = [
                "Tarkov_Uniforms_1",
                "V_PlateCarrier2_blk"
            ];
            {
                player addItem _x;
            } forEach _itemEquipment;
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        
        println!("\nDEBUG: Found {} items:", result.len());
        for (i, item) in result.iter().enumerate() {
            println!("Item {}: id='{}', kind={:?}", i, item.item_id, item.kind);
        }
        
        // Print the variables map state
        println!("\nDEBUG: Variables state in scanner:");
        let mut variables = HashMap::new();
        let (position, _) = setup_workspace(code, &file_path).unwrap();
        let token = Arc::new(Token::new(Symbol::Word(code.to_string()), position));
        let processed = Processed::new(
            vec![Output::Direct(token)],
            Default::default(),
            Vec::new(),
            false
        ).unwrap();
        
        if let Ok(parsed) = hemtt_sqf::parser::run(&Database::a3(false), &processed) {
            for statement in parsed.content() {
                if let Statement::AssignLocal(name, Expression::Array(elements, _), _) = statement {
                    println!("\nArray '{}' contains:", name);
                    for (i, element) in elements.iter().enumerate() {
                        println!("  Element {}: {:?}", i, element);
                    }
                    variables.insert(name.clone(), elements.clone());
                }
            }
        }
        
        assert_eq!(result.len(), 2, "Expected 2 items (one for each array element), but found {}", result.len());
        assert!(result.iter().any(|item| item.item_id == "_x" && matches!(item.kind, ItemKind::Item)),
            "Expected to find item with id '_x' and kind Item");
    }

    #[test]
    fn test_scan_array_modifications() {
        let code = r#"
            private _items = ["FirstAidKit"];
            _items pushBack "Medikit";
            _items pushBackUnique "Bandage";
            {
                player addItem _x;
            } forEach _items;
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert!(result.iter().any(|item| item.item_id == "_x" && matches!(item.kind, ItemKind::Item)));
    }

    #[test]
    fn test_scan_mixed_equipment() {
        let code = r#"
            player addWeapon "rhs_weap_m4a1_m320";
            player addHeadgear "rhsusf_ach_helmet_ocp";
            player addGoggles "G_Combat";
            player addBackpack "B_AssaultPack_mcamo";
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result.iter().any(|item| item.item_id == "rhs_weap_m4a1_m320" && matches!(item.kind, ItemKind::Weapon)));
        assert!(result.iter().any(|item| item.item_id == "rhsusf_ach_helmet_ocp" && matches!(item.kind, ItemKind::Headgear)));
        assert!(result.iter().any(|item| item.item_id == "G_Combat" && matches!(item.kind, ItemKind::Goggles)));
        assert!(result.iter().any(|item| item.item_id == "B_AssaultPack_mcamo" && matches!(item.kind, ItemKind::Backpack)));
    }

    #[test]
    fn test_scan_nested_code_with_conditions() {
        let code = r#"
            if (alive player) then {
                if (primaryWeapon player == "") then {
                    player addWeapon "rhs_weap_m16a4_imod";
                } else {
                    player addItem "FirstAidKit";
                };
            };
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|item| item.item_id == "rhs_weap_m16a4_imod" && matches!(item.kind, ItemKind::Weapon)));
        assert!(result.iter().any(|item| item.item_id == "FirstAidKit" && matches!(item.kind, ItemKind::Item)));
    }

    #[test]
    fn test_scan_foreach_with_primary_items() {
        let code = r#"
            {
                player addItem _x;
            } forEach (primaryWeaponItems player);
            {
                player addItem _x;
            } forEach (handgunItems player);
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert_eq!(result.len(), 2); // Two forEach loops with addItem
        assert!(result.iter().all(|item| item.item_id == "_x" && matches!(item.kind, ItemKind::Item)));
    }

    #[test]
    fn test_scan_invalid_code() {
        let code = "this is not valid sqf code;";
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_complex_arsenal_setup() {
        let code = r#"
            private _itemEquipment = [
                "Tarkov_Uniforms_1",
                "V_PlateCarrier2_blk"
            ];

            private _itemWeaponRifle = [
                "rhs_weap_hk416d145",
                "rhs_weap_m16a4_imod"
            ];

            {
                _itemEquipment pushBackUnique _x;
            } forEach (primaryWeaponItems player);

            {
                _itemEquipment pushBackUnique _x;
            } forEach (handgunItems player);

            _itemEquipment pushBack uniform player;
            _itemEquipment pushBack vest player;
            _itemEquipment pushBack backpack player;
            _itemEquipment pushBack headgear player;

            {
                player addItem _x;
            } forEach (_itemEquipment + _itemWeaponRifle);
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        assert!(result.iter().any(|item| item.item_id == "_x" && matches!(item.kind, ItemKind::Item)));
    }
} 