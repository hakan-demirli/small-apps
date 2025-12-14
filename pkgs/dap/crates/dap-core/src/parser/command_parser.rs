use crate::types::{Patch, PatchOp};
use std::path::PathBuf;

pub const MARKER_DELETE: &str = "<<<<<<< DELETE";
pub const MARKER_MOVE: &str = "<<<<<<< MOVE >>>>>>>";

pub fn parse_line_command(line: &str) -> Option<Patch> {
    let stripped = line.trim();

    if stripped.ends_with(MARKER_DELETE) {
        let parts: Vec<&str> = stripped.split(MARKER_DELETE).collect();
        if !parts.is_empty() {
            let fpath = parts[0].trim();
            if !fpath.is_empty() {
                return Some(Patch {
                    file_path: PathBuf::from(fpath),
                    op: PatchOp::Delete,
                });
            }
        }
    }

    if stripped.contains(MARKER_MOVE) {
        let parts: Vec<&str> = stripped.split(MARKER_MOVE).collect();
        if parts.len() == 2 {
            let src = parts[0].trim();
            let dst = parts[1].trim();
            if !src.is_empty() && !dst.is_empty() {
                return Some(Patch {
                    file_path: PathBuf::from(src),
                    op: PatchOp::Move(PathBuf::from(dst)),
                });
            }
        }
    }

    None
}
