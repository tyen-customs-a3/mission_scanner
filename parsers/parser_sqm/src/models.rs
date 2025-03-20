use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Array(Vec<String>),
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: String,
    pub properties: HashMap<String, Property>,
    pub nested_classes: Vec<Class>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: HashMap::new(),
            nested_classes: Vec::new(),
        }
    }

    pub fn find_classes<'a>(&'a self, predicate: &dyn Fn(&Class) -> bool) -> Vec<&'a Class> {
        let mut results = Vec::new();
        if predicate(self) {
            results.push(self);
        }
        for nested in &self.nested_classes {
            results.extend(nested.find_classes(predicate));
        }
        results
    }

    pub fn find_properties<'a>(&'a self, predicate: &dyn Fn(&Property) -> bool) -> Vec<&'a Property> {
        let mut results = Vec::new();
        for property in self.properties.values() {
            if predicate(property) {
                results.push(property);
            }
        }
        for nested in &self.nested_classes {
            results.extend(nested.find_properties(predicate));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_class_creation_and_modification() {
        let mut class = Class::new("TestClass".to_string());
        
        // Test initial state
        assert_eq!(class.name, "TestClass");
        assert!(class.properties.is_empty());
        assert!(class.nested_classes.is_empty());

        // Add a property
        let property = Property {
            name: "test_prop".to_string(),
            value: PropertyValue::String("value".to_string()),
        };
        class.properties.insert(property.name.clone(), property);
        assert_eq!(class.properties.len(), 1);

        // Add a nested class
        let nested = Class::new("NestedClass".to_string());
        class.nested_classes.push(nested);
        assert_eq!(class.nested_classes.len(), 1);
    }

    #[test]
    fn test_find_classes() {
        let mut root = Class::new("Root".to_string());
        let mut nested1 = Class::new("Target".to_string());
        let nested2 = Class::new("Target".to_string());
        let other = Class::new("Other".to_string());
        
        nested1.nested_classes.push(nested2);
        root.nested_classes.push(nested1);
        root.nested_classes.push(other);

        let results = root.find_classes(&|class| class.name == "Target");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|c| c.name == "Target"));
    }

    #[test]
    fn test_find_properties() {
        let mut root = Class::new("Root".to_string());
        
        // Add properties to root
        let properties = vec![
            ("name", PropertyValue::String("test".to_string())),
            ("items", PropertyValue::Array(vec!["item1".to_string(), "item2".to_string()])),
            ("count", PropertyValue::Number(5.0)),
        ];

        for (name, value) in properties {
            root.properties.insert(name.to_string(), Property {
                name: name.to_string(),
                value,
            });
        }

        // Add nested class with properties
        let mut nested = Class::new("Nested".to_string());
        nested.properties.insert("name".to_string(), Property {
            name: "name".to_string(),
            value: PropertyValue::String("nested_value".to_string()),
        });

        root.nested_classes.push(nested);

        // Find all properties named "name"
        let results = root.find_properties(&|prop| prop.name == "name");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|p| p.name == "name"));

        // Find all string properties
        let results = root.find_properties(&|prop| matches!(prop.value, PropertyValue::String(_)));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_sqm_file_as_class() {
        // Create an SQM file as a root class
        let mut sqm = Class::new("Mission".to_string());
        
        // Add top-level properties
        sqm.properties.insert("version".to_string(), Property {
            name: "version".to_string(),
            value: PropertyValue::Number(54.0),
        });
        
        // Add nested classes
        let mut config = Class::new("Config".to_string());
        let mut inventory = Class::new("Inventory".to_string());
        
        inventory.properties.insert("weapon".to_string(), Property {
            name: "weapon".to_string(),
            value: PropertyValue::String("rifle".to_string()),
        });
        
        config.nested_classes.push(inventory);
        sqm.nested_classes.push(config);

        // Test finding classes
        let inventory_classes = sqm.find_classes(&|class| class.name == "Inventory");
        assert_eq!(inventory_classes.len(), 1);
        
        // Test finding properties at root level
        let version_props = sqm.find_properties(&|prop| prop.name == "version");
        assert_eq!(version_props.len(), 1);
        assert!(matches!(&version_props[0].value, PropertyValue::Number(54.0)));
        
        // Test finding nested properties
        let weapon_props = sqm.find_properties(&|prop| prop.name == "weapon");
        assert_eq!(weapon_props.len(), 1);
        assert!(matches!(&weapon_props[0].value, PropertyValue::String(s) if s == "rifle"));
    }
} 