use std::collections::{HashSet, HashMap};
use crate::ast::SqfExpr;
use crate::parser;
use chumsky::Parser;

#[derive(Debug)]
pub struct ItemReference {
    pub item_id: String,
    pub context: String,
}

// Track both the value and whether it's an item
#[derive(Debug, Clone)]
struct VarInfo {
    value: String,
    is_item: bool,
    // Track which functions use this variable
    used_by: Vec<String>,
    // Track which variables this value came from
    source_vars: Vec<String>,
}

pub fn scan_for_items(exprs: &[SqfExpr]) -> Vec<ItemReference> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    let mut var_info = HashMap::new();

    fn is_item_context(name: &str) -> bool {
        let item_related_functions = [
            // Add/remove items
            "additem", "additems", "removeitem", "removeitems",
            // Weapons
            "addweapon", "addweapons", "removeweapon", "removeweapons",
            "addprimaryweaponitem", "addsecondaryweaponitem", "addhandgunitem",
            "addweaponitem",
            // Magazines/ammo
            "addmagazine", "addmagazines", "removemagazine", "removemagazines",
            // Equipment
            "addbackpack", "addbackpacks", "removebackpack",
            "addgoggles", "removegoggles",
            "addheadgear", "removeheadgear",
            "adduniform", "removeuniform", "forceadduniform",
            "addvest", "removevest",
            // Container operations
            "additemtobackpack", "additemtovest", "additemtouniform",
            // Special functions
            "linkitem", "unlinkitem",
            "ace_arsenal_fnc_initbox", "bis_fnc_addvirtualitemcargo",
            "bis_fnc_addvirtualweaponcargo", "bis_fnc_addvirtualmagazinecargo",
            "bis_fnc_addvirtualbackpackcargo",
            // Selection functions that might contain items
            "selectrandom", "selectrandomweighted",
            // Cargo functions
            "ace_cargo_fnc_loaditem",
            // Vehicle spawn functions that create items
            "createvehicle", "bis_fnc_spawnvehicle"
        ];

        let name = name.to_lowercase();
        for func in item_related_functions {
            if name.contains(func) {
                eprintln!("[DEBUG] Found item-related function: {}", name);
                return true;
            }
        }
        false
    }

    fn is_item_variable(name: &str) -> bool {
        let item_related_names = [
            "item", "weapon", "uniform", "vest", "backpack", 
            "gear", "magazine", "ammo", "optic", "attachment",
            "goggles", "headgear", "nvg", "bp", "mat"  // Common variable patterns
        ];

        let name = name.to_lowercase();
        let is_item = item_related_names.iter().any(|&pattern| name.contains(pattern));
        if is_item {
            eprintln!("[DEBUG] Found item-related variable: {}", name);
        }
        is_item
    }

    fn add_item_reference(items: &mut Vec<ItemReference>, seen: &mut HashSet<String>, item_id: String, context: &str) {
        if !item_id.is_empty() && !seen.contains(&item_id) {
            seen.insert(item_id.clone());
            eprintln!("[DEBUG] Adding item reference: {} (context: {})", item_id, context);
            items.push(ItemReference {
                item_id,
                context: context.to_string(),
            });
        }
    }

    fn scan_expr(expr: &SqfExpr, items: &mut Vec<ItemReference>, seen: &mut HashSet<String>, var_info: &mut HashMap<String, VarInfo>, context: &str) {
        eprintln!("[DEBUG] Scanning expression: {:?} with context: {}", expr, context);
        match expr {
            SqfExpr::String(s) if !s.is_empty() => {
                if !context.is_empty() {
                    add_item_reference(items, seen, s.clone(), context);
                }
            }
            SqfExpr::Array(elements) => {
                let mut is_weighted_array = false;
                if context.to_lowercase().contains("selectrandomweighted") && elements.len() % 2 == 0 {
                    is_weighted_array = true;
                }

                if context.is_empty() {
                    for element in elements {
                        scan_expr(element, items, seen, var_info, "");
                    }
                } else {
                    // For selectRandomWeighted arrays, we want to scan only the items (even indices)
                    // and use the parent context to determine their type
                    for (i, element) in elements.iter().enumerate() {
                        if !is_weighted_array || i % 2 == 0 {
                            scan_expr(element, items, seen, var_info, context);
                        }
                    }
                }
            }
            SqfExpr::Block(exprs) => {
                for expr in exprs {
                    scan_expr(expr, items, seen, var_info, context);
                }
            }
            SqfExpr::Assignment { name, value } | SqfExpr::ForceAssignment { name, value } => {
                eprintln!("[DEBUG] Processing assignment: {} = {:?}", name, value);
                match &**value {
                    SqfExpr::String(s) => {
                        let is_item = is_item_variable(name) || !context.is_empty();
                        var_info.insert(name.clone(), VarInfo {
                            value: s.clone(),
                            is_item,
                            used_by: Vec::new(),
                            source_vars: Vec::new(),
                        });
                        eprintln!("[DEBUG] Stored variable info: {} = {} (is_item: {})", name, s, is_item);
                        
                        if is_item {
                            add_item_reference(items, seen, s.clone(), "assignment");
                        }
                    }
                    SqfExpr::Array(elements) => {
                        // For arrays, we need to track each string element as a potential item
                        let mut array_items = Vec::new();
                        for element in elements {
                            if let SqfExpr::String(s) = element {
                                array_items.push(s.clone());
                                // Add each array element as a potential item
                                add_item_reference(items, seen, s.clone(), "array_assignment");
                            }
                        }
                        if !array_items.is_empty() {
                            var_info.insert(name.clone(), VarInfo {
                                value: array_items.join(", "),
                                is_item: true, // Arrays of strings are likely items
                                used_by: Vec::new(),
                                source_vars: Vec::new(),
                            });
                        }
                    }
                    SqfExpr::Variable(source_var) => {
                        // Track the source variable
                        if let Some(source_info) = var_info.get(source_var) {
                            let source_info = source_info.clone();
                            var_info.insert(name.clone(), VarInfo {
                                value: source_info.value.clone(),
                                is_item: source_info.is_item,
                                used_by: Vec::new(),
                                source_vars: {
                                    let mut sources = vec![source_var.clone()];
                                    sources.extend(source_info.source_vars.iter().cloned());
                                    sources
                                },
                            });
                            if source_info.is_item {
                                add_item_reference(items, seen, source_info.value.clone(), "assignment");
                            }
                        }
                    }
                    SqfExpr::FunctionCall { name, args } => {
                        // Track function call results assigned to variables
                        if name == "selectRandomWeighted" {
                            if let Some(SqfExpr::Array(array_args)) = args.first() {
                                for (i, arg) in array_args.iter().enumerate() {
                                    if i % 2 == 0 {  // Only process items, skip weights
                                        if let SqfExpr::String(s) = arg {
                                            var_info.insert(name.clone(), VarInfo {
                                                value: s.clone(),
                                                is_item: true,
                                                used_by: Vec::new(),
                                                source_vars: Vec::new(),
                                            });
                                            add_item_reference(items, seen, s.clone(), "selectRandomWeighted");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                scan_expr(value, items, seen, var_info, "");
            }
            SqfExpr::FunctionCall { name, args } => {
                let is_item_func = is_item_context(&name);
                eprintln!("[DEBUG] Processing function call: {} (is_item_func: {})", name, is_item_func);
                
                let new_context = if is_item_func {
                    name.clone()
                } else {
                    context.to_string()
                };

                // For direct function calls (e.g. addHeadgear "rhs_tsh4")
                // the item is usually the last argument
                if is_item_func && !args.is_empty() {
                    match args.last().unwrap() {
                        SqfExpr::String(s) => {
                            add_item_reference(items, seen, s.clone(), &new_context);
                        }
                        SqfExpr::Variable(var_name) => {
                            eprintln!("[DEBUG] Processing variable in function call: {}", var_name);
                            // Track that this variable is used by this function
                            if let Some(info) = var_info.get_mut(var_name) {
                                info.used_by.push(name.clone());
                                eprintln!("[DEBUG] Found variable info: {:?}", info);
                                if info.is_item {
                                    add_item_reference(items, seen, info.value.clone(), &new_context);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Still scan all arguments for nested items
                for arg in args {
                    match arg {
                        SqfExpr::Variable(var_name) => {
                            eprintln!("[DEBUG] Processing variable in function call: {}", var_name);
                            // Track that this variable is used by this function
                            if let Some(info) = var_info.get_mut(var_name) {
                                info.used_by.push(name.clone());
                                eprintln!("[DEBUG] Found variable info: {:?}", info);
                                if info.is_item {
                                    add_item_reference(items, seen, info.value.clone(), &new_context);
                                }
                            }
                        }
                        _ => scan_expr(arg, items, seen, var_info, &new_context),
                    }
                }
            }
            SqfExpr::BinaryOp { left, right, .. } => {
                scan_expr(left, items, seen, var_info, context);
                scan_expr(right, items, seen, var_info, context);
            }
            SqfExpr::ArrayAccess { array, index } => {
                scan_expr(array, items, seen, var_info, context);
                scan_expr(index, items, seen, var_info, context);
            }
            SqfExpr::ForEach { body, array } => {
                // First scan the array to get its items
                scan_expr(array, items, seen, var_info, context);
                
                // When we see a forEach loop, we know that _x will contain each item from the array
                match &**array {
                    SqfExpr::Variable(array_name) => {
                        if let Some(array_info) = var_info.get(array_name) {
                            if array_info.is_item {
                                // Split the array value into individual items
                                for item in array_info.value.split(", ") {
                                    add_item_reference(items, seen, item.to_string(), "forEach");
                                }
                                var_info.insert("_x".to_string(), VarInfo {
                                    value: array_info.value.clone(),
                                    is_item: true,
                                    used_by: Vec::new(),
                                    source_vars: vec![array_name.clone()],
                                });
                            }
                        }
                    }
                    SqfExpr::Array(elements) => {
                        // If we have a direct array, each element becomes a potential value for _x
                        for element in elements {
                            if let SqfExpr::String(s) = element {
                                add_item_reference(items, seen, s.clone(), "forEach");
                                var_info.insert("_x".to_string(), VarInfo {
                                    value: s.clone(),
                                    is_item: true,
                                    used_by: Vec::new(),
                                    source_vars: Vec::new(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
                
                // Now scan the body with the updated variable info
                scan_expr(body, items, seen, var_info, context);
            }
            _ => {}
        }
    }

    // First pass: collect all variable assignments and their initial values
    for expr in exprs.iter() {
        scan_expr(expr, &mut items, &mut seen, &mut var_info, "");
    }

    items
}

// For single expression convenience
pub fn scan_single(expr: &SqfExpr) -> Vec<ItemReference> {
    scan_for_items(std::slice::from_ref(expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use chumsky::Parser;

    #[test]
    fn test_variable_tracking() {
        // Create a sequence of expressions
        let exprs = vec![
            SqfExpr::Assignment {
                name: "_bp".to_string(),
                value: Box::new(SqfExpr::String("rhs_rpg_empty".to_string())),
            },
            SqfExpr::FunctionCall {
                name: "addBackpack".to_string(),
                args: vec![SqfExpr::Variable("_bp".to_string())],
            }
        ];

        let items = scan_for_items(&exprs);
        
        // We should find the item once
        assert!(items.iter().any(|item| 
            item.item_id == "rhs_rpg_empty"
        ));
    }

    #[test]
    fn test_direct_item_assignment() {
        let expr = SqfExpr::FunctionCall {
            name: "addHeadgear".to_string(),
            args: vec![SqfExpr::String("rhs_tsh4".to_string())],
        };

        let items = scan_single(&expr);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_id, "rhs_tsh4");
    }

    #[test]
    fn test_weighted_selection() {
        let input = r#"
        private _facewearPoolWeighted = selectRandomWeighted [
            "goggles1", 4,
            "goggles2", 1
        ];
        _unit addGoggles _facewearPoolWeighted;
    "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        // Check that we found both goggles
        assert!(all_items.iter().any(|item| item.item_id == "goggles1"));
        assert!(all_items.iter().any(|item| item.item_id == "goggles2"));
    }

    #[test]
    fn test_for_loop_basic() {
        let input = r#"
            private _matAT = "rhs_rpg7_PG7VL_mag";
            for "_i" from 1 to 5 do {
                _unit addItemToBackpack _matAT;
            }
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
    }

    #[test]
    fn test_for_loop_with_random() {
        let input = r#"
            private _matAT = "rhs_rpg7_PG7VL_mag";
            for "_i" from 1 to (random 5) do {
                _unit addItemToBackpack _matAT;
            }
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
    }

    #[test]
    fn test_for_loop_with_random_array() {
        let input = r#"
            private _matAT = "rhs_rpg7_PG7VL_mag";
            for "_i" from 1 to (random [2,3,5]) do {
                _unit addItemToBackpack _matAT;
            }
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
    }

    #[test]
    fn test_for_loop_with_ceil_random() {
        let input = r#"
            private _matAT = "rhs_rpg7_PG7VL_mag";
            for "_i" from 1 to (ceil (random 5)) do {
                _unit addItemToBackpack _matAT;
            }
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
    }

    #[test]
    fn test_for_loop_with_ceil_random_array() {
        let input = r#"
            private _matAT = "rhs_rpg7_PG7VL_mag";
            for "_i" from 1 to (ceil (random [2,3,5])) do {
                _unit addItemToBackpack _matAT;
            }
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        assert!(all_items.iter().any(|item| item.item_id == "rhs_rpg7_PG7VL_mag"));
    }


    #[test]
    fn test_multiple_statements() {
        let input = r#"
            private _bp = "rhs_rpg_empty";
            private _mat = "rhs_weap_rpg7";
            private _matAT = "rhs_rpg7_PG7VL_mag";

            _unit addBackpack _bp;
            _unit addWeapon _mat;
            _unit addWeaponItem [_mat, _matAT, true];
            for "_i" from 1 to (ceil (random [2,3,5])) do {
                _unit addItemToBackpack _matAT;
            };

            _unit addHeadgear "rhs_tsh4";

            private _facewearPoolWeighted = selectRandomWeighted [
                "goggles1", 4,
                "goggles2", 1
            ];
            _unit addGoggles _facewearPoolWeighted;
        "#;

        let ast = parser::parser().parse(input).unwrap();
        let mut all_items = Vec::new();
        for expr in &ast.expressions {
            all_items.extend(scan_single(expr));
        }
        
        // Check that we found all items
        let expected_items = vec![
            "rhs_rpg_empty",
            "rhs_weap_rpg7",
            "rhs_rpg7_PG7VL_mag",
            "rhs_tsh4",
            "goggles1",
            "goggles2",
        ];

        for id in expected_items {
            assert!(
                all_items.iter().any(|item| item.item_id == id),
                "Missing item: {}", id
            );
        }
    }
} 