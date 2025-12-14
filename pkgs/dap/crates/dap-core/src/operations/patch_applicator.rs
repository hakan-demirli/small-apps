use crate::matcher::find_occurrences;
use crate::types::{HunkLine, Patch, PatchOp};
use anyhow::{anyhow, Result};
use std::fs;

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

                let (matches, match_len) = find_occurrences(&source_lines, search, None);

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
        PatchOp::Udiff(hunks) => {
            let mut current_content = if path.exists() {
                fs::read_to_string(path)?
            } else {
                String::new()
            };

            let mut line_offset: isize = 0;

            for (i, hunk) in hunks.iter().enumerate() {
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

                if search_block.is_empty() && current_content.is_empty() {
                    current_content = replace_block;
                    continue;
                }

                let source_lines: Vec<String> = current_content
                    .split_inclusive('\n')
                    .map(|s| s.to_string())
                    .collect();

                let hint = if hunk.old_start > 0 {
                    Some((hunk.old_start as isize + line_offset).max(0) as usize)
                } else {
                    None
                };

                let (matches, match_len) = find_occurrences(&source_lines, &search_block, hint);

                if matches.len() != 1 {
                    return Err(anyhow!(
                        "    [ERROR] Hunk #{} failed. Expected 1 match for block, found {}.\nSearch block:\n---\n{}---",
                        i + 1,
                        matches.len(),
                        search_block
                    ));
                }

                let start_idx = matches[0];
                let end_idx = start_idx + match_len;

                let mut new_lines = source_lines;
                let replace_parts: Vec<String> = replace_block
                    .split_inclusive('\n')
                    .map(|s| s.to_string())
                    .collect();

                let added = replace_parts.len();
                let removed = match_len;
                line_offset += added as isize - removed as isize;

                new_lines.splice(start_idx..end_idx, replace_parts);
                current_content = new_lines.concat();
            }

            if dry_run {
                Ok("    [DRY RUN] Udiff patch(es) would be applied.".to_string())
            } else {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, current_content)?;
                Ok("    [SUCCESS] Udiff patch(es) applied.".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Hunk, HunkLine};
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

    #[test]
    fn test_apply_udiff_simple_addition() {
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

        apply_patch(&patch, false).unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("print('Hello')"));
        assert!(content.contains("def hello():"));
        assert!(content.contains("pass"));
    }

    #[test]
    fn test_apply_udiff_simple_removal() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, "def hello():\n    print('Debug')\n    pass").unwrap();

        let hunk = Hunk {
            old_start: 1,
            old_len: 3,
            new_start: 1,
            new_len: 2,
            lines: vec![
                HunkLine::Context(" def hello():\n".to_string()),
                HunkLine::Remove("-    print('Debug')\n".to_string()),
                HunkLine::Context("     pass\n".to_string()),
            ],
        };

        let patch = Patch {
            file_path: file_path.clone(),
            op: PatchOp::Udiff(vec![hunk]),
        };

        apply_patch(&patch, false).unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("print('Debug')"));
        assert!(content.contains("def hello():"));
        assert!(content.contains("pass"));
    }

    #[test]
    fn test_apply_udiff_dry_run() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        let original_content = "def hello():\n    pass";
        fs::write(&file_path, original_content).unwrap();

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

        let result = apply_patch(&patch, true).unwrap();
        assert!(result.contains("DRY RUN"));

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, original_content);
    }
}
