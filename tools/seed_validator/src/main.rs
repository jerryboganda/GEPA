use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GEPA Seed Validator ===");
    let seed_dir = Path::new("seed");
    if !seed_dir.exists() {
        eprintln!("Error: seed directory not found at 'seed'");
        std::process::exit(1);
    }

    let manifest_str = fs::read_to_string(seed_dir.join("manifest.json"))?;
    let manifest: Value = serde_json::from_str(&manifest_str)?;

    let expected_files = manifest["files"]
        .as_object()
        .expect("manifest.json missing 'files'");

    let mut checksum_errors = 0;
    for (filename, expected_hash) in expected_files {
        let file_path = seed_dir.join(filename);
        if !file_path.exists() {
            eprintln!("Missing required seed file: {}", filename);
            checksum_errors += 1;
            continue;
        }
        let data = fs::read(&file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let actual_hash = format!("{:x}", hasher.finalize());
        let expected = expected_hash.as_str().unwrap_or("");
        if actual_hash != expected {
            eprintln!(
                "Checksum mismatch for {}: expected {}, got {}",
                filename, expected, actual_hash
            );
            checksum_errors += 1;
        } else {
            println!("✓ Checksum valid: {}", filename);
        }
    }

    // Verify item counts
    let ls_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("language_systems.json"))?)?;
    let ls_items = ls_val["items"].as_array().expect("LS missing items");
    assert_eq!(ls_items.len(), 52, "Expected 52 LS items, got {}", ls_items.len());
    println!("✓ LS item count: 52");

    let rd_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("reading.json"))?)?;
    let rd_items = rd_val["items"].as_array().expect("RD missing items");
    let rd_stimuli = rd_val["stimuli"].as_array().expect("RD missing stimuli");
    assert_eq!(rd_items.len(), 42, "Expected 42 RD items, got {}", rd_items.len());
    assert_eq!(rd_stimuli.len(), 21, "Expected 21 RD stimuli, got {}", rd_stimuli.len());
    println!("✓ RD item count: 42, stimuli count: 21");

    let lsn_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("listening.json"))?)?;
    let lsn_items = lsn_val["items"].as_array().expect("LSN missing items");
    let lsn_stimuli = lsn_val["stimuli"].as_array().expect("LSN missing stimuli");
    assert_eq!(lsn_items.len(), 42, "Expected 42 LSN items, got {}", lsn_items.len());
    assert_eq!(lsn_stimuli.len(), 21, "Expected 21 LSN stimuli, got {}", lsn_stimuli.len());
    println!("✓ LSN item count: 42, stimuli count: 21");

    let spk_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("speaking_tasks.json"))?)?;
    let spk_tasks = spk_val["tasks"].as_array().expect("SPK missing tasks");
    assert_eq!(spk_tasks.len(), 48, "Expected 48 SPK tasks, got {}", spk_tasks.len());
    println!("✓ SPK task count: 48");

    let wrt_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("writing_tasks.json"))?)?;
    let wrt_tasks = wrt_val["tasks"].as_array().expect("WRT missing tasks");
    assert_eq!(wrt_tasks.len(), 24, "Expected 24 WRT tasks, got {}", wrt_tasks.len());
    println!("✓ WRT task count: 24");

    let keys_val: Value = serde_json::from_str(&fs::read_to_string(seed_dir.join("RESTRICTED_answer_keys.json"))?)?;
    let keys = keys_val["keys"].as_object().expect("RESTRICTED_answer_keys missing keys map");
    assert_eq!(keys.len(), 136, "Expected 136 answer keys (52+42+42), got {}", keys.len());
    println!("✓ Answer key count: 136");

    if checksum_errors > 0 {
        eprintln!("Seed validation failed with {} errors.", checksum_errors);
        std::process::exit(1);
    }

    println!("\n Seed bank validation successful: ALL INVARIANTS PASS.");
    Ok(())
}
