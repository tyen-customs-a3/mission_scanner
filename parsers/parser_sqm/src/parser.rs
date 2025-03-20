use std::path::PathBuf;
use std::fs;
use hemtt_config::{Config, parse, Property, Class, Value};
use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::{Codes, Processed}, LayerType, Workspace};
use tempfile::NamedTempFile;

use crate::models::{Class as SqmClass, Property as SqmProperty, PropertyValue as SqmPropertyValue};

/// Parse an SQM file content and return a Class structure
pub fn parse_sqm(content: &str) -> Result<SqmClass, Codes> {
    // Create a temporary workspace with the content
    let temp_file = NamedTempFile::new().map_err(|_| vec![])?;
    fs::write(temp_file.path(), content).map_err(|_| vec![])?;
    
    let parent_path = PathBuf::from(temp_file.path().parent().unwrap());
    let workspace = Workspace::builder()
        .physical(&parent_path, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .map_err(|_| vec![])?;
        
    let path = workspace.join(temp_file.path().file_name().unwrap().to_str().unwrap()).map_err(|_| vec![])?;
    let processed = match Processor::run(&path) {
        Ok(processed) => processed,
        Err((_, e)) => return Err(vec![]),
    };
    
    let report = parse(None, &processed)?;
    let config = report.into_config();
    
    // Find the Mission class in the config
    let mission_class = config.0.iter()
        .find_map(|prop| {
            if let Property::Class(class) = prop {
                if let Class::Local { name, .. } = class {
                    if name.as_str() == "Mission" {
                        return Some(class);
                    }
                }
            }
            None
        })
        .ok_or_else(|| vec![])?;

    if let Class::Local { name, properties, .. } = mission_class {
        let mut root = SqmClass::new(name.as_str().to_string());
        convert_config_to_sqm(&Config(properties.clone()), &mut root);
        Ok(root)
    } else {
        Err(vec![])
    }
}

fn convert_config_to_sqm(config: &Config, sqm_class: &mut SqmClass) {
    for property in config.0.iter() {
        match property {
            Property::Class(class) => {
                if let Class::Local { name, properties, .. } = class {
                    let mut nested_class = SqmClass::new(name.as_str().to_string());
                    convert_config_to_sqm(&Config(properties.clone()), &mut nested_class);
                    sqm_class.nested_classes.push(nested_class);
                }
            }
            Property::Entry { name, value, .. } => {
                sqm_class.properties.insert(
                    name.as_str().to_string(),
                    SqmProperty {
                        name: name.as_str().to_string(),
                        value: convert_value(value),
                    },
                );
            }
            _ => {}
        }
    }
}

fn convert_value(value: &Value) -> SqmPropertyValue {
    match value {
        Value::Str(s) => SqmPropertyValue::String(s.value().to_string()),
        Value::Number(n) => {
            match n {
                hemtt_config::Number::Int32 { value, .. } => SqmPropertyValue::Number(*value as f64),
                hemtt_config::Number::Int64 { value, .. } => SqmPropertyValue::Number(*value as f64),
                hemtt_config::Number::Float32 { value, .. } => SqmPropertyValue::Number(*value as f64),
            }
        }
        Value::Array(arr) => {
            let values = arr.items.iter()
                .filter_map(|item| {
                    match item {
                        hemtt_config::Item::Str(s) => Some(s.value().to_string()),
                        hemtt_config::Item::Number(n) => Some(n.to_string()),
                        _ => None,
                    }
                })
                .collect();
            SqmPropertyValue::Array(values)
        }
        _ => SqmPropertyValue::String(String::new()), // Default for unhandled types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PropertyValue;

    #[test]
    fn test_parse_basic_sqm() {
        let content = r#"
            class Mission {
                class Intel {
                    briefingName = "Test Mission";
                    startWeather = 0.3;
                    startWind = 0.1;
                };
            };
        "#;

        let result = parse_sqm(content).unwrap();
        assert_eq!(result.name, "Mission");
        
        // Find Intel class
        let intel = result.nested_classes.iter()
            .find(|c| c.name == "Intel")
            .expect("Intel class not found");
            
        // Check properties
        assert!(intel.properties.contains_key("briefingName"));
        assert!(intel.properties.contains_key("startWeather"));
        assert!(intel.properties.contains_key("startWind"));
        
        // Check values
        if let PropertyValue::String(name) = &intel.properties["briefingName"].value {
            assert_eq!(name, "Test Mission");
        } else {
            panic!("briefingName should be a string");
        }
    }

    #[test]
    fn test_parse_arrays() {
        let content = r#"
            class Mission {
                class Item0 {
                    position[] = {1234.5, 67.8, 90.1};
                    items[] = {"item1", "item2", "item3"};
                };
            };
        "#;

        let result = parse_sqm(content).unwrap();
        let item0 = result.nested_classes.iter()
            .find(|c| c.name == "Item0")
            .expect("Item0 class not found");
            
        // Check position array
        if let PropertyValue::Array(pos) = &item0.properties["position"].value {
            assert_eq!(pos.len(), 3);
            assert!(pos.contains(&"1234.5".to_string()));
            assert!(pos.contains(&"67.8".to_string()));
            assert!(pos.contains(&"90.1".to_string()));
        } else {
            panic!("position should be an array");
        }
        
        // Check items array
        if let PropertyValue::Array(items) = &item0.properties["items"].value {
            assert_eq!(items.len(), 3);
            assert!(items.contains(&"item1".to_string()));
            assert!(items.contains(&"item2".to_string()));
            assert!(items.contains(&"item3".to_string()));
        } else {
            panic!("items should be an array");
        }
    }
} 