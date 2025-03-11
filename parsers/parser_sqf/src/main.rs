mod ast;
mod parser;
mod scanner;

use std::env;
use std::fs;
use chumsky::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <sqf_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            std::process::exit(1);
        }
    };

    match parser::parser().parse(source) {
        Ok(sqf_file) => {
            let mut all_items = Vec::new();
            
            for expr in &sqf_file.expressions {
                all_items.extend(scanner::scan_single(expr));
            }

            println!("Found items in {}:", file_path);
            
            // Print all items with their contexts
            for item in &all_items {
                println!("  {} (in {})", item.item_id, item.context);
            }

            println!("\nVariables:");
            for (name, value) in &sqf_file.variables {
                if matches!(value, ast::SqfExpr::Array(_)) {
                    println!("  {} = Array[...]", name);
                } else {
                    println!("  {} = {:?}", name, value);
                }
            }
        }
        Err(errs) => {
            eprintln!("Parse errors:");
            for err in errs {
                eprintln!("  {}", err);
            }
            std::process::exit(1);
        }
    }
} 