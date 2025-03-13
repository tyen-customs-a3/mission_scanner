use chumsky::prelude::*;
use hemtt_config::{Config, Property, Class};
use hemtt_workspace::reporting::Processed;

/// Helper functions for parsing specific HPP file patterns
pub trait HppPatternParser {
    fn find_class_by_name(&self, name: &str) -> Option<&Class>;
    fn find_property_by_name(&self, name: &str) -> Option<&Property>;
}

impl HppPatternParser for Config {
    fn find_class_by_name(&self, name: &str) -> Option<&Class> {
        self.0.iter().find_map(|prop| {
            if let Property::Class(class) = prop {
                if let Class::Local { name: class_name, .. } = class {
                    if class_name.as_str() == name {
                        return Some(class);
                    }
                }
            }
            None
        })
    }

    fn find_property_by_name(&self, name: &str) -> Option<&Property> {
        self.0.iter().find(|prop| {
            prop.name().as_str() == name
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemtt_config::parse;
    use hemtt_preprocessor::Processor;
    use hemtt_workspace::{LayerType, Workspace};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::fs;

    fn process_content(content: &str) -> Config {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();
        
        let parent_path = PathBuf::from(temp_file.path().parent().unwrap());
        let workspace = Workspace::builder()
            .physical(&parent_path, LayerType::Source)
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();
            
        let path = workspace.join(temp_file.path().file_name().unwrap().to_str().unwrap()).unwrap();
        let processed = Processor::run(&path).unwrap();
        parse(None, &processed).unwrap().into_config()
    }

    #[test]
    fn test_find_class() {
        let content = r#"
            class BaseMan {
                displayName = "Base";
            };
        "#;
        
        let config = process_content(content);
        
        assert!(config.find_class_by_name("BaseMan").is_some());
        assert!(config.find_class_by_name("NonExistent").is_none());
    }

    #[test]
    fn test_find_property() {
        let content = r#"
            myProperty = "test";
            class BaseMan {
                displayName = "Base";
            };
        "#;
        
        let config = process_content(content);
        
        assert!(config.find_property_by_name("myProperty").is_some());
        assert!(config.find_property_by_name("nonExistent").is_none());
    }
} 