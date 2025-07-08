#!/usr/bin/env rust-script

use std::fs;
use std::collections::HashMap;
use serde_json::Value;

fn main() {
    let content = fs::read_to_string("compile.log").unwrap_or_default();
    
    let mut error_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut total_errors = 0;
    
    for line in content.lines() {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(message) = json.get("message") {
                if let Some(level) = message.get("level") {
                    if level == "error" {
                        total_errors += 1;
                        let package = json.get("package_id")
                            .and_then(|p| p.as_str())
                            .unwrap_or("unknown");
                        
                        let message_text = message.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown error");
                        
                        let location = if let Some(spans) = message.get("spans") {
                            if let Some(span) = spans.as_array().and_then(|a| a.first()) {
                                format!("{}:{}:{}", 
                                    span.get("file_name").and_then(|f| f.as_str()).unwrap_or("unknown"),
                                    span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0),
                                    span.get("column_start").and_then(|c| c.as_u64()).unwrap_or(0)
                                )
                            } else {
                                "unknown location".to_string()
                            }
                        } else {
                            "unknown location".to_string()
                        };
                        
                        let crate_name = package.split('#').next().unwrap_or(package);
                        error_map.entry(crate_name.to_string())
                            .or_default()
                            .push(format!("{}: {}", location, message_text));
                    }
                }
            }
        }
    }
    
    // Create errors directory
    fs::create_dir_all("errors/by_crate").unwrap();
    
    // Generate summary
    let mut summary = format!("# Compilation Error Summary\n\nTotal errors: {}\n\n", total_errors);
    
    for (crate_name, errors) in &error_map {
        summary.push_str(&format!("## {} ({} errors)\n\n", crate_name, errors.len()));
        for error in errors {
            summary.push_str(&format!("- {}\n", error));
        }
        summary.push('\n');
        
        // Create per-crate file
        let crate_content = errors.join("\n");
        fs::write(format!("errors/by_crate/{}.md", crate_name), crate_content).unwrap();
    }
    
    fs::write("ERROR_SUMMARY.md", summary).unwrap();
    println!("Generated ERROR_SUMMARY.md and errors/by_crate/*.md files");
    println!("Total compilation errors found: {}", total_errors);
}
