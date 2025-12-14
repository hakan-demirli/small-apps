use anyhow::Result;
use dap_core::{apply_patch, parse, run_preflight_checks};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut patch_file = None;
    let mut dry_run = false;
    let mut help = false;

    for arg in &args[1..] {
        if arg == "--dry-run" {
            dry_run = true;
        } else if arg == "--help" || arg == "-h" {
            help = true;
        } else {
            patch_file = Some(arg);
        }
    }

    if help {
        println!("Usage: dap [PATCH_FILE] [--dry-run]");
        println!("Apply custom patches (diff-fenced format).");
        return Ok(());
    }

    let patch_content = if let Some(path) = patch_file {
        fs::read_to_string(path).unwrap_or_else(|_| {
            eprintln!("Error: Patch file not found at '{}'", path);
            process::exit(1);
        })
    } else {
        if atty::is(atty::Stream::Stdin) {
            eprintln!("Error: No patch file specified and no data piped from stdin.");
            process::exit(1);
        }
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    if patch_content.is_empty() {
        eprintln!("Error: Empty patch content.");
        process::exit(1);
    }

    let patches = parse(&patch_content);

    if patches.is_empty() {
        println!("No valid patch blocks or commands found in the input.");
        process::exit(0);
    }

    match run_preflight_checks(&patches) {
        Ok(_) => println!("\n--- Preflight Checks Passed. Proceeding with patching. ---"),
        Err(errors) => {
            println!("\n--- Preflight Checks Failed ---");
            for err in errors {
                println!("{}", err);
            }
            println!("\nAborting. No files were modified.");
            process::exit(1);
        }
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    for patch in &patches {
        match apply_patch(patch, dry_run) {
            Ok(msg) => {
                println!("{}", msg);
                success_count += 1;
            }
            Err(e) => {
                println!("{}", e);
                fail_count += 1;
            }
        }
    }

    println!("\n--- Summary ---");
    println!("Total patches:        {}", patches.len());
    println!("Successfully applied: {}", success_count);
    println!("Failed to apply:      {}", fail_count);

    if fail_count > 0 {
        process::exit(1);
    }

    Ok(())
}
