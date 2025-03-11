use chumsky::prelude::*;
use std::collections::HashMap;
use crate::ast::{SqfExpr, SqfFile};

pub fn parser() -> impl Parser<char, SqfFile, Error = Simple<char>> {
    let ident = text::ident()
        .padded();

    let number = text::int(10)
        .then(just('.').then(text::int(10)).or_not())
        .map(|(whole, decimal)| {
            if let Some((_, decimal)) = decimal {
                format!("{}.{}", whole, decimal)
            } else {
                whole
            }
        })
        .map(|s: String| s.parse::<f64>().unwrap())
        .map(SqfExpr::Number)
        .padded();

    let string = just('"')
        .ignore_then(none_of("\"").repeated())
        .then_ignore(just('"'))
        .map(|chars| chars.into_iter().collect::<String>())
        .map(SqfExpr::String)
        .padded();

    let comment = just("//")
        .ignore_then(none_of("\n").repeated())
        .collect::<String>()
        .map(SqfExpr::Comment)
        .padded()
        .or(just("/*")
            .ignore_then(none_of("*/").repeated())
            .then_ignore(just("*/"))
            .collect::<String>()
            .map(SqfExpr::Comment)
            .padded());

    recursive(|expr| {
        let array = expr.clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .delimited_by(just('['), just(']'))
            .map(SqfExpr::Array)
            .padded();

        let block = expr.clone()
            .repeated()
            .delimited_by(just('{'), just('}'))
            .map(SqfExpr::Block)
            .padded();

        let atom = number
            .or(string.clone())
            .or(array.clone())
            .or(ident.map(|s: String| {
                if s.starts_with('_') {
                    SqfExpr::Variable(s)
                } else {
                    match s.as_str() {
                        "true" => SqfExpr::Bool(true),
                        "false" => SqfExpr::Bool(false),
                        "nil" => SqfExpr::Null,
                        _ => SqfExpr::Variable(s),
                    }
                }
            }))
            .or(expr.clone().delimited_by(just('('), just(')')));

        // Handle array access separately from binary operations
        let array_access = atom.clone()
            .then(
                just('#')
                .padded()
                .ignore_then(atom.clone())
                .repeated()
            )
            .map(|(first, indices)| {
                indices.into_iter().fold(first, |array, index| {
                    SqfExpr::ArrayAccess {
                        array: Box::new(array),
                        index: Box::new(index),
                    }
                })
            });

        let function_call = choice((
            // Call operator function call (e.g., [_weapons, _ammo] call fnc_loadout)
            array_access.clone().then(just("call").padded()).then(ident)
                .map(|((params, _), name)| {
                    match params {
                        SqfExpr::Array(args) => SqfExpr::FunctionCall { name, args },
                        other => SqfExpr::FunctionCall { 
                            name, 
                            args: vec![other] 
                        }
                    }
                }),
            // Direct function call with array parameter (e.g., random [2, 3, 5])
            ident.then(array.clone())
                .map(|(name, args)| {
                    if let SqfExpr::Array(args) = args {
                        if name == "selectRandomWeighted" {
                            // Keep the array as a single argument for weighted selection
                            SqfExpr::FunctionCall {
                                name,
                                args: vec![SqfExpr::Array(args)],
                            }
                        } else {
                            // For other functions, spread the array as arguments
                            SqfExpr::FunctionCall {
                                name,
                                args,
                            }
                        }
                    } else {
                        unreachable!()
                    }
                }),
            // Function call with multiple arguments (e.g., _unit addHeadgear "rhs_tsh4")
            array_access.clone().then(ident).then(array_access.clone().repeated())
                .map(|((first_arg, name), rest_args)| {
                    let mut args = vec![first_arg];
                    args.extend(rest_args);
                    SqfExpr::FunctionCall {
                        name,
                        args,
                    }
                }),
            // Direct function call with parentheses (e.g., ceil(random [2, 3, 5]))
            ident.then(expr.clone().delimited_by(just('('), just(')')))
                .map(|(name, arg)| SqfExpr::FunctionCall {
                    name,
                    args: vec![arg],
                })
        ));

        let op = choice((
            just("+").padded(),
            just("-").padded(),
            just("*").padded(),
            just("/").padded(),
        ));

        let binary = array_access.clone()
            .then(op.then(array_access.clone()).repeated())
            .foldl(|left, (op, right)| {
                SqfExpr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            });

        let assignment = choice((
            // Double force assignment (e.g., force force ace_arsenal_enableIdentityTabs = false)
            just("force").padded()
                .ignore_then(just("force").padded())
                .ignore_then(ident)
                .then_ignore(just('=').padded())
                .then(expr.clone())
                .map(|(name, value)| SqfExpr::ForceAssignment {
                    name,
                    value: Box::new(value),
                }),
            // Single force assignment (e.g., force ace_medical_blood_enabledFor = 1)
            just("force").padded()
                .ignore_then(ident)
                .then_ignore(just('=').padded())
                .then(expr.clone())
                .map(|(name, value)| SqfExpr::ForceAssignment {
                    name,
                    value: Box::new(value),
                }),
            // Regular assignment (e.g., _x = 42)
            ident
                .then_ignore(just('=').padded())
                .then(expr.clone())
                .map(|(name, value)| SqfExpr::Assignment {
                    name,
                    value: Box::new(value),
                }),
            // Private declaration with assignment (e.g., private _x = 42)
            just("private").padded()
                .ignore_then(ident)
                .then_ignore(just('=').padded())
                .then(expr.clone())
                .map(|(name, value)| SqfExpr::Assignment {
                    name,
                    value: Box::new(value),
                }),
            // Variable assignment with setVariable (e.g., missionNamespace setVariable ["name", value])
            array_access.clone()
                .then(just("setVariable").padded())
                .then(array.clone())
                .map(|((namespace, _), args)| {
                    if let SqfExpr::Array(mut args) = args {
                        if args.len() >= 2 {
                            let value = args.remove(1);
                            if let SqfExpr::String(name) = args.remove(0) {
                                return SqfExpr::Assignment {
                                    name,
                                    value: Box::new(value),
                                };
                            }
                        }
                        SqfExpr::FunctionCall {
                            name: "setVariable".to_string(),
                            args,
                        }
                    } else {
                        unreachable!()
                    }
                })
        ));

        // Simplified for loop handling - just treat it as a block
        let for_loop = just("for").padded()
            .ignore_then(
                // Skip everything until we find a block
                none_of("{").repeated()
            )
            .then(block.clone())
            .map(|(_, body)| body);

        let statement = choice((
            // forEach loop (e.g., { _unit addItem _x } forEach _array)
            block.clone()
                .then_ignore(just("forEach").padded())
                .then(array_access.clone())
                .map(|(body, array)| SqfExpr::ForEach {
                    body: Box::new(body),
                    array: Box::new(array),
                }),
            for_loop,
            assignment,
            function_call.clone(),
            binary,
            array_access,
            comment,
        ))
        .then_ignore(just(';').padded().or_not());

        statement.or(block)
    })
    .repeated()
    .then_ignore(end())
    .map(|expressions| {
        let mut variables = HashMap::new();
        for expr in expressions.iter() {
            if let SqfExpr::Assignment { name, value } = expr {
                variables.insert(name.clone(), *value.clone());
            }
        }
        SqfFile {
            expressions,
            variables,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> SqfFile {
        parser().parse(input).unwrap()
    }

    #[test]
    fn test_basic_expressions() {
        // Test numbers
        let result = parse("42");
        assert!(matches!(&result.expressions[0], SqfExpr::Number(n) if n == &42.0));

        let result = parse("3.14");
        assert!(matches!(&result.expressions[0], SqfExpr::Number(n) if n == &3.14));

        // Test strings
        let result = parse("\"hello world\"");
        assert!(matches!(&result.expressions[0], SqfExpr::String(s) if s == "hello world"));

        // Test variables
        let result = parse("_someVar");
        assert!(matches!(&result.expressions[0], SqfExpr::Variable(s) if s == "_someVar"));

        // Test booleans and nil
        let result = parse("true");
        assert!(matches!(&result.expressions[0], SqfExpr::Bool(true)));
        
        let result = parse("false");
        assert!(matches!(&result.expressions[0], SqfExpr::Bool(false)));
        
        let result = parse("nil");
        assert!(matches!(&result.expressions[0], SqfExpr::Null));
    }

    #[test]
    fn test_arrays() {
        let result = parse("[1, 2, 3]");
        if let SqfExpr::Array(elements) = &result.expressions[0] {
            assert_eq!(elements.len(), 3);
            assert!(matches!(&elements[0], SqfExpr::Number(n) if n == &1.0));
            assert!(matches!(&elements[1], SqfExpr::Number(n) if n == &2.0));
            assert!(matches!(&elements[2], SqfExpr::Number(n) if n == &3.0));
        } else {
            panic!("Expected Array expression");
        }

        // Test nested arrays
        let result = parse("[[1, 2], [3, 4]]");
        if let SqfExpr::Array(outer) = &result.expressions[0] {
            assert_eq!(outer.len(), 2);
            for inner in outer {
                if let SqfExpr::Array(elements) = inner {
                    assert_eq!(elements.len(), 2);
                } else {
                    panic!("Expected inner Array expression");
                }
            }
        } else {
            panic!("Expected outer Array expression");
        }
    }

    #[test]
    fn test_binary_operations() {
        let result = parse("1 + 2");
        if let SqfExpr::BinaryOp { op, left, right } = &result.expressions[0] {
            assert_eq!(op, "+");
            assert!(matches!(**left, SqfExpr::Number(n) if n == 1.0));
            assert!(matches!(**right, SqfExpr::Number(n) if n == 2.0));
        } else {
            panic!("Expected BinaryOp expression");
        }

        // Test operator precedence with parentheses
        let result = parse("(1 + 2) * 3");
        if let SqfExpr::BinaryOp { op, left, .. } = &result.expressions[0] {
            assert_eq!(op, "*");
            if let SqfExpr::BinaryOp { op, .. } = &**left {
                assert_eq!(op, "+");
            } else {
                panic!("Expected inner BinaryOp expression");
            }
        }
    }

    #[test]
    fn test_function_calls() {
        // Test array parameter function call
        let result = parse("[_player, \"hello\"] call fnc_greet");
        if let SqfExpr::FunctionCall { name, args } = &result.expressions[0] {
            assert_eq!(name, "fnc_greet");
            assert_eq!(args.len(), 2);
            assert!(matches!(&args[0], SqfExpr::Variable(s) if s == "_player"));
            assert!(matches!(&args[1], SqfExpr::String(s) if s == "hello"));
        } else {
            panic!("Expected FunctionCall expression");
        }

        // Test single parameter function call
        let result = parse("_player call fnc_getName");
        if let SqfExpr::FunctionCall { name, args } = &result.expressions[0] {
            assert_eq!(name, "fnc_getName");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], SqfExpr::Variable(s) if s == "_player"));
        } else {
            panic!("Expected FunctionCall expression");
        }

        // Test empty array parameter function call
        let result = parse("[] call fnc_init");
        if let SqfExpr::FunctionCall { name, args } = &result.expressions[0] {
            assert_eq!(name, "fnc_init");
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected FunctionCall expression");
        }
    }

    #[test]
    fn test_assignments() {
        let result = parse("_result = 42");
        if let SqfExpr::Assignment { name, value } = &result.expressions[0] {
            assert_eq!(name, "_result");
            assert!(matches!(**value, SqfExpr::Number(n) if n == 42.0));
        } else {
            panic!("Expected Assignment expression");
        }

        // Test that assignment is stored in variables map
        assert!(result.variables.contains_key("_result"));
        assert!(matches!(result.variables.get("_result").unwrap(), SqfExpr::Number(n) if n == &42.0));
    }

    #[test]
    fn test_comments() {
        // Test single-line comments
        let result = parse("// This is a comment\n42");
        assert!(matches!(&result.expressions[0], SqfExpr::Comment(s) if s == " This is a comment"));
        assert!(matches!(&result.expressions[1], SqfExpr::Number(n) if n == &42.0));

        // Test multi-line comments
        let result = parse("/* Multi-line\ncomment */\n42");
        assert!(matches!(&result.expressions[0], SqfExpr::Comment(s) if s == " Multi-line\ncomment "));
        assert!(matches!(&result.expressions[1], SqfExpr::Number(n) if n == &42.0));
    }

    #[test]
    fn test_complex_expressions() {
        let result = parse(r#"
            _weapons = ["rifle", "pistol"];
            _ammo = [["556_mag", 30], ["9mm_mag", 15]];
            [_weapons, _ammo] call fnc_loadout;
            // Initialize player
            _health = 100;
        "#);

        assert_eq!(result.expressions.len(), 5); // 3 expressions + 1 comment
        assert!(result.variables.contains_key("_weapons"));
        assert!(result.variables.contains_key("_ammo"));
        assert!(result.variables.contains_key("_health"));
    }

    #[test]
    fn test_function_with_array_param() {
        // Test function call with array parameter
        let result = parse("random [2, 3, 5]");
        if let SqfExpr::FunctionCall { name, args } = &result.expressions[0] {
            assert_eq!(name, "random");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[0], SqfExpr::Number(n) if n == &2.0));
            assert!(matches!(&args[1], SqfExpr::Number(n) if n == &3.0));
            assert!(matches!(&args[2], SqfExpr::Number(n) if n == &5.0));
        } else {
            panic!("Expected FunctionCall expression");
        }

        // Test function call with array parameter in a more complex expression
        let result = parse(r#"
            _count = ceil (random [2, 3, 5]);
        "#);
        if let SqfExpr::Assignment { name, value } = &result.expressions[0] {
            assert_eq!(name, "_count");
            if let SqfExpr::FunctionCall { name: outer_name, args: outer_args } = &**value {
                assert_eq!(outer_name, "ceil");
                assert_eq!(outer_args.len(), 1);
                if let SqfExpr::FunctionCall { name: inner_name, args: inner_args } = &outer_args[0] {
                    assert_eq!(inner_name, "random");
                    assert_eq!(inner_args.len(), 3);
                    assert!(matches!(&inner_args[0], SqfExpr::Number(n) if n == &2.0));
                    assert!(matches!(&inner_args[1], SqfExpr::Number(n) if n == &3.0));
                    assert!(matches!(&inner_args[2], SqfExpr::Number(n) if n == &5.0));
                } else {
                    panic!("Expected inner FunctionCall expression");
                }
            } else {
                panic!("Expected outer FunctionCall expression");
            }
        } else {
            panic!("Expected Assignment expression");
        }
    }

    #[test]
    fn test_equipment_assignment() {
        // Test direct equipment assignment
        let result = parse(r#"
            _unit addHeadgear "rhs_tsh4";
        "#);
        if let SqfExpr::FunctionCall { name, args } = &result.expressions[0] {
            assert_eq!(name, "addHeadgear");
            assert_eq!(args.len(), 2);
            assert!(matches!(&args[0], SqfExpr::Variable(s) if s == "_unit"));
            assert!(matches!(&args[1], SqfExpr::String(s) if s == "rhs_tsh4"));
        } else {
            panic!("Expected FunctionCall expression");
        }

        // Test variable-based equipment assignment
        let result = parse(r#"
            _bp = "rhs_rpg_empty";
            _unit addBackpack _bp;
        "#);
        assert_eq!(result.expressions.len(), 2);
        
        if let SqfExpr::Assignment { name, value } = &result.expressions[0] {
            assert_eq!(name, "_bp");
            assert!(matches!(&**value, SqfExpr::String(s) if s == "rhs_rpg_empty"));
        } else {
            panic!("Expected Assignment expression");
        }

        if let SqfExpr::FunctionCall { name, args } = &result.expressions[1] {
            assert_eq!(name, "addBackpack");
            assert_eq!(args.len(), 2);
            assert!(matches!(&args[0], SqfExpr::Variable(s) if s == "_unit"));
            assert!(matches!(&args[1], SqfExpr::Variable(s) if s == "_bp"));
        } else {
            panic!("Expected FunctionCall expression");
        }
    }

    #[test]
    fn test_weighted_selection() {
        // Test weighted selection assignment
        let result = parse(r#"
            _facewearPoolWeighted = selectRandomWeighted [
                "goggles1", 4,
                "goggles2", 1
            ];
        "#);
        
        if let SqfExpr::Assignment { name, value } = &result.expressions[0] {
            assert_eq!(name, "_facewearPoolWeighted");
            if let SqfExpr::FunctionCall { name: func_name, args } = &**value {
                assert_eq!(func_name, "selectRandomWeighted");
                assert_eq!(args.len(), 1);
                if let SqfExpr::Array(elements) = &args[0] {
                    assert_eq!(elements.len(), 4);
                    assert!(matches!(&elements[0], SqfExpr::String(s) if s == "goggles1"));
                    assert!(matches!(&elements[1], SqfExpr::Number(n) if n == &4.0));
                    assert!(matches!(&elements[2], SqfExpr::String(s) if s == "goggles2"));
                    assert!(matches!(&elements[3], SqfExpr::Number(n) if n == &1.0));
                } else {
                    panic!("Expected Array expression");
                }
            } else {
                panic!("Expected FunctionCall expression");
            }
        } else {
            panic!("Expected Assignment expression");
        }
    }
} 