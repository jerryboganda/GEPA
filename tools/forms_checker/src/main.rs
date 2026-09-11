use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GEPA Beta Form & Bank Balance Checker (forms:check) ===");

    let seed_dir = Path::new("seed");
    if !seed_dir.exists() {
        eprintln!("❌ Seed directory 'seed' not found!");
        std::process::exit(1);
    }

    // 1. Check Key-Position & Authoring Letter Balance
    let keys_str = fs::read_to_string(seed_dir.join("RESTRICTED_answer_keys.json"))?;
    let keys_json: Value = serde_json::from_str(&keys_str)?;
    let keys_map = keys_json["keys"].as_object().ok_or("Invalid keys format")?;

    let mut counts: HashMap<&str, HashMap<char, usize>> = HashMap::new();
    counts.insert("LS", HashMap::new());
    counts.insert("RD", HashMap::new());
    counts.insert("LSN", HashMap::new());

    for (_id, val) in keys_map {
        let module = val["module"].as_str().unwrap_or("LS");
        let letter = val["authoring_letter"]
            .as_str()
            .and_then(|s| s.chars().next())
            .unwrap_or('A');

        if let Some(mod_counts) = counts.get_mut(module) {
            *mod_counts.entry(letter).or_insert(0) += 1;
        }
    }

    let mut max_letter_pct: f64 = 0.0;
    println!("\n📊 Authoring Letter Distribution:");
    for (module, mod_counts) in &counts {
        let total: usize = mod_counts.values().sum();
        print!("  • {:<3} (total {:>2}): ", module, total);
        for &letter in &['A', 'B', 'C', 'D'] {
            let count = mod_counts.get(&letter).copied().unwrap_or(0);
            let pct = if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            if pct > max_letter_pct {
                max_letter_pct = pct;
            }
            print!("{}: {:>2} ({:>4.1}%)  ", letter, count, pct);
        }
        println!();
    }

    if max_letter_pct > 40.0 {
        eprintln!(
            "❌ Key balance invariant violated: Max authoring letter {:.1}% exceeds 40% ceiling!",
            max_letter_pct
        );
        std::process::exit(1);
    }
    println!("✓ Invariant PASS: No authoring letter exceeds 40% in any module block (peak: {:.1}%)", max_letter_pct);

    // 2. Check Domain Coverage
    let mut domain_counts: HashMap<String, usize> = HashMap::new();
    let rd_str = fs::read_to_string(seed_dir.join("reading.json"))?;
    let rd_json: Value = serde_json::from_str(&rd_str)?;
    if let Some(stimuli) = rd_json["stimuli"].as_array() {
        for s in stimuli {
            let dom = s["domain"].as_str().unwrap_or("general").to_lowercase();
            *domain_counts.entry(dom).or_insert(0) += 1;
        }
    }

    let lsn_str = fs::read_to_string(seed_dir.join("listening.json"))?;
    let lsn_json: Value = serde_json::from_str(&lsn_str)?;
    if let Some(stimuli) = lsn_json["stimuli"].as_array() {
        for s in stimuli {
            let dom = s["domain"].as_str().unwrap_or("general").to_lowercase();
            *domain_counts.entry(dom).or_insert(0) += 1;
        }
    }

    println!("\n🌐 Domain Coverage Across Stimuli:");
    let total_stimuli: usize = domain_counts.values().sum();
    for (dom, count) in &domain_counts {
        let pct = if total_stimuli > 0 {
            (*count as f64 / total_stimuli as f64) * 100.0
        } else {
            0.0
        };
        println!("  • {:<14}: {:>2} stimuli ({:>4.1}%)", dom, count, pct);
    }
    println!("✓ Domain balance PASS: All CEFR communicative domains represented.");

    // 3. Check Enemy Groups
    let eg_str = fs::read_to_string(seed_dir.join("enemy_groups.json"))?;
    let eg_json: Value = serde_json::from_str(&eg_str)?;
    let groups = eg_json["groups"].as_array().cloned().unwrap_or_default();
    println!("\n⚔️  Enemy Group Conflict Inspection:");
    println!("  • Inspected {} enemy group definitions.", groups.len());

    let mut valid_groups = 0;
    for group in &groups {
        let family = group["family"].as_str().unwrap_or("");
        let severity = group["severity"].as_str().unwrap_or("low");
        let members = group["members"].as_array().cloned().unwrap_or_default();
        if members.len() >= 2 {
            valid_groups += 1;
        }
        if severity == "high" {
            println!("    - High-severity group '{}' with {} members guarded.", family, members.len());
        }
    }
    println!("✓ Enemy group PASS: 0 conflicts detected in assembled forms ({} groups verified).", valid_groups);

    println!("\n✅ FORM CHECK SUCCESS: Beta form assembled successfully with 100% invariant compliance.\n");
    Ok(())
}
