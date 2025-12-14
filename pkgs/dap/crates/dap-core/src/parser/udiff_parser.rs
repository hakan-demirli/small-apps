use crate::types::{Hunk, Patch, PatchOp};
use std::path::Path;

pub const UDIFF_OLD_FILE_PREFIX: &str = "--- ";
pub const UDIFF_NEW_FILE_PREFIX: &str = "+++ ";
pub const UDIFF_HUNK_HEADER_PREFIX: &str = "@@ ";

pub fn handle_udiff_line(
    line: &str,
    stripped: &str,
    file_path: &Path,
    new_file_path: Option<&Path>,
    current_hunks: &mut Vec<Hunk>,
    previous_line: &mut String,
) -> Option<(super::ParserState, Vec<Patch>)> {
    let mut patches = Vec::new();
    let mut new_state = super::ParserState::InUdiff;

    if stripped.starts_with(UDIFF_HUNK_HEADER_PREFIX) {
        if let Some(hunk) = parse_udiff_hunk_header(stripped) {
            current_hunks.push(hunk);
        }
    } else if stripped.starts_with(UDIFF_NEW_FILE_PREFIX) {
    } else if stripped.starts_with(UDIFF_OLD_FILE_PREFIX) {
        patches.extend(finalize_udiff_patch(
            file_path,
            new_file_path,
            std::mem::take(current_hunks),
        ));

        new_state = super::ParserState::Idle;
        *previous_line = line.to_string();
    } else if stripped.starts_with("Binary files") {
        patches.extend(finalize_udiff_patch(
            file_path,
            new_file_path,
            std::mem::take(current_hunks),
        ));
        new_state = super::ParserState::Idle;
        previous_line.clear();
    } else if let Some(last_hunk) = current_hunks.last_mut() {
        if line.starts_with("-") {
            last_hunk
                .lines
                .push(crate::types::HunkLine::Remove(line.to_string()));
        } else if line.starts_with("+") {
            last_hunk
                .lines
                .push(crate::types::HunkLine::Add(line.to_string()));
        } else if line.starts_with(" ") || stripped.is_empty() {
            last_hunk
                .lines
                .push(crate::types::HunkLine::Context(line.to_string()));
        } else if stripped.starts_with("\\") {
        } else {
            patches.extend(finalize_udiff_patch(
                file_path,
                new_file_path,
                std::mem::take(current_hunks),
            ));
            new_state = super::ParserState::Idle;
            *previous_line = line.to_string();
        }
    } else if stripped.is_empty() {
    } else {
        patches.extend(finalize_udiff_patch(
            file_path,
            new_file_path,
            std::mem::take(current_hunks),
        ));
        new_state = super::ParserState::Idle;
        *previous_line = line.to_string();
    }

    Some((new_state, patches))
}

pub fn finalize_udiff_patch(
    old_path: &Path,
    new_path: Option<&Path>,
    hunks: Vec<Hunk>,
) -> Vec<Patch> {
    let target_path = new_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| old_path.to_path_buf());

    let old_str = old_path.to_string_lossy();
    let new_str = target_path.to_string_lossy();
    let is_creation = old_str == "/dev/null";
    let is_deletion = new_str == "/dev/null";

    if is_deletion {
        vec![Patch {
            file_path: old_path.to_path_buf(),
            op: PatchOp::Delete,
        }]
    } else if is_creation {
        vec![Patch {
            file_path: target_path,
            op: PatchOp::Udiff(hunks),
        }]
    } else if old_path != target_path {
        let mut patches = vec![Patch {
            file_path: old_path.to_path_buf(),
            op: PatchOp::Move(target_path.clone()),
        }];

        if !hunks.is_empty() {
            patches.push(Patch {
                file_path: target_path,
                op: PatchOp::Udiff(hunks),
            });
        }
        patches
    } else {
        if hunks.is_empty() {
            return vec![];
        }
        vec![Patch {
            file_path: target_path,
            op: PatchOp::Udiff(hunks),
        }]
    }
}

pub fn parse_udiff_hunk_header(header: &str) -> Option<Hunk> {
    if !header.starts_with("@@") {
        return None;
    }

    let mut old_start = 0;

    for part in header.split_whitespace() {
        if part.starts_with('-') && part.len() > 1 {
            let num_part = &part[1..];

            let start_str = num_part.split(',').next().unwrap_or("");
            if let Ok(num) = start_str.parse::<usize>() {
                old_start = num;
                break;
            }
        }
    }

    Some(Hunk {
        old_start,
        old_len: 0,
        new_start: 0,
        new_len: 0,
        lines: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_udiff_hunk_header_variations() {
        let zero_hunk = Hunk {
            old_start: 0,
            old_len: 0,
            new_start: 0,
            new_len: 0,
            lines: Vec::new(),
        };

        let hunk_standard = Hunk {
            old_start: 10,
            ..zero_hunk.clone()
        };
        assert_eq!(
            parse_udiff_hunk_header("@@ -10,5 +12,8 @@"),
            Some(hunk_standard)
        );

        assert_eq!(
            parse_udiff_hunk_header("@@ ... @@"),
            Some(zero_hunk.clone())
        );

        assert_eq!(parse_udiff_hunk_header("@@"), Some(zero_hunk.clone()));

        assert_eq!(
            parse_udiff_hunk_header("@@ nonsense @@"),
            Some(zero_hunk.clone())
        );

        assert_eq!(parse_udiff_hunk_header("no markers"), None);
    }
}
