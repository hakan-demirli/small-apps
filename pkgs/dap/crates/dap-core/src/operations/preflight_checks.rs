use crate::matcher::find_occurrences;
use crate::types::{HunkLine, Patch, PatchOp};
use anyhow::Result;
use std::fs;

pub fn run_preflight_checks(patches: &[Patch]) -> Result<(), Vec<String>> {
    println!("--- Running Preflight Checks ---");
    let mut errors = Vec::new();

    for (i, patch) in patches.iter().enumerate() {
        let prefix = format!("  - Patch #{} for '{:?}':", i + 1, patch.file_path);

        if patch.file_path.exists() {
            if let Ok(metadata) = fs::metadata(&patch.file_path) {
                if metadata.permissions().readonly() {
                    errors.push(format!("{} FAILED (File is read-only)", prefix));
                    continue;
                }
            }
        }

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
                    if patch.file_path.exists() {
                        println!("{} OK (File will be overwritten)", prefix);
                    } else {
                        println!("{} OK (New file creation)", prefix);
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

                        let (matches, _) = find_occurrences(&source_lines, search, None);
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
            PatchOp::Udiff(hunks) => {
                let is_new_file = !patch.file_path.exists();

                if is_new_file {
                    if hunks.iter().any(|h| h.old_start == 0) {
                        println!("{} OK (New file creation via Udiff)", prefix);
                        continue;
                    } else {
                        errors.push(format!("{} FAILED (File not found)", prefix));
                        continue;
                    }
                }

                if hunks.is_empty() {
                    errors.push(format!("{} FAILED (Udiff patch contains no hunks)", prefix));
                    continue;
                }

                match fs::read_to_string(&patch.file_path) {
                    Ok(content) => {
                        let mut simulated_content = content;
                        let mut line_offset: isize = 0;
                        let mut all_hunks_ok = true;

                        for (h_idx, hunk) in hunks.iter().enumerate() {
                            let mut search_lines = Vec::new();
                            let mut replace_lines = Vec::new();

                            for line in &hunk.lines {
                                match line {
                                    HunkLine::Context(s) => {
                                        let content = if s.len() > 1 {
                                            s[1..].to_string()
                                        } else {
                                            "\n".to_string()
                                        };
                                        search_lines.push(content.clone());
                                        replace_lines.push(content);
                                    }
                                    HunkLine::Remove(s) => {
                                        let content = if s.len() > 1 {
                                            s[1..].to_string()
                                        } else {
                                            "\n".to_string()
                                        };
                                        search_lines.push(content);
                                    }
                                    HunkLine::Add(s) => {
                                        let content = if s.len() > 1 {
                                            s[1..].to_string()
                                        } else {
                                            "\n".to_string()
                                        };
                                        replace_lines.push(content);
                                    }
                                }
                            }

                            let search_block = search_lines.concat();
                            let replace_block = replace_lines.concat();

                            let source_lines: Vec<String> = simulated_content
                                .split_inclusive('\n')
                                .map(|s| s.to_string())
                                .collect();

                            let hint = if hunk.old_start > 0 {
                                Some((hunk.old_start as isize + line_offset).max(0) as usize)
                            } else {
                                None
                            };

                            let (matches, match_len) =
                                find_occurrences(&source_lines, &search_block, hint);

                            if matches.len() != 1 {
                                errors.push(format!(
                                    "{} FAILED (Hunk #{} failed. Expected 1 match, found {})",
                                    prefix,
                                    h_idx + 1,
                                    matches.len()
                                ));
                                all_hunks_ok = false;
                                break;
                            }

                            let start_idx = matches[0];
                            let end_idx = start_idx + match_len;

                            let mut new_lines = source_lines;
                            let replace_parts: Vec<String> = replace_block
                                .split_inclusive('\n')
                                .map(|s| s.to_string())
                                .collect();

                            line_offset += replace_parts.len() as isize - match_len as isize;

                            new_lines.splice(start_idx..end_idx, replace_parts);
                            simulated_content = new_lines.concat();
                        }

                        if all_hunks_ok {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Hunk, HunkLine};
    use tempfile::tempdir;

    #[test]
    fn test_run_preflight_checks_modify_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, "def hello():\n    pass").unwrap();

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Modify {
                search: "def hello():\n    pass".to_string(),
                replace: "def world()".to_string(),
            },
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_preflight_checks_modify_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.py");

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Modify {
                search: "def hello()".to_string(),
                replace: "def world()".to_string(),
            },
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("File not found"));
    }

    #[test]
    fn test_run_preflight_checks_move_success() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("old.py");
        let dst = dir.path().join("new.py");
        fs::write(&src, "content").unwrap();

        let patch = Patch {
            file_path: src.clone(),
            op: PatchOp::Move(dst.clone()),
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_preflight_checks_delete_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, "content").unwrap();

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Delete,
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_preflight_checks_udiff() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, "def hello():\n    pass").unwrap();

        let hunk = Hunk {
            old_start: 1,
            old_len: 2,
            new_start: 1,
            new_len: 3,
            lines: vec![
                HunkLine::Context(" def hello():\n".to_string()),
                HunkLine::Add("+    print('Hello')\n".to_string()),
                HunkLine::Context("     pass\n".to_string()),
            ],
        };

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Udiff(vec![hunk]),
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_preflight_checks_udiff_file_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.py");

        let hunk = Hunk {
            old_start: 1,
            old_len: 1,
            new_start: 1,
            new_len: 1,
            lines: vec![HunkLine::Context(" test\n".to_string())],
        };

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Udiff(vec![hunk]),
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("File not found"));
    }

    #[test]
    fn test_run_preflight_checks_udiff_empty_hunks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, "def hello():\n    pass").unwrap();

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Udiff(vec![]),
        };

        let result = run_preflight_checks(&[patch]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("contains no hunks"));
    }
}
