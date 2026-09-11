use shared_engine::wording_policy::scan_candidate_copy;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    println!("=== GEPA Claims Policy & Copy Linter ===");
    let mut violations = 0;

    let target_dirs = ["client/src/copy", "client/src/pages", "client/src/components"];

    for dir_str in &target_dirs {
        let dir = Path::new(dir_str);
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "ts" || ext == "tsx" || ext == "astro" || ext == "json" {
                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    for (line_num, line) in content.lines().enumerate() {
                        // Support intentional whitelist comments if any
                        if line.contains("// copy-lint-allow") {
                            continue;
                        }
                        // Ignore comments or imports
                        let trimmed = line.trim();
                        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                            continue;
                        }

                        if let Err(err) = scan_candidate_copy(trimmed) {
                            eprintln!(
                                "Forbidden wording violation in {}:{}: {}\n   Line: {}",
                                path.display(),
                                line_num + 1,
                                err,
                                trimmed
                            );
                            violations += 1;
                        }
                    }
                }
            }
        }
    }

    if violations > 0 {
        eprintln!("\n❌ Copy lint failed with {} violations.", violations);
        std::process::exit(1);
    } else {
        println!("\n✅ Copy lint PASSED: 0 forbidden wording violations.");
    }
}
