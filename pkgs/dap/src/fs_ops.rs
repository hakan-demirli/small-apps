use crate::matcher::find_occurrences;
use crate::types::{Patch, PatchOp};
use anyhow::{anyhow, Result};
use std::fs;

pub fn run_preflight_checks(patches: &[Patch]) -> Result<(), Vec<String>> {
    println!("--- Running Preflight Checks ---");
    let mut errors = Vec::new();

    for (i, patch) in patches.iter().enumerate() {
        let prefix = format!("  - Patch #{} for '{:?}':", i + 1, patch.file_path);

        match &patch.op {
            PatchOp::Move(dest) => {
                if !patch.file_path.exists() {
                    errors.push(format!("{} FAILED (Source file not found)", prefix));
                } else if dest.exists() {
                    errors.push(format!(
                        "{} FAILED (Destination file '{:?}' already exists)",
                        prefix, dest
                    ));
                } else {
                    println!("{} OK (Move to '{:?}')", prefix, dest);
                }
            }
            PatchOp::Delete => {
                if !patch.file_path.exists() {
                    errors.push(format!("{} FAILED (File not found, cannot delete)", prefix));
                } else {
                    println!("{} OK (File scheduled for deletion)", prefix);
                }
            }
            PatchOp::Modify { search, .. } => {
                if search.trim().is_empty() {
                    if !patch.file_path.exists() {
                        println!("{} OK (New File Creation)", prefix);
                        continue;
                    }

                    match fs::read_to_string(&patch.file_path) {
                        Ok(content) => {
                            if !content.trim().is_empty() {
                                errors.push(format!("{} FAILED (Search block is empty, but target file is not empty)", prefix));
                            } else {
                                println!("{} OK (Overwrite Empty File)", prefix);
                            }
                        }
                        Err(e) => {
                            errors.push(format!(
                                "{} FAILED (Could not read existing file: {})",
                                prefix, e
                            ));
                        }
                    }
                    continue;
                }

                if !patch.file_path.exists() {
                    errors.push(format!("{} FAILED (File not found)", prefix));
                    continue;
                }

                match fs::read_to_string(&patch.file_path) {
                    Ok(content) => {
                        let source_lines: Vec<String> = content
                            .split_inclusive('\n')
                            .map(|s| s.to_string())
                            .collect();

                        let (matches, _) = find_occurrences(&source_lines, search);
                        if matches.is_empty() {
                            errors.push(format!("{} FAILED (Search block not found)", prefix));
                        } else if matches.len() > 1 {
                            errors.push(format!(
                                "{} FAILED (Search block is ambiguous, found {} times)",
                                prefix,
                                matches.len()
                            ));
                        } else {
                            println!("{} OK", prefix);
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{} FAILED (Could not read file: {})", prefix, e));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn apply_patch(patch: &Patch, dry_run: bool) -> Result<String> {
    let path = &patch.file_path;
    println!("--- Applying patch to: {:?}", path);

    match &patch.op {
        PatchOp::Move(dest) => {
            if dry_run {
                Ok(format!("    [DRY RUN] File would be moved to {:?}", dest))
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::rename(path, dest)?;
                Ok(format!("    [SUCCESS] File moved to {:?}", dest))
            }
        }
        PatchOp::Delete => {
            if dry_run {
                Ok("    [DRY RUN] File would be deleted.".to_string())
            } else {
                fs::remove_file(path)?;
                Ok("    [SUCCESS] File deleted.".to_string())
            }
        }
        PatchOp::Modify { search, replace } => {
            if search.trim().is_empty() {
                if dry_run {
                    Ok("    [DRY RUN] File would be created/overwritten.".to_string())
                } else {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(path, replace)?;
                    Ok("    [SUCCESS] File created/overwritten.".to_string())
                }
            } else {
                let content = fs::read_to_string(path)?;
                let mut source_lines: Vec<String> = content
                    .split_inclusive('\n')
                    .map(|s| s.to_string())
                    .collect();

                let (matches, match_len) = find_occurrences(&source_lines, search);

                if matches.len() != 1 {
                    return Err(anyhow!(
                        "    [ERROR] Expected 1 replacement, but {} occurred. Aborting.",
                        matches.len()
                    ));
                }

                if dry_run {
                    Ok("    [DRY RUN] Patch would be applied successfully.".to_string())
                } else {
                    let start_idx = matches[0];
                    let end_idx = start_idx + match_len;

                    let replace_lines: Vec<String> = replace
                        .split_inclusive('\n')
                        .map(|s| s.to_string())
                        .collect();

                    source_lines.splice(start_idx..end_idx, replace_lines);

                    fs::write(path, source_lines.concat())?;
                    Ok("    [SUCCESS] Patch applied.".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_apply_patch_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("code.py");
        fs::write(&file_path, "def hello():\n    print('Hi')").unwrap();

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Modify {
                search: "def hello():\n    print('Hi')".to_string(),
                replace: "def hello():\n    print('Hello World')".to_string(),
            },
        };

        apply_patch(&patch, false).unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("Hello World"));
        assert!(!content.contains("Hi"));
    }

    #[test]
    fn test_file_creation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("new.rs");

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Modify {
                search: "".to_string(),
                replace: "fn main() {}".to_string(),
            },
        };

        apply_patch(&patch, false).unwrap();
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "fn main() {}");
    }

    #[test]
    fn test_move() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("old.py");
        let dst = dir.path().join("subdir").join("new.py");
        fs::write(&src, "import os").unwrap();

        let patch = Patch {
            file_path: src.clone(),
            op: PatchOp::Move(dst.clone()),
        };

        apply_patch(&patch, false).unwrap();
        assert!(!src.exists());
        assert!(dst.exists());
    }
}
