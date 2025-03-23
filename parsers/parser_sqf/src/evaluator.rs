use hemtt_sqf::{Expression, Statement, Statements, BinaryCommand, UnaryCommand};
use crate::models::{ClassReference, UsageContext, AnalysisResult};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use super::array_handler::ArrayHandler;

/// Represents a value in SQF execution
#[derive(Debug, Clone, PartialEq)]
pub enum SqfValue {
    String(String),
    Array(Vec<SqfValue>),
    Partial(Vec<String>),
    Unknown,
}

/// SQF evaluator that tracks variable usage to identify class references
pub struct Evaluator {
    /// Current state of variables
    variables: HashMap<String, SqfValue>,
    /// Class references found through function usage
    references: Arc<Mutex<HashMap<String, HashSet<UsageContext>>>>,
    /// Current execution scope name
    current_scope: String,
    /// The set of function names that indicate class references
    class_reference_functions: HashSet<String>,
    /// Array handler for array operations
    array_handler: ArrayHandler,
}

impl Default for Evaluator {
    fn default() -> Self {
        // Initialize with known functions that indicate class references
        let mut class_reference_functions = HashSet::new();
        
        // Add functions
        class_reference_functions.insert("ace_arsenal_fnc_initbox".to_string());
        
        // Add commands that take class references
        for cmd in &[
            "addWeapon", "addWeaponCargo", "addWeaponGlobal", "addWeaponCargoGlobal",
            "addMagazine", "addMagazineCargo", "addMagazineGlobal", "addMagazineCargoGlobal",
            "addItem", "addItemCargo", "addItemToBackpack", "addItemToUniform", "addItemToVest",
            "addBackpack", "addBackpackCargo", "addBackpackGlobal", "addBackpackCargoGlobal",
            "addGoggles", "addHeadgear", "forceAddUniform", "addVest", "addUniform",
            "linkItem",
        ] {
            class_reference_functions.insert(cmd.to_string().to_lowercase());
        }

        // Create a new evaluator with a reference callback
        let references = Arc::new(Mutex::new(HashMap::new()));
        let variables = HashMap::new();
        let current_scope = String::new();

        // Create the array handler with a closure that captures references
        let references_clone = Arc::clone(&references);
        let array_handler = ArrayHandler::new(move |s: String, ctx: UsageContext| {
            references_clone.lock().unwrap()
                .entry(s)
                .or_insert_with(HashSet::new)
                .insert(ctx);
        });

        Self {
            variables,
            references,
            current_scope,
            class_reference_functions,
            array_handler,
        }
    }
}

impl Evaluator {
    /// Evaluate a complete SQF script
    pub fn evaluate_script(&mut self, statements: &Statements) {
        for statement in statements.content() {
            self.evaluate_statement(statement);
        }
    }

