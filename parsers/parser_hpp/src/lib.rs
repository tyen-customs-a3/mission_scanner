use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use hemtt_config::{Config, parse, Property, Class, Value, Array, Item};
use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::{Codes, Processed, Code, Diagnostic, Severity}, LayerType, Workspace, WorkspacePath};
use serde::{Serialize, Deserialize};
use tempfile::NamedTempFile;

mod parser;
pub use parser::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HppClass {
    pub name: String,
    pub parent: Option<String>,
    pub properties: Vec<HppProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HppProperty {
    pub name: String,
    pub value: HppValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HppValue {
    String(String),
    Array(Vec<String>),
    Number(i64),
    Class(HppClass),
}

pub struct HppParser {
    config: Config,
}

/// Parse an HPP file and return a vector of classes.
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the HPP file to parse
/// 
/// # Returns
/// 
/// * `Result<Vec<HppClass>, Codes>` - List of classes found in the file or error
pub fn parse_file(file_path: &std::path::Path) -> Result<Vec<HppClass>, Codes> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|_| vec![])?;
    
    let parser = HppParser::new(&content)?;
    Ok(parser.parse_classes())
}

impl HppParser {
    pub fn new(content: &str) -> Result<Self, Codes> {
        // Create a temporary workspace with the content
        let temp_file = NamedTempFile::new().map_err(|e| vec![])?;
        fs::write(temp_file.path(), content).map_err(|e| vec![])?;
        
        let parent_path = PathBuf::from(temp_file.path().parent().unwrap());
        let workspace = Workspace::builder()
            .physical(&parent_path, LayerType::Source)
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .map_err(|e| vec![])?;
            
        let path = workspace.join(temp_file.path().file_name().unwrap().to_str().unwrap()).map_err(|e| vec![])?;
        let processed = match Processor::run(&path) {
            Ok(processed) => processed,
            Err((_, e)) => {
                // Create a custom error that implements Code
                #[derive(Debug)]
                struct ProcessorError(hemtt_preprocessor::Error);
                impl Code for ProcessorError {
                    fn message(&self) -> String { self.0.to_string() }
                    fn severity(&self) -> Severity { Severity::Error }
                    fn diagnostic(&self) -> Option<Diagnostic> { None }
                    fn ident(&self) -> &'static str { "processor_error" }
                }
                return Err(vec![Arc::new(ProcessorError(e))]);
            }
        };
        let report = parse(None, &processed)?;
        
        Ok(Self {
            config: report.into_config(),
        })
    }

    pub fn parse_classes(&self) -> Vec<HppClass> {
        let mut classes = Vec::new();
        self.extract_classes(&self.config, &mut classes);
        classes
    }

    fn extract_classes(&self, config: &Config, classes: &mut Vec<HppClass>) {
        for property in config.0.iter() {
            if let Property::Class(class) = property {
                if let Class::Local { name, parent, properties, .. } = class {
                    let mut hpp_class = HppClass {
                        name: name.as_str().to_string(),
                        parent: parent.as_ref().map(|p| p.as_str().to_string()),
                        properties: Vec::new(),
                    };

                    // Extract properties from the class
                    for prop in properties {
                        if let Property::Entry { name, value, .. } = prop {
                            hpp_class.properties.push(HppProperty {
                                name: name.as_str().to_string(),
                                value: self.convert_value(value),
                            });
                        }
                    }

                    classes.push(hpp_class);

                    for prop in properties {
                        if let Property::Class(_) = prop {
                            let mut nested_classes = Vec::new();
                            let nested_config = Config(vec![prop.clone()]);
                            self.extract_classes(&nested_config, &mut nested_classes);
                            classes.extend(nested_classes);
                        }
                    }
                }
            }
        }
    }

    fn convert_value(&self, value: &Value) -> HppValue {
        match value {
            Value::Str(s) => HppValue::String(s.value().to_string()),
            Value::Number(n) => {
                match n {
                    hemtt_config::Number::Int32 { value, .. } => HppValue::Number(*value as i64),
                    hemtt_config::Number::Int64 { value, .. } => HppValue::Number(*value),
                    hemtt_config::Number::Float32 { value, .. } => HppValue::Number(*value as i64),
                }
            }
            Value::Array(arr) => {
                let mut values = Vec::new();
                for item in arr.items.iter() {
                    match item {
                        Item::Str(s) => values.push(s.value().to_string()),
                        Item::Number(n) => values.push(n.to_string()),
                        Item::Macro(m) => {
                            let macro_name = m.name.value();
                            
                            if macro_name.starts_with("LIST_") {
                                // Just add the inner item once, don't expand based on count
                                if let Some(first_arg) = m.args.first() {
                                    values.push(first_arg.value().to_string());
                                }
                            } else {
                                // For complex macros with multiple arguments, preserve as a single string
                                let args_str = m.args.iter()
                                    .map(|arg| arg.value().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                
                                if !m.args.is_empty() {
                                    values.push(format!("{}({})", macro_name, args_str));
                                } else {
                                    values.push(macro_name.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                HppValue::Array(values)
            }
            _ => HppValue::String(String::new()), // Default for unhandled types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_class_parsing() {
        let content = r#"
            class BaseMan {
                displayName = "Unarmed";
                uniform[] = {"uniform1", "uniform2"};
                items[] = {"item1", "item2"};
            };
        "#;

        let parser = HppParser::new(content).unwrap();
        let classes = parser.parse_classes();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "BaseMan");
        assert_eq!(classes[0].properties.len(), 3);
    }

    #[test]
    fn test_inheritance() {
        let content = r#"
            class BaseMan {
                displayName = "Base";
            };
            class Rifleman : BaseMan {
                displayName = "Rifleman";
            };
        "#;

        let parser = HppParser::new(content).unwrap();
        let classes = parser.parse_classes();

        assert_eq!(classes.len(), 2);
        assert_eq!(classes[1].parent.as_deref(), Some("BaseMan"));
    }

    #[test]
    fn test_array_with_list_macro() {
        let content = r#"
            class Test {
                uniform[] = {
                    LIST_2("usp_g3c_kp_mx_aor2"),
                    "usp_g3c_rs_kp_mx_aor2",
                    "usp_g3c_rs2_kp_mx_aor2"
                };
            };
        "#;
        let parser = HppParser::new(content).unwrap();
        let classes = parser.parse_classes();
        
        assert_eq!(classes.len(), 1);
        let test_class = &classes[0];
        assert_eq!(test_class.name, "Test");
        
        let uniform_prop = test_class.properties.iter().find(|p| p.name == "uniform").unwrap();
        if let HppValue::Array(uniforms) = &uniform_prop.value {
            // Check that the array contains items with these strings (possibly with quotes)
            assert!(uniforms.iter().any(|u| u.contains("usp_g3c_kp_mx_aor2")), 
                   "Missing 'usp_g3c_kp_mx_aor2'. Found: {:?}", uniforms);
            assert!(uniforms.iter().any(|u| u.contains("usp_g3c_rs_kp_mx_aor2")), 
                   "Missing 'usp_g3c_rs_kp_mx_aor2'. Found: {:?}", uniforms);
            assert!(uniforms.iter().any(|u| u.contains("usp_g3c_rs2_kp_mx_aor2")), 
                   "Missing 'usp_g3c_rs2_kp_mx_aor2'. Found: {:?}", uniforms);
            assert_eq!(uniforms.len(), 3); // Should have 3 items because LIST_2 is not expanded
        } else {
            panic!("Expected uniform to be an array");
        }
    }
} 