use parsers::{hpp, sqf};

use std::fs;
use std::path::Path;
use std::error::Error;
use std::env;
use std::num::NonZeroU32;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <file_path> [file_type]", args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    let file_type = if args.len() >= 3 {
        Some(args[2].as_str())
    } else {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
    };
    
    let content = fs::read_to_string(file_path)?;
    
    match file_type {
        Some("hpp") => process_hpp(&content),
        Some("sqf") => process_sqf(&content),
        Some("sqm") => process_sqm(&content),
        _ => {
            println!("Unsupported file type. Please specify one of: hpp, sqf, sqm");
            Ok(())
        }
    }
}

fn process_hpp(content: &str) -> Result<(), Box<dyn Error>> {
    println!("Processing HPP file...");
    
    // Try to parse the items array
    if let Ok((_, items)) = hpp::parsers::items_array(content) {
        println!("Found {} items:", items.len());
        for item in items {
            println!("- {} ({})", item.item_name, item.count.map_or(1, |c| c.get()));
        }
    } else {
        println!("No items array found. Looking for any item references...");
        
        // Try to find any item references
        let mut start = 0;
        while start < content.len() {
            if let Some(pos) = content[start..].find('"') {
                let pos = start + pos;
                if let Ok((rest, item)) = hpp::parsers::string_literal(&content[pos..]) {
                    println!("- {}", item);
                    start = pos + (content.len() - rest.len());
                } else {
                    start = pos + 1;
                }
            } else {
                break;
            }
        }
    }
    
    Ok(())
}

fn process_sqf(content: &str) -> Result<(), Box<dyn Error>> {
    println!("Processing SQF file...");
    
    let items = sqf::parsers::parse_sqf_content(content);
    
    if items.is_empty() {
        println!("No item additions found.");
    } else {
        println!("Found {} item additions:", items.len());
        for item in items {
            let container_info = match &item.container {
                Some(container) => format!(" in {}", container),
                None => String::new(),
            };
            println!("- {}{}", item.item_name, container_info);
        }
    }
    
    Ok(())
}

fn process_sqm(content: &str) -> Result<(), Box<dyn Error>> {
    println!("Processing SQM file...");
    
    // Note: We'll need to implement the SQM parser module
    println!("SQM parsing not yet implemented");
    
    Ok(())
}