    /// Evaluate a single statement
    fn evaluate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr, _) => {
                println!("Evaluating expression");
                self.evaluate_expression(expr);
            },
            Statement::AssignGlobal(name, expr, _) | Statement::AssignLocal(name, expr, _) => {
                let var_name = name.clone();
                println!("Assigning to variable: {}", var_name);
                self.current_scope = var_name.clone();
                
                // First evaluate the expression to get any direct references
                self.evaluate_expression(expr);
                
                // Then evaluate to value for storage
                let value = self.array_handler.evaluate_expression_to_value(expr, &self.variables);
                println!("Value: {:?}", value);
                
                // Store the value for later use
                self.variables.insert(var_name, value);
                self.current_scope.clear();
            }
        }
    }

    /// Evaluate an expression and track class reference usage
    fn evaluate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::BinaryCommand(cmd, lhs, rhs, _) => {
                if let BinaryCommand::Named(name) = cmd {
                    let cmd_name = name.to_string();
                    let cmd_name_lower = cmd_name.to_lowercase();
                    println!("Processing command: {}", cmd_name);
                    
                    // Check if this is a function call that indicates class references
                    if cmd_name_lower == "call" {
                        if let Expression::Variable(func_name, _) = &**rhs {
                            if self.class_reference_functions.contains(&func_name.to_string().to_lowercase()) {
                                println!("Found class reference function: {}", func_name);
                                // Handle known function that takes class references
                                self.handle_class_reference_function(&func_name.to_string(), lhs);
                                return;
                            }
                        }
                    } 
                    // Check if this is a command that takes class references
                    else if self.class_reference_functions.contains(&cmd_name_lower) {
                        println!("Found class reference command: {}", cmd_name);
                        // For add* commands, we don't care about the left operand (target unit)
                        // We only care about the right operand which contains the class name
                        if let Expression::String(s, _, _) = &**rhs {
                            self.add_reference(s.to_string(), UsageContext::AddCommand(cmd_name));
                        } else {
                            // If the right operand is not a direct string, try to extract class references
                            self.extract_class_from_expression(rhs, UsageContext::AddCommand(cmd_name));
                        }
                        return;
                    }
                    // Handle selectRandomWeighted command
                    else if cmd_name_lower == "selectrandomweighted" {
                        println!("Processing selectRandomWeighted");
                        // Extract strings from the array argument
                        if let Expression::Array(elements, _) = &**lhs {
                            for (i, element) in elements.iter().enumerate() {
                                if i % 2 == 0 { // Even indices are items, odd are weights
                                    if let Expression::String(s, _, _) = element {
                                        println!("Found selectRandomWeighted item: {}", s);
                                        // Store the string in current scope if we have one
                                        if !self.current_scope.is_empty() {
                                            println!("Adding reference in scope {}: {}", self.current_scope, s);
                                            self.add_reference(s.to_string(), UsageContext::DirectReference);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Handle array operations
                    else if cmd_name_lower == "+" || cmd_name_lower == "pushback" || cmd_name_lower == "pushbackunique" {
                        // For array operations, evaluate both sides to capture any references
                        self.evaluate_expression(lhs);
                        self.evaluate_expression(rhs);
                        
                        // Handle the array operation
                        if let Expression::Variable(var_name, _) = &**lhs {
                            if let Some(value) = self.array_handler.handle_array_operation(
                                &cmd_name_lower,
                                lhs,
                                rhs,
                                &self.variables,
                                UsageContext::DirectReference
                            ) {
                                self.variables.insert(var_name.to_string(), value);
                            }
                        }
                        return;
                    }
                }
                
                // Process both sides of the binary command
                self.evaluate_expression(lhs);
                self.evaluate_expression(rhs);
            },
            Expression::Array(elements, _) => {
                // Process array elements recursively
                for element in elements {
                    self.evaluate_expression(element);
                }
            },
            Expression::String(s, _, _) => {
                // Only add string as reference if we're in a known class reference context
                if !self.current_scope.is_empty() {
                    self.add_reference(s.to_string(), UsageContext::DirectReference);
                }
            },
            Expression::Code(code) => {
                // Process code blocks
                for stmt in code.content() {
                    self.evaluate_statement(stmt);
                }
            },
            Expression::UnaryCommand(cmd, operand, _) => {
                if let UnaryCommand::Named(name) = cmd {
                    if self.class_reference_functions.contains(&name.to_string().to_lowercase()) {
                        // Some unary commands might take class references
                        self.extract_class_from_expression(operand, UsageContext::AddCommand(name.to_string().to_lowercase()));
                        return;
                    }
                }
                self.evaluate_expression(operand);
            },
            _ => {}
        }
    }

    /// Extract class references from an expression based on a usage context
    fn extract_class_from_expression(&mut self, expr: &Expression, context: UsageContext) {
        let mut result = Vec::new();
        self.array_handler.extract_array_values(expr, &self.variables, &mut result);
        
        // Process extracted class names
        for class_name in result {
            self.add_reference(class_name, context.clone());
        }
    }

    /// Handle functions known to use class references (like ace_arsenal_fnc_initBox)
    fn handle_class_reference_function(&mut self, func_name: &str, args: &Expression) {
        let context = UsageContext::KnownFunction(func_name.to_string());
        
        // Extract arguments based on the function
        if func_name.to_lowercase() == "ace_arsenal_fnc_initbox" {
            // ace_arsenal_fnc_initBox can be called with [box, items] or just [items]
            if let Expression::Array(elements, _) = args {
                // Get the items argument (either first or second element depending on call format)
                let items_arg = if elements.len() >= 2 {
                    &elements[1]
                } else if elements.len() == 1 {
                    &elements[0]
                } else {
                    return;
                };
                
                // Extract class references from the items argument
                self.extract_class_from_expression(items_arg, context);
            }
        } else {
            // For other known functions, just process all arguments
            self.extract_class_from_expression(args, context);
        }
    }

    /// Add a class reference with usage context
    fn add_reference(&mut self, class_name: String, context: UsageContext) {
        self.references.lock().unwrap()
            .entry(class_name)
            .or_insert_with(HashSet::new)
            .insert(context);
    }

    /// Get all found class references with their contexts
    pub fn into_result(self) -> AnalysisResult {
        let mut references = Vec::new();
        let refs = self.references.lock().unwrap();
        for (class_name, contexts) in refs.iter() {
            for context in contexts {
                references.push(ClassReference {
                    class_name: class_name.clone(),
                    context: context.to_string(),
                });
            }
        }
        AnalysisResult { references }
    }

    /// Get a reference to the set of class reference functions
    pub fn get_class_reference_functions(&self) -> &HashSet<String> {
        &self.class_reference_functions
    }

    /// Quick check if content contains any class reference functions
    /// Uses a buffered reader to efficiently scan large files
    pub fn should_evaluate<R: std::io::BufRead>(reader: R) -> bool {
        // Create default evaluator to get the function set
        let evaluator = Self::default();
        let functions = evaluator.get_class_reference_functions();
        
        // Convert all functions to lowercase once
        let functions_lower: HashSet<String> = functions.iter()
            .map(|f| f.to_lowercase())
            .collect();
            
        // Buffer for the current line
        let mut line_buffer = String::new();
        
        // Read the file line by line
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    line_buffer.clear();
                    line_buffer.push_str(&line.to_lowercase());
                    
                    // Check if any function exists in this line
                    if functions_lower.iter().any(|func| line_buffer.contains(func)) {
                        return true;
                    }
                }
                Err(_) => break
            }
        }
        
        false
    }
}

