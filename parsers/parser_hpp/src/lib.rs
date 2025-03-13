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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HppClass {
    pub name: String,
    pub parent: Option<String>,
    pub properties: Vec<HppProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HppProperty {
    pub name: String,
    pub value: HppValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HppValue {
    String(String),
    Array(Vec<String>),
    Number(i64),
    Class(HppClass),
}

pub struct HppParser {
    config: Config,
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

                    // Recursively extract nested classes
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
                for item in arr.items() {
                    match item {
                        Item::Str(s) => values.push(s.value().to_string()),
                        Item::Macro((macro_name, item, _)) => {
                            // Extract the count from the macro name (e.g., "LIST_2" -> 2)
                            let count = macro_name.value()
                                .strip_prefix("LIST_")
                                .and_then(|n| n.parse::<usize>().ok())
                                .unwrap_or(1);
                            for _ in 0..count {
                                values.push(item.value().to_string());
                            }
                        }
                        Item::Eval { expression, .. } => values.push(expression.value().to_string()),
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
            assert!(uniforms.contains(&"usp_g3c_kp_mx_aor2".to_string()));
            assert!(uniforms.contains(&"usp_g3c_rs_kp_mx_aor2".to_string()));
            assert!(uniforms.contains(&"usp_g3c_rs2_kp_mx_aor2".to_string()));
            assert_eq!(uniforms.len(), 4); // Should have 4 items because LIST_2 expands to 2
        } else {
            panic!("Expected uniform to be an array");
        }
    }
} 