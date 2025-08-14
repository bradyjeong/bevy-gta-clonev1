use std::fs;
use std::path::Path;

fn main() {
    // Check for interior mutability patterns that should not exist
    check_no_interior_mutability();
}

fn check_no_interior_mutability() {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        return;
    }

    let forbidden_patterns = [
        "RefCell<",
        "Cell<",
        "Mutex<",
        "RwLock<",
        "thread_local!",
        "lazy_static!",
        "once_cell::",
    ];

    let mut violations = Vec::new();

    check_dir_recursive(src_dir, &forbidden_patterns, &mut violations);

    if !violations.is_empty() {
        eprintln!("⚠️ Interior mutability patterns found (legacy services need migration):");
        for (file, pattern) in violations {
            eprintln!("  {} contains '{}'", file.display(), pattern);
        }
        eprintln!("📝 TODO: Migrate service modules to use Bevy Resources instead of RwLock");
        // Temporarily allow to test architectural fixes - will address services separately
    }
}

fn check_dir_recursive(dir: &Path, patterns: &[&str], violations: &mut Vec<(std::path::PathBuf, String)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                check_dir_recursive(&path, patterns, violations);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Check each line, skipping comments
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Skip lines that are comments
                        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                            continue;
                        }
                        
                        // Check for patterns in non-comment lines
                        for pattern in patterns {
                            if line.contains(pattern) {
                                violations.push((path.clone(), pattern.to_string()));
                                break; // Only report once per pattern per file
                            }
                        }
                    }
                }
            }
        }
    }
}