/// Evaluate an SQF script to extract all class references
pub fn evaluate_sqf(statements: &Statements) -> Result<AnalysisResult, String> {
    let mut evaluator = Evaluator::default();
    evaluator.evaluate_script(statements);
    Ok(evaluator.into_result())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemtt_sqf::parser::{run as parse_sqf, database::Database};
    use hemtt_workspace::reporting::{Processed, Output, Token, Symbol};
    use hemtt_workspace::position::{Position, LineCol};
    use std::sync::Arc;
    use hemtt_common::config::PDriveOption;
    use hemtt_workspace::Workspace;
    use std::io::Write;

    fn evaluate_code(code: &str) -> Vec<ClassReference> {
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
        evaluate_sqf(&statements).unwrap().references
    }

    #[test]
    fn test_add_commands() {
        let code = r#"
            _unit addWeapon "rhs_weap_m4a1";
            _unit addVest "some_vest";
            _unit forceAddUniform "some_uniform";
        "#;
        let references = evaluate_code(code);
        
        // All items should be found with proper contexts
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"rhs_weap_m4a1".to_string()));
        assert!(reference_names.contains(&"some_vest".to_string()));
        assert!(reference_names.contains(&"some_uniform".to_string()));
    }

    #[test]
    fn test_selectrandomweighted() {
        let code = r#"
            private _uniformPool = selectRandomWeighted 
            [
                "uniform1", 3,
                "uniform2", 2
            ];
            _unit forceAddUniform _uniformPool;
        "#;
        let references = evaluate_code(code);
        
        // Print out what we found for debugging
        println!("Found references:");
        for reference in &references {
            println!("  {} ({})", reference.class_name, reference.context);
        }
        
        // Both uniforms should be found
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"uniform1".to_string()));
        assert!(reference_names.contains(&"uniform2".to_string()));
    }

    #[test]
    fn test_arsenal_function() {
        let code = r#"
            _weapons = ["weapon1", "weapon2"];
            _items = ["item1", "item2"];
            [_box, (_weapons + _items)] call ace_arsenal_fnc_initBox;
        "#;
        let references = evaluate_code(code);
        
        // All items should be found through the arsenal function call
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"weapon1".to_string()));
        assert!(reference_names.contains(&"weapon2".to_string()));
        assert!(reference_names.contains(&"item1".to_string()));
        assert!(reference_names.contains(&"item2".to_string()));
    }

    #[test]
    fn test_array_operations() {
        let code = r#"
            _weapons = ["weapon1", "weapon2"];
            _items = ["item1", "item2"];
            _combined = _weapons + _items;
            _more_weapons = ["weapon3"];
            _more_weapons pushBack "weapon4";
            _more_weapons pushBackUnique "weapon4"; // Duplicate, shouldn't be added twice
            _more_weapons pushBackUnique "weapon5";
            _all_items = _combined + _more_weapons;
            [_box, _all_items] call ace_arsenal_fnc_initBox;
        "#;
        let references = evaluate_code(code);
        
        // All items should be found properly
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("weapon1"));
        assert!(reference_names.contains("weapon2"));
        assert!(reference_names.contains("item1"));
        assert!(reference_names.contains("item2"));
        assert!(reference_names.contains("weapon3"));
        assert!(reference_names.contains("weapon4"));
        assert!(reference_names.contains("weapon5"));
        assert_eq!(reference_names.len(), 7); // Ensure no duplicates
    }

    #[test]
    fn test_direct_add_commands_with_variables() {
        let code = r#"
            _weapon = "rhs_weap_m4a1_blockII";
            _vest = "rhsusf_spcs_ocp";
            _unit addWeapon _weapon;
            _unit addVest _vest;
        "#;
        let references = evaluate_code(code);
        
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"rhs_weap_m4a1_blockII".to_string()));
        assert!(reference_names.contains(&"rhsusf_spcs_ocp".to_string()));
    }

    #[test]
    fn test_nested_variables() {
        let code = r#"
            _primary_weapons = ["rhs_weap_m4a1", "rhs_weap_m16a4"];
            _secondary_weapons = ["rhs_weap_M136", "rhs_weap_M72A7"];
            _all_weapons = _primary_weapons + _secondary_weapons;
            _magazines = ["rhs_mag_30Rnd_556x45_M855A1_Stanag"];
            _equipment = _all_weapons + _magazines;
            [_box, _equipment] call ace_arsenal_fnc_initBox;
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("rhs_weap_m4a1"));
        assert!(reference_names.contains("rhs_weap_m16a4"));
        assert!(reference_names.contains("rhs_weap_M136"));
        assert!(reference_names.contains("rhs_weap_M72A7"));
        assert!(reference_names.contains("rhs_mag_30Rnd_556x45_M855A1_Stanag"));
        assert_eq!(reference_names.len(), 5);
    }

    #[test]
    fn test_complex_array_building() {
        let code = r#"
            _rifle_magazines = [];
            _rifle_magazines pushBack "rhs_mag_30Rnd_556x45_M855A1_Stanag";
            _rifle_magazines pushBack "rhs_mag_30Rnd_556x45_M855A1_PMAG";
            
            _pistol_magazines = ["rhsusf_mag_15Rnd_9x19_JHP"];
            
            _launchers = ["rhs_weap_M136", "rhs_weap_m72a7"];
            _launcher_ammo = []; // empty array
            
            _all_equipment = _rifle_magazines + _pistol_magazines + _launchers + _launcher_ammo;
            
            // The array is passed to another variable first
            _arsenal_items = _all_equipment;
            [_box, _arsenal_items] call ace_arsenal_fnc_initBox;
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("rhs_mag_30Rnd_556x45_M855A1_Stanag"));
        assert!(reference_names.contains("rhs_mag_30Rnd_556x45_M855A1_PMAG"));
        assert!(reference_names.contains("rhsusf_mag_15Rnd_9x19_JHP"));
        assert!(reference_names.contains("rhs_weap_M136"));
        assert!(reference_names.contains("rhs_weap_m72a7"));
        assert_eq!(reference_names.len(), 5);
    }

    #[test]
    fn test_add_item_to_container() {
        let code = r#"
            _unit addItemToUniform "ACE_fieldDressing";
            _unit addItemToVest "ACE_morphine";
            _unit addItemToBackpack "ACE_bloodIV";
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("ACE_fieldDressing"));
        assert!(reference_names.contains("ACE_morphine"));
        assert!(reference_names.contains("ACE_bloodIV"));
    }
    
    #[test]
    fn test_add_equipment() {
        let code = r#"
            _unit addHeadgear "rhsusf_ach_helmet_ocp";
            _unit addGoggles "rhs_goggles_black";
            _unit addWeapon "Binocular";
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("rhsusf_ach_helmet_ocp"));
        assert!(reference_names.contains("rhs_goggles_black"));
        assert!(reference_names.contains("Binocular"));
    }

    #[test]
    fn test_add_headgear() {
        let code = r#"
            _unit addHeadgear "rhsusf_ach_helmet_ocp";
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("rhsusf_ach_helmet_ocp"));
    }
    
    #[test]
    fn test_add_goggles() {
        let code = r#"
            _unit addGoggles "rhs_goggles_black";
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("rhs_goggles_black"));
    }
    
    #[test]
    fn test_add_binocular() {
        let code = r#"
            _unit addWeapon "Binocular";
        "#;
        let references = evaluate_code(code);
        
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains("Binocular"));
    }

    #[test]
    fn test_should_evaluate() {
        let content_with_match = "player addWeapon \"rhs_weap_m4a1\";";
        assert!(Evaluator::should_evaluate(std::io::BufReader::new(content_with_match.as_bytes())));
        
        let content_with_arsenal = "items call ace_arsenal_fnc_initBox;";
        assert!(Evaluator::should_evaluate(std::io::BufReader::new(content_with_arsenal.as_bytes())));
        
        let content_without_match = "player setPos [0, 0, 0]; hint \"No class references\";";
        assert!(!Evaluator::should_evaluate(std::io::BufReader::new(content_without_match.as_bytes())));
    }

    #[test]
    fn test_mixed_case_commands() {
        let code = r#"
            // Test case insensitivity for commands
            _unit AddWeapon "rhs_weap_m4a1"; // Capital A
            _unit addVEST "rhsusf_spcs_ocp"; // Capital letters
            _box CALL ace_arsenal_fnc_initBox; // All caps CALL
        "#;
        let references = evaluate_code(code);
        
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"rhs_weap_m4a1".to_string()));
        assert!(reference_names.contains(&"rhsusf_spcs_ocp".to_string()));
    }

    #[test]
    fn test_direct_arsenal_calls() {
        let code = r#"
            [arsenal, ["rhs_weap_m4a1", "rhsusf_spcs_ocp"]] call ace_arsenal_fnc_initBox;
        "#;
        let references = evaluate_code(code);
        
        let reference_names: Vec<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
            
        assert!(reference_names.contains(&"rhs_weap_m4a1".to_string()));
        assert!(reference_names.contains(&"rhsusf_spcs_ocp".to_string()));
    }

    #[test]
    fn test_real_arsenal_file() {
        let code = include_str!("../tests/example_data/arsenal.sqf");
        let references = evaluate_code(code);
        
        // Convert to a HashSet of class names for easier verification
        let reference_names: HashSet<_> = references.iter()
            .map(|r| r.class_name.clone())
            .collect();
        
        // Non-equipment items that should be ignored
        let non_equipment = [
            "personal_arsenal", 
            "startpos", 
            "building", 
            "\\A3\\ui_f\\data\\igui\\cfg\\weaponicons\\MG_ca.paa", 
            "Personal Arsenal"
        ];
        
        // Count valid equipment references (excluding non-equipment items)
        let valid_equipment_count = reference_names.iter()
            .filter(|name| !non_equipment.contains(&name.as_str()))
            .count();
        
        // Test rifles
        assert!(reference_names.contains("rhs_weap_hk416d145"));
        assert!(reference_names.contains("rhs_weap_m16a4_imod"));
        assert!(reference_names.contains("rhs_weap_m4a1_m320"));
        assert!(reference_names.contains("rhs_weap_M136"));
        
        // Test magazines
        assert!(reference_names.contains("rhs_mag_30Rnd_556x45_M855A1_Stanag"));
        assert!(reference_names.contains("rhsusf_200Rnd_556x45_M855_mixed_soft_pouch"));
        
        // Test equipment
        assert!(reference_names.contains("Tarkov_Uniforms_1"));
        assert!(reference_names.contains("V_PlateCarrier2_blk"));
        assert!(reference_names.contains("rhsusf_spcs_ocp_saw"));
        
        // Test accessories
        assert!(reference_names.contains("rhsusf_acc_eotech_552"));
        assert!(reference_names.contains("rhsusf_acc_compm4"));
        assert!(reference_names.contains("rhsusf_acc_grip1"));
        
        // Test ACE items
        assert!(reference_names.contains("ACE_HandFlare_Green"));
        
        // Test grenades
        assert!(reference_names.contains("1Rnd_HE_Grenade_shell"));
        assert!(reference_names.contains("HandGrenade"));
        assert!(reference_names.contains("SmokeShell"));
        
        // Ensure we found a significant number of valid equipment items
        assert!(valid_equipment_count >= 25, "Should find at least 25 valid equipment references, found: {}", valid_equipment_count);
        
        // Print out all valid equipment references for debugging
        println!("Found {} valid equipment references:", valid_equipment_count);
        for name in reference_names.iter().filter(|name| !non_equipment.contains(&name.as_str())) {
            println!("  {}", name);
        }
    }
} 