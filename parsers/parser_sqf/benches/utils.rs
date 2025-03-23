use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn generate_simple_assignments(count: usize) -> String {
    let mut code = String::new();
    for i in 0..count {
        code.push_str(&format!(
            r#"_unit addWeapon "weapon_{i}";
               _unit addMagazine "magazine_{i}";
               _unit addBackpack "backpack_{i}";
               _unit addVest "vest_{i}";
               _unit addUniform "uniform_{i}";
               _unit addItem "item_{i}";"#
        ));
    }
    code
}

pub fn generate_nested_arrays(depth: usize, width: usize) -> String {
    let mut code = String::new();
    code.push_str("private _equipment = [");
    
    fn generate_array(depth: usize, width: usize, prefix: &str) -> String {
        if depth == 0 {
            return (0..width)
                .map(|i| format!("\"{}_{i}\"", prefix))
                .collect::<Vec<_>>()
                .join(", ");
        }
        
        (0..width)
            .map(|i| {
                format!("[{}]",
                    generate_array(depth - 1, width, &format!("{prefix}_{i}"))
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    code.push_str(&generate_array(depth, width, "item"));
    code.push_str("];\n[_equipment] call ace_arsenal_fnc_initBox;");
    code
}

pub fn generate_arsenal_init(item_count: usize) -> String {
    let mut code = String::new();
    code.push_str("private _weapons = [");
    for i in 0..item_count {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("\"weapon_{i}\""));
    }
    code.push_str("];\n");
    
    code.push_str("private _magazines = [");
    for i in 0..item_count {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("\"magazine_{i}\""));
    }
    code.push_str("];\n");
    
    code.push_str("private _items = [");
    for i in 0..item_count {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("\"item_{i}\""));
    }
    code.push_str("];\n");
    
    code.push_str("[_weapons + _magazines + _items] call ace_arsenal_fnc_initBox;");
    code
}

pub fn generate_conditional_assignments(depth: usize) -> String {
    fn generate_nested_if(depth: usize, index: usize) -> String {
        if depth == 0 {
            return format!(
                r#"_unit addWeapon "weapon_{index}";
                   _unit addMagazine "magazine_{index}";"#
            );
        }
        
        format!(
            r#"if (_role == "role_{index}") then {{
                {}
            }} else {{
                {}
            }};"#,
            generate_nested_if(depth - 1, index * 2),
            generate_nested_if(depth - 1, index * 2 + 1)
        )
    }
    
    generate_nested_if(depth, 0)
}

pub fn generate_variable_tracking(var_count: usize, ops_per_var: usize) -> String {
    let mut code = String::new();
    
    // Initialize variables
    for i in 0..var_count {
        code.push_str(&format!("private _var_{i} = [];\n"));
    }
    
    // Perform operations
    for i in 0..var_count {
        for j in 0..ops_per_var {
            code.push_str(&format!(
                r#"_var_{i} pushBack "item_{i}_{j}";
                   _var_{i} = _var_{i} + ["extra_{i}_{j}"];"#
            ));
        }
    }
    
    // Use variables in arsenal
    code.push_str("\nprivate _allItems = [];\n");
    for i in 0..var_count {
        code.push_str(&format!("_allItems append _var_{i};\n"));
    }
    code.push_str("[_allItems] call ace_arsenal_fnc_initBox;");
    
    code
}

pub fn write_test_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

pub fn copy_benchmark_file_small(dir: &TempDir) -> PathBuf {
    let benchmark_file_small_content = include_str!("benchmark_file_small.sqf");
    write_test_file(dir, "benchmark_file_small.sqf", benchmark_file_small_content)
}

pub fn copy_benchmark_file_large(dir: &TempDir) -> PathBuf {
    let benchmark_file_large = include_str!("benchmark_file_large.sqf");
    write_test_file(dir, "benchmark_file_large.sqf", benchmark_file_large)
} 