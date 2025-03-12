use std::collections::HashMap;
use std::sync::Arc;

use hemtt_workspace::reporting::{Output, Processed, Symbol, Token};
use log::{debug, trace};

use crate::models::{ItemKind, ItemReference, ClassContext, AnalysisResult, determine_item_kind, parse_list_macro};

#[derive(Default)]
struct CollectedData {
    current_class: Option<String>,
    current_array: Option<String>,
    current_parent: Option<String>,
    items: Vec<ItemReference>,
    in_array: bool,
    in_string: bool,
    current_string: String,
    is_append: bool,  // Track if we're appending to an array
    expect_array_value: bool,  // Track if we're expecting an array value after =
}

pub struct Analyzer {
    collected: CollectedData,
    classes: Vec<ClassContext>,
    current_scope: String,
    class_stack: Vec<String>,  // Track nested class context
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            collected: CollectedData::default(),
            classes: Vec::new(),
            current_scope: String::new(),
            class_stack: Vec::new(),
        }
    }
}

impl Analyzer {
    fn process_token(&mut self, token: Arc<Token>) {
        trace!("Processing token: {:?}", token.symbol());
        
        match token.symbol() {
            Symbol::Word(word) => {
                if self.collected.in_string {
                    trace!("Adding word to string: {}", word);
                    self.collected.current_string.push_str(&word);
                } else if word == "class" {
                    debug!("Found new class definition");
                    self.finish_current_class();
                    self.collected.current_class = None;
                } else if self.collected.current_class.is_none() && !word.contains('"') {
                    debug!("Found class name: {}", word);
                    self.collected.current_class = Some(word.to_string());
                } else if word == ":" {
                    trace!("Found inheritance operator");
                    self.collected.current_parent = None;
                } else if self.collected.current_parent.is_none() && self.collected.current_class.is_some() && !word.ends_with("[]") {
                    debug!("Found parent class: {}", word);
                    self.collected.current_parent = Some(word.to_string());
                } else if word.ends_with("[]") {
                    let array_name = word.trim_end_matches("[]");
                    debug!("Found array: {}", array_name);
                    self.collected.current_array = Some(array_name.to_string());
                    self.collected.expect_array_value = true;
                    self.collected.in_array = false;  // Reset array state
                } else if self.collected.in_string {
                    // Handle words inside strings (like LIST macros)
                    self.collected.current_string.push_str(&word);
                }
            }
            Symbol::Equals => {
                if self.collected.current_array.is_some() {
                    debug!("Found array assignment");
                    self.collected.expect_array_value = true;
                    self.collected.is_append = false;
                }
            }
            Symbol::Plus => {
                if self.collected.current_array.is_some() {
                    debug!("Found array append operator");
                    self.collected.is_append = true;
                }
            }
            Symbol::DoubleQuote => {
                if self.collected.in_string {
                    trace!("End of string: {}", self.collected.current_string);
                    if self.collected.in_array {
                        if let Some(array_name) = &self.collected.current_array {
                            let kind = determine_item_kind(array_name);
                            debug!("Processing array item in '{}' array", array_name);
                            
                            if self.collected.current_string.starts_with("LIST_") {
                                trace!("Found LIST macro: {}", self.collected.current_string);
                                if let Some((count, item)) = parse_list_macro(&self.collected.current_string) {
                                    debug!("Parsed LIST macro: {} x {}", count, item);
                                    self.collected.items.push(ItemReference {
                                        class_id: item,
                                        kind: kind.clone(),
                                        count,
                                    });
                                }
                            } else if !self.collected.current_string.is_empty() {
                                debug!("Adding item: {} ({})", self.collected.current_string, kind);
                                self.collected.items.push(ItemReference {
                                    class_id: self.collected.current_string.clone(),
                                    kind,
                                    count: 1,
                                });
                            }
                        }
                    }
                    self.collected.current_string.clear();
                }
                self.collected.in_string = !self.collected.in_string;
            }
            Symbol::Unicode(content) => {
                if self.collected.in_string {
                    trace!("Adding unicode content to string: {}", content);
                    self.collected.current_string.push_str(&content);
                }
            }
            Symbol::LeftBrace => {
                trace!("Start of array/class content");
                if self.collected.expect_array_value {
                    debug!("Starting array content");
                    self.collected.in_array = true;
                    self.collected.expect_array_value = false;
                } else if let Some(class_name) = &self.collected.current_class {
                    debug!("Entering class scope: {}", class_name);
                    
                    // Create and store the current class before pushing it onto the stack
                    let context = ClassContext {
                        class_name: class_name.clone(),
                        parent_class: self.collected.current_parent.take(),
                        items: Vec::new(),
                        scope: if self.current_scope.is_empty() {
                            class_name.clone()
                        } else {
                            format!("{}::{}", self.current_scope, class_name)
                        },
                    };
                    self.classes.push(context);
                    
                    // Update scope tracking
                    if !self.current_scope.is_empty() {
                        self.current_scope.push_str("::");
                    }
                    self.current_scope.push_str(class_name);
                    self.class_stack.push(class_name.clone());
                    
                    // Clear current class as we've processed it
                    self.collected.current_class = None;
                }
            }
            Symbol::RightBrace => {
                if self.collected.in_array {
                    debug!("End of array: {:?}", self.collected.current_array);
                    self.collected.in_array = false;
                    self.collected.current_array = None;
                    self.collected.is_append = false;
                    self.collected.expect_array_value = false;
                } else if !self.class_stack.is_empty() {
                    let class_name = self.class_stack.pop().unwrap();
                    debug!("Exiting class scope: {}", class_name);
                    
                    // Move collected items to the current class
                    if !self.collected.items.is_empty() {
                        if let Some(class_context) = self.classes.iter_mut()
                            .find(|c| c.scope == self.current_scope) {
                            class_context.items.append(&mut self.collected.items);
                        }
                    }
                    
                    // Update scope
                    if let Some(pos) = self.current_scope.rfind("::") {
                        self.current_scope.truncate(pos);
                    } else {
                        self.current_scope.clear();
                    }
                }
            }
            Symbol::Semicolon => {
                trace!("End of statement");
                if self.collected.in_array {
                    debug!("End of array statement");
                    // Move collected items to the current class
                    if !self.collected.items.is_empty() {
                        if let Some(class_context) = self.classes.iter_mut()
                            .find(|c| c.scope == self.current_scope) {
                            class_context.items.append(&mut self.collected.items);
                        }
                    }
                    
                    self.collected.in_array = false;
                    self.collected.current_array = None;
                    self.collected.is_append = false;
                    self.collected.expect_array_value = false;
                }
            }
            _ => {
                trace!("Ignoring token: {:?}", token.symbol());
            }
        }
    }

