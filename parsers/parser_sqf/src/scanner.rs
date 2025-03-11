use hemtt_sqf::{Expression, Statement, parser::database::Database};
use hemtt_workspace::reporting::{WorkspaceFiles, Processed, Symbol, Token, Output};
use hemtt_workspace::position::{Position, LineCol};
use hemtt_workspace::WorkspacePath;
use std::sync::Arc;
use std::collections::HashMap;
use crate::models::ItemReference;
use crate::workspace::setup_workspace;
use crate::extractors::{try_extract_item, try_extract_items_from_array};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum ArrayValue {
    String(String),
    Array(Vec<Expression>),
    Concatenation(Vec<String>),
}

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

fn scan_statement(statement: &Statement, items: &mut Vec<ItemReference>, variables: &mut HashMap<String, ArrayValue>) {
    match statement {
        Statement::Expression(expr, _) => {
            scan_expression(expr, items, variables);
        }
        Statement::AssignGlobal(name, expr, _) | Statement::AssignLocal(name, expr, _) => {
            println!("Processing assignment: {} = {:?}", name, expr);
            match expr {
                Expression::Array(elements, _) => {
                    variables.insert(name.clone(), ArrayValue::Array(elements.clone()));
                    println!("Stored array {} with {} elements", name, elements.len());
                }
                Expression::String(s, _, _) => {
                    variables.insert(name.clone(), ArrayValue::String(s.to_string()));
                    println!("Stored string {} = {}", name, s);
                }
                _ => scan_expression(expr, items, variables),
            }
        }
    }
}

fn scan_expression(expr: &Expression, items: &mut Vec<ItemReference>, variables: &mut HashMap<String, ArrayValue>) {
    match expr {
        Expression::BinaryCommand(cmd, left, right, _) => {
            let command_name = cmd.as_str();
            println!("Processing command: {}", command_name);
            
            match command_name {
                "call" => {
                    // Check if we're calling ace_arsenal_fnc_initBox
                    if let Expression::Variable(func_name, _) = &**right {
                        if func_name == "ace_arsenal_fnc_initBox" {
                            println!("Found ace_arsenal_fnc_initBox call");
                            // Extract the array argument from the left side
                            if let Expression::Array(args, _) = &**left {
                                if args.len() >= 2 {
                                    println!("Processing arsenal args: {:?}", args[1]);
                                    match &args[1] {
                                        Expression::BinaryCommand(op, left, right, _) if op.as_str() == "+" => {
                                            println!("Found array concatenation: {:?} + {:?}", left, right);
                                            scan_array_concatenation(left, items, variables);
                                            scan_array_concatenation(right, items, variables);
                                        }
                                        Expression::Variable(var_name, _) => {
                                            println!("Found variable reference: {}", var_name);
                                            if let Some(array_value) = variables.get(var_name) {
                                                match array_value {
                                                    ArrayValue::Array(elements) => {
                                                        println!("Found array with {} elements", elements.len());
                                                        for element in elements {
                                                            if let Expression::String(s, _, _) = element {
                                                                println!("Adding item: {}", s);
                                                                items.push(ItemReference {
                                                                    item_id: s.to_string(),
                                                                    kind: crate::models::ItemKind::Item,
                                                                });
                                                            }
                                                        }
                                                    }
                                                    _ => println!("Unexpected array value type"),
                                                }
                                            } else {
                                                println!("Variable {} not found in variables map", var_name);
                                            }
                                        }
                                        Expression::Array(elements, _) => {
                                            println!("Found direct array with {} elements", elements.len());
                                            for element in elements {
                                                if let Expression::String(s, _, _) = element {
                                                    println!("Adding direct array item: {}", s);
                                                    items.push(ItemReference {
                                                        item_id: s.to_string(),
                                                        kind: crate::models::ItemKind::Item,
                                                    });
                                                }
                                            }
                                        }
                                        _ => println!("Unhandled arsenal argument type: {:?}", args[1]),
                                    }
                                }
                            }
                        }
                    }
                }
                "addItemToUniform" | "addItemToVest" | "addItemToBackpack" | "addMagazine" => {
                    if let Expression::String(item_id, _, _) = &**right {
                        println!("Adding item from {}: {}", command_name, item_id);
                        items.push(ItemReference {
                            item_id: item_id.to_string(),
                            kind: crate::models::ItemKind::Item,
                        });
                    } else if let Expression::Variable(var_name, _) = &**right {
                        if let Some(ArrayValue::String(item_id)) = variables.get(var_name) {
                            println!("Adding item from variable {}: {}", var_name, item_id);
                            items.push(ItemReference {
                                item_id: item_id.clone(),
                                kind: crate::models::ItemKind::Item,
                            });
                        }
                    }
                }
                "addWeapon" => {
                    if let Expression::String(weapon_id, _, _) = &**right {
                        println!("Adding weapon: {}", weapon_id);
                        items.push(ItemReference {
                            item_id: weapon_id.to_string(),
                            kind: crate::models::ItemKind::Item,
                        });
                    } else if let Expression::Variable(var_name, _) = &**right {
                        if let Some(ArrayValue::String(weapon_id)) = variables.get(var_name) {
                            println!("Adding weapon from variable: {}", weapon_id);
                            items.push(ItemReference {
                                item_id: weapon_id.clone(),
                                kind: crate::models::ItemKind::Item,
                            });
                        }
                    }
                }
                "addWeaponItem" => {
                    if let Expression::Array(args, _) = &**right {
                        if args.len() >= 2 {
                            if let Expression::String(item_id, _, _) = &args[1] {
                                println!("Adding weapon item: {}", item_id);
                                items.push(ItemReference {
                                    item_id: item_id.to_string(),
                                    kind: crate::models::ItemKind::Item,
                                });
                            } else if let Expression::Variable(var_name, _) = &args[1] {
                                if let Some(ArrayValue::String(item_id)) = variables.get(var_name) {
                                    println!("Adding weapon item from variable: {}", item_id);
                                    items.push(ItemReference {
                                        item_id: item_id.clone(),
                                        kind: crate::models::ItemKind::Item,
                                    });
                                }
                            }
                        }
                    }
                }
                "do" => {
                    // Handle for loop body
                    scan_expression(right, items, variables);
                }
                _ => if let Some(item) = try_extract_item(command_name, right) {
                    items.push(item);
                }
            }
            
            scan_expression(left, items, variables);
            scan_expression(right, items, variables);
        }
        Expression::Code(statements) => {
            // Handle code blocks (used in for loops and switch cases)
            for statement in statements.content() {
                scan_statement(statement, items, variables);
            }
        }
        _ => {}
    }
}

