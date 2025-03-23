use std::collections::HashSet;
use hemtt_sqm::{Class, Value};

/// Utility functions for working with HEMTT SQM classes
pub(crate) trait ClassExt {
    /// Find classes that match the given predicate
    fn find_classes<F>(&self, predicate: F) -> Vec<&Class>
    where
        F: Fn(&Class) -> bool + Copy;

    /// Extract property value as a string if it exists
    fn get_property_string(&self, name: &str) -> Option<String>;
}

impl ClassExt for Class {
    fn find_classes<F>(&self, predicate: F) -> Vec<&Class>
    where
        F: Fn(&Class) -> bool + Copy,
    {
        let mut results = Vec::new();
        
        // Add self if it matches
        if predicate(self) {
            results.push(self);
        }
        
        // Search in nested classes
        for (_, class_list) in &self.classes {
            for class in class_list {
                results.extend(class.find_classes(predicate));
            }
        }
        
        results
    }
    
    fn get_property_string(&self, name: &str) -> Option<String> {
        self.properties.get(name).and_then(|value| {
            match value {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }
        })
    }
}

/// Utility for collecting dependencies from SQM files
pub(crate) struct DependencyCollector {
    dependencies: HashSet<String>,
}

impl DependencyCollector {
    pub fn new() -> Self {
        Self {
            dependencies: HashSet::new(),
        }
    }
    
    /// Add a dependency string if it's valid
    /// 
    /// Dependencies are invalid if:
    /// - They are empty strings
    /// - They contain a colon (typically used for special commands)
    pub fn add_dependency(&mut self, dependency: String) {
        if !dependency.is_empty() && !dependency.contains(':') {
            self.dependencies.insert(dependency);
        }
    }
    
    /// Consume this collector and return the HashSet of dependencies
    pub fn get_dependencies(self) -> HashSet<String> {
        self.dependencies
    }
}