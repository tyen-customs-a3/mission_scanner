use hemtt_sqf::Expression;
use std::collections::HashMap;
use crate::models::UsageContext;
use super::evaluator::SqfValue;

/// Handles array operations and value extraction
pub struct ArrayHandler {
    /// Callback for when a class reference is found
    reference_callback: Box<dyn Fn(String, UsageContext) + Send>,
}

impl ArrayHandler {
    /// Create a new array handler with a callback for class references
    pub fn new<F>(callback: F) -> Self 
    where
        F: Fn(String, UsageContext) + Send + 'static
    {
        Self {
            reference_callback: Box::new(callback),
        }
    }

    /// Handle array operations like pushBack and array concatenation
    pub fn handle_array_operation(
        &self,
        operation: &str,
        lhs: &Expression,
        rhs: &Expression,
        variables: &HashMap<String, SqfValue>,
        context: UsageContext,
    ) -> Option<SqfValue> {
        match operation.to_lowercase().as_str() {
            "+" => self.handle_array_concat(lhs, rhs, variables),
            "pushback" | "pushbackunique" => self.handle_push_back(lhs, rhs, variables, context, operation),
            _ => None
        }
    }

    /// Handle array concatenation operation
    fn handle_array_concat(
        &self,
        lhs: &Expression,
        rhs: &Expression,
        variables: &HashMap<String, SqfValue>,
    ) -> Option<SqfValue> {
        let lhs_value = self.evaluate_expression_to_value(lhs, variables);
        let rhs_value = self.evaluate_expression_to_value(rhs, variables);

        match (lhs_value, rhs_value) {
            (SqfValue::Array(mut left), SqfValue::Array(right)) => {
                left.extend(right);
                Some(SqfValue::Array(left))
            },
            (SqfValue::Array(mut arr), other) => {
                arr.push(other);
                Some(SqfValue::Array(arr))
            },
            (other, SqfValue::Array(mut arr)) => {
                arr.insert(0, other);
                Some(SqfValue::Array(arr))
            },
            (lhs, rhs) => Some(SqfValue::Array(vec![lhs, rhs]))
        }
    }

    /// Handle pushBack and pushBackUnique operations
    fn handle_push_back(
        &self,
        lhs: &Expression,
        rhs: &Expression,
        variables: &HashMap<String, SqfValue>,
        context: UsageContext,
        operation: &str,
    ) -> Option<SqfValue> {
        let mut array = match self.evaluate_expression_to_value(lhs, variables) {
            SqfValue::Array(arr) => arr,
            _ => Vec::new()
        };

        let value = self.evaluate_expression_to_value(rhs, variables);
        
        // For strings, add them as references
        if let SqfValue::String(s) = &value {
            (self.reference_callback)(s.clone(), context);
        }

        // For pushBackUnique, only add if not already present
        let is_unique = operation.to_lowercase() == "pushbackunique";
        if !is_unique || !array.contains(&value) {
            array.push(value);
        }

        Some(SqfValue::Array(array))
    }

    /// Evaluate an expression to an SqfValue
    pub fn evaluate_expression_to_value(
        &self,
        expr: &Expression,
        variables: &HashMap<String, SqfValue>
    ) -> SqfValue {
        match expr {
            Expression::String(s, _, _) => SqfValue::String(s.to_string()),
            Expression::Array(elements, _) => {
                let values: Vec<_> = elements.iter()
                    .map(|e| self.evaluate_expression_to_value(e, variables))
                    .collect();
                SqfValue::Array(values)
            },
            Expression::Variable(name, _) => {
                variables.get(name).cloned().unwrap_or(SqfValue::Unknown)
            },
            _ => SqfValue::Unknown
        }
    }

    /// Extract array values from an expression
    pub fn extract_array_values(
        &self,
        expr: &Expression,
        variables: &HashMap<String, SqfValue>,
        result: &mut Vec<String>
    ) {
        match self.evaluate_expression_to_value(expr, variables) {
            SqfValue::String(s) => result.push(s),
            SqfValue::Array(values) => {
                for value in values {
                    if let SqfValue::String(s) = value {
                        result.push(s);
                    }
                }
            },
            _ => {}
        }
    }
} 