fn scan_array_concatenation(expr: &Expression, items: &mut Vec<ItemReference>, variables: &HashMap<String, ArrayValue>) {
    match expr {
        Expression::Variable(var_name, _) => {
            println!("Scanning variable in concatenation: {}", var_name);
            // Try the exact variable name first
            let array_value = variables.get(var_name).or_else(|| {
                // If not found, try with _itemWeapon prefix
                if var_name == "_itemLAT" {
                    variables.get("_itemWeaponLAT")
                } else {
                    None
                }
            });
            
            if let Some(array_value) = array_value {
                match array_value {
                    ArrayValue::Array(elements) => {
                        println!("Found array with {} elements", elements.len());
                        for element in elements {
                            if let Expression::String(s, _, _) = element {
                                println!("Adding item from array: {}", s);
                                items.push(ItemReference {
                                    item_id: s.to_string(),
                                    kind: crate::models::ItemKind::Item,
                                });
                            }
                        }
                    }
                    ArrayValue::String(s) => {
                        println!("Adding string item: {}", s);
                        items.push(ItemReference {
                            item_id: s.clone(),
                            kind: crate::models::ItemKind::Item,
                        });
                    }
                    ArrayValue::Concatenation(strings) => {
                        for s in strings {
                            println!("Adding concatenated item: {}", s);
                            items.push(ItemReference {
                                item_id: s.clone(),
                                kind: crate::models::ItemKind::Item,
                            });
                        }
                    }
                }
            } else {
                println!("Variable {} not found in variables map", var_name);
            }
        }
        Expression::Array(elements, _) => {
            println!("Found direct array with {} elements", elements.len());
            for element in elements {
                if let Expression::String(s, _, _) = element {
                    println!("Adding direct array item: {}", s);
                    items.push(ItemReference {
                        item_id: s.to_string(),
                        kind: crate::models::ItemKind::Item,
                    });
                }
            }
        }
        Expression::BinaryCommand(op, left, right, _) if op.as_str() == "+" => {
            println!("Found nested concatenation: {:?} + {:?}", left, right);
            scan_array_concatenation(left, items, variables);
            scan_array_concatenation(right, items, variables);
        }
        _ => {
            println!("Unhandled expression in concatenation: {:?}", expr);
        }
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
            [box1, _itemEquipment] call ace_arsenal_fnc_initBox;
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|item| item.item_id == "Tarkov_Uniforms_1"));
        assert!(result.iter().any(|item| item.item_id == "V_PlateCarrier2_blk"));
    }

    #[test]
    fn test_scan_array_concatenation() {
        let code = r#"
            private _weapons = ["weapon1", "weapon2"];
            private _items = ["item1", "item2"];
            [box1, (_weapons + _items)] call ace_arsenal_fnc_initBox;
        "#;
        let (_temp_dir, file_path) = setup_test_file(code);
        let result = scan_sqf(code, &file_path).unwrap();
        
        assert_eq!(result.len(), 4);
        assert!(result.iter().any(|item| item.item_id == "weapon1"));
        assert!(result.iter().any(|item| item.item_id == "weapon2"));
        assert!(result.iter().any(|item| item.item_id == "item1"));
        assert!(result.iter().any(|item| item.item_id == "item2"));
    }
} 