//! Core data structures for SQF parsing and analysis

use std::fmt;

/// Represents a class reference found in SQF code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassReference {
    /// The class name/ID
    pub class_name: String,
    /// The context where it was found (scope/conditions)
    pub context: String,
}

/// Represents how a class reference was discovered
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsageContext {
    /// Used in an add* command (addWeapon, addVest, etc.)
    AddCommand(String),
    /// Used in a function known to use class references
    KnownFunction(String),
    /// Directly used as a string in a context that suggests it's a class
    DirectReference,
}

impl fmt::Display for UsageContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsageContext::AddCommand(cmd) => write!(f, "Used in command: {}", cmd),
            UsageContext::KnownFunction(func) => write!(f, "Used in function: {}", func),
            UsageContext::DirectReference => write!(f, "Direct reference"),
        }
    }
}

/// Represents the result of analyzing SQF code
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub references: Vec<ClassReference>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_reference_equality() {
        let ref1 = ClassReference {
            class_name: "test_item".to_string(),
            context: "test_scope".to_string(),
        };
        
        let ref2 = ClassReference {
            class_name: "test_item".to_string(),
            context: "test_scope".to_string(),
        };
        
        let ref3 = ClassReference {
            class_name: "different_item".to_string(),
            context: "test_scope".to_string(),
        };
        
        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn test_usage_context_display() {
        assert_eq!(
            UsageContext::AddCommand("addWeapon".to_string()).to_string(),
            "Used in command: addWeapon"
        );
        assert_eq!(
            UsageContext::KnownFunction("ace_arsenal_fnc_initBox".to_string()).to_string(),
            "Used in function: ace_arsenal_fnc_initBox"
        );
        assert_eq!(
            UsageContext::DirectReference.to_string(),
            "Direct reference"
        );
    }
}