pub mod models;
mod parser;
mod query;

use std::collections::HashSet;
use parser::parse_sqm_content;
use query::DependencyExtractor;

/// Extract class dependencies from SQM content
/// 
/// This function parses an SQM file and extracts all dependencies including:
/// - Weapons, magazines, and other equipment
/// - Uniforms, vests, backpacks, headgear
/// - Items inside containers
/// - Object types
/// 
/// # Examples
/// 
/// ```
/// use parser_sqm::extract_class_dependencies;
/// use std::collections::HashSet;
/// 
/// let sqm_content = r#"
/// class Mission {
///     class Item1 {
///         class Attributes {
///             class Inventory {
///                 class primaryWeapon {
///                     name = "arifle_MX_F";
///                 };
///                 uniform = "U_B_CombatUniform_mcam";
///             };
///         };
///     };
/// };"#;
/// 
/// let dependencies = extract_class_dependencies(sqm_content);
/// assert!(dependencies.contains("U_B_CombatUniform_mcam"));
/// assert!(dependencies.contains("arifle_MX_F"));
/// ```
pub fn extract_class_dependencies(sqm_content: &str) -> HashSet<String> {
    match parse_sqm_content(sqm_content) {
        Ok(sqm_file) => {
            let extractor = DependencyExtractor::new(&sqm_file);
            extractor.extract_dependencies()
        }
        Err(_) => HashSet::new()
    }
}