    fn finish_current_class(&mut self) {
        if let Some(class_name) = &self.collected.current_class {
            debug!("Finishing class '{}' with {} items", class_name, self.collected.items.len());
            trace!("Class items: {:?}", self.collected.items);
            
            // Move any remaining items to the class
            if !self.collected.items.is_empty() {
                if let Some(class_context) = self.classes.iter_mut()
                    .find(|c| c.scope == self.current_scope) {
                    class_context.items.append(&mut self.collected.items);
                }
            }
        }
    }

    pub fn into_result(mut self) -> AnalysisResult {
        debug!("Converting analyzer state to final result");
        self.finish_current_class();
        AnalysisResult {
            classes: self.classes,
        }
    }
}

/// Analyze preprocessed HPP content and extract class information
///
/// # Arguments
/// * `processed` - The preprocessed HPP content
///
/// # Returns
/// * `Result<AnalysisResult, String>` - Analysis results or error message
pub fn analyze_hpp(processed: &Processed) -> Result<AnalysisResult, String> {
    debug!("Starting HPP analysis");
    let mut analyzer = Analyzer::default();
    
    // Process each token in sequence
    for output in processed.raw_mappings() {
        trace!("Processing token: {:?}", output.token().symbol());
        analyzer.process_token(output.token().clone());
    }
    
    debug!("Completed HPP analysis");
    Ok(analyzer.into_result())
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use hemtt_workspace::{Workspace, LayerType};
    use hemtt_common::config::PDriveOption;
    use hemtt_preprocessor::Processor;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    fn process_code(code: &str) -> Vec<ClassContext> {
        let workspace = Workspace::builder()
            .memory()
            .finish(None, false, &PDriveOption::Disallow)
            .unwrap();
        let test_file = workspace.join("test.hpp").unwrap();
        test_file.create_file().unwrap().write_all(code.as_bytes()).unwrap();
        
        let processed = Processor::run(&test_file).unwrap();
        let mut analyzer = Analyzer::default();
        
        debug!("Processing test code:\n{}", code);
        for output in processed.raw_mappings() {
            analyzer.process_token(output.token().clone());
        }
        
        analyzer.into_result().classes
    }

    #[test]
    fn test_basic_class_with_items() {
        init();
        debug!("Running test_basic_class_with_items");
        
        let code = r#"
            class Soldier {
                uniform[] = {"uniform_item"};
                vest[] = {"vest_item"};
                weapons[] = {
                    "primary_weapon",
                    LIST_2("secondary_weapon")
                };
            };
        "#;
        let classes = process_code(code);
        
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].class_name, "Soldier");
        assert_eq!(classes[0].items.len(), 4);
        
        // Check uniform
        assert!(classes[0].items.iter().any(|item| 
            item.class_id == "uniform_item" && 
            item.kind == ItemKind::Uniform &&
            item.count == 1
        ));
        
        // Check vest
        assert!(classes[0].items.iter().any(|item| 
            item.class_id == "vest_item" && 
            item.kind == ItemKind::Vest &&
            item.count == 1
        ));
        
        // Check weapons
        assert!(classes[0].items.iter().any(|item| 
            item.class_id == "primary_weapon" && 
            item.kind == ItemKind::Weapon &&
            item.count == 1
        ));
        
        assert!(classes[0].items.iter().any(|item| 
            item.class_id == "secondary_weapon" && 
            item.kind == ItemKind::Weapon &&
            item.count == 2
        ));
    }

    #[test]
    fn test_class_inheritance() {
        init();
        debug!("Running test_class_inheritance");
        
        let code = r#"
            class Soldier {
                uniform[] = {"base_uniform"};
            };
            class Medic : Soldier {
                backpack[] = {"medic_backpack"};
            };
        "#;
        let classes = process_code(code);
        
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].class_name, "Soldier");
        assert_eq!(classes[1].class_name, "Medic");
        assert_eq!(classes[1].parent_class, Some("Soldier".to_string()));
        
        // Check items
        assert!(classes[0].items.iter().any(|item| item.class_id == "base_uniform"));
        assert!(classes[1].items.iter().any(|item| item.class_id == "medic_backpack"));
    }

    #[test]
    fn test_array_append() {
        init();
        debug!("Running test_array_append");
        
        let code = r#"
            class Base {
                items[] = {"base_item"};
            };
            class Derived : Base {
                items[] += {"extra_item"};
            };
        "#;
        let classes = process_code(code);
        
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].class_name, "Base");
        assert_eq!(classes[1].class_name, "Derived");
        
        // Check items
        assert!(classes[0].items.iter().any(|item| item.class_id == "base_item"));
        assert!(classes[1].items.iter().any(|item| item.class_id == "extra_item"));
    }

    #[test]
    fn test_array_assignment_types() {
        init();
        debug!("Running test_array_assignment_types");
        
        let code = r#"
            class Equipment {
                // Simple array assignment
                magazines[] = {"mag1"};
                
                // Multi-item array
                weapons[] = {
                    "weapon1",
                    "weapon2"
                };
                
                // Array with LIST macro
                grenades[] = {
                    LIST_3("grenade1"),
                    "grenade2"
                };
                
                // Empty array
                misc[] = {};
            };
        "#;
        let classes = process_code(code);
        
        assert_eq!(classes.len(), 1);
        let equipment = &classes[0];
        assert_eq!(equipment.class_name, "Equipment");
        
        // Check magazines
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "mag1" && 
            item.kind == ItemKind::Magazine &&
            item.count == 1
        ));
        
        // Check weapons
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "weapon1" && 
            item.kind == ItemKind::Weapon &&
            item.count == 1
        ));
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "weapon2" && 
            item.kind == ItemKind::Weapon &&
            item.count == 1
        ));
        
        // Check grenades
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "grenade1" && 
            item.count == 3
        ));
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "grenade2" && 
            item.count == 1
        ));
    }

    #[test]
    fn test_array_append_operations() {
        init();
        debug!("Running test_array_append_operations");
        
        let code = r#"
            class BaseLoadout {
                weapons[] = {"primary"};
                magazines[] = {"mag1"};
            };
            
            class ExtendedLoadout : BaseLoadout {
                // Append to weapons
                weapons[] += {"secondary"};
                
                // Append multiple items
                magazines[] += {
                    "mag2",
                    LIST_2("mag3")
                };
                
                // New array with append
                grenades[] += {"grenade1"};
            };
        "#;
        let classes = process_code(code);
        
        assert_eq!(classes.len(), 2);
        let base = &classes[0];
        let extended = &classes[1];
        
        // Check base loadout
        assert_eq!(base.class_name, "BaseLoadout");
        assert!(base.items.iter().any(|item| 
            item.class_id == "primary" && 
            item.kind == ItemKind::Weapon
        ));
        assert!(base.items.iter().any(|item| 
            item.class_id == "mag1" && 
            item.kind == ItemKind::Magazine
        ));
        
        // Check extended loadout
        assert_eq!(extended.class_name, "ExtendedLoadout");
        assert_eq!(extended.parent_class, Some("BaseLoadout".to_string()));
        
        assert!(extended.items.iter().any(|item| 
            item.class_id == "secondary" && 
            item.kind == ItemKind::Weapon &&
            item.count == 1
        ));
        
        assert!(extended.items.iter().any(|item| 
            item.class_id == "mag2" && 
            item.kind == ItemKind::Magazine &&
            item.count == 1
        ));
        
        assert!(extended.items.iter().any(|item| 
            item.class_id == "mag3" && 
            item.kind == ItemKind::Magazine &&
            item.count == 2
        ));
        
        assert!(extended.items.iter().any(|item| 
            item.class_id == "grenade1" && 
            item.count == 1
        ));
    }

    #[test]
    fn test_nested_arrays_and_scopes() {
        init();
        debug!("Running test_nested_arrays_and_scopes");
        
        let code = r#"
            class Squad {
                class Soldier1 {
                    weapons[] = {"rifle1"};
                    magazines[] = {"mag1"};
                };
                
                class Soldier2 {
                    weapons[] = {"rifle2"};
                    magazines[] = {
                        "mag2",
                        // Nested array content
                        LIST_2("mag3")
                    };
                    
                    class Equipment {
                        items[] = {"item1"};
                    };
                };
            };
        "#;
        let classes = process_code(code);
        
        // Check class hierarchy and scopes
        assert!(classes.iter().any(|c| c.class_name == "Soldier1" && c.scope == "Squad::Soldier1"));
        assert!(classes.iter().any(|c| c.class_name == "Soldier2" && c.scope == "Squad::Soldier2"));
        assert!(classes.iter().any(|c| c.class_name == "Equipment" && c.scope == "Squad::Soldier2::Equipment"));
        
        // Check items in nested scopes
        let soldier1 = classes.iter().find(|c| c.class_name == "Soldier1").unwrap();
        assert!(soldier1.items.iter().any(|item| 
            item.class_id == "rifle1" && 
            item.kind == ItemKind::Weapon
        ));
        
        let soldier2 = classes.iter().find(|c| c.class_name == "Soldier2").unwrap();
        assert!(soldier2.items.iter().any(|item| 
            item.class_id == "rifle2" && 
            item.kind == ItemKind::Weapon
        ));
        assert!(soldier2.items.iter().any(|item| 
            item.class_id == "mag3" && 
            item.kind == ItemKind::Magazine &&
            item.count == 2
        ));
        
        let equipment = classes.iter().find(|c| c.class_name == "Equipment").unwrap();
        assert!(equipment.items.iter().any(|item| 
            item.class_id == "item1"
        ));
    }
}
