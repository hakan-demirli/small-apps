use crate::types::{Patch, PatchOp};
use std::path::PathBuf;

const MARKER_SEARCH_START: &str = "<<<<<<< SEARCH";
const MARKER_DIVIDER: &str = "=======";
const MARKER_REPLACE_END: &str = ">>>>>>> REPLACE";
const MARKER_DELETE: &str = "<<<<<<< DELETE";
const MARKER_MOVE: &str = "<<<<<<< MOVE >>>>>>>";

#[derive(PartialEq)]
enum ParserState {
    Idle,
    InSearch,
    InReplace,
}

pub fn parse(content: &str) -> Vec<Patch> {
    let mut patches = Vec::new();
    let mut state = ParserState::Idle;
    let mut previous_line = String::new();
    let mut file_path = PathBuf::new();
    let mut search_lines = Vec::new();
    let mut replace_lines = Vec::new();

    for line in content.split_inclusive('\n') {
        let stripped = line.trim();

        match state {
            ParserState::Idle => {
                if stripped == MARKER_SEARCH_START {
                    let potential_path = previous_line.trim();
                    if !potential_path.is_empty() {
                        file_path = PathBuf::from(potential_path);
                    }
                    state = ParserState::InSearch;
                    search_lines.clear();
                    replace_lines.clear();
                } else if let Some(patch) = parse_line_command(line) {
                    patches.push(patch);
                    previous_line.clear();
                } else if stripped.starts_with("```") {
                } else if stripped.is_empty() {
                    previous_line.clear();
                } else {
                    previous_line = line.to_string();
                }
            }
            ParserState::InSearch => {
                if stripped == MARKER_DIVIDER {
                    state = ParserState::InReplace;
                } else {
                    search_lines.push(line);
                }
            }
            ParserState::InReplace => {
                if stripped == MARKER_REPLACE_END {
                    patches.push(Patch {
                        file_path: file_path.clone(),
                        op: PatchOp::Modify {
                            search: search_lines.concat(),
                            replace: replace_lines.concat(),
                        },
                    });
                    state = ParserState::Idle;
                    previous_line.clear();
                } else {
                    replace_lines.push(line);
                }
            }
        }
    }

    patches
}

fn parse_line_command(line: &str) -> Option<Patch> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_fenced() {
        let patch_text = format!(
            "src/main.rs\n{}\nold\n{}\nnew\n{}\n",
            MARKER_SEARCH_START, MARKER_DIVIDER, MARKER_REPLACE_END
        );
        let patches = parse(&patch_text);
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].file_path, PathBuf::from("src/main.rs"));
        if let PatchOp::Modify { search, .. } = &patches[0].op {
            assert!(search.contains("old"));
        } else {
            panic!("Wrong op");
        }
    }

    #[test]
    fn test_parse_marker_edge_cases() {
        let indented = format!(
            "\n    file1.rs\n      {}\n    old\n    {}\n    new\n    {}\n    ",
            MARKER_SEARCH_START, MARKER_DIVIDER, MARKER_REPLACE_END
        );
        let patches = parse(&indented);
        assert_eq!(patches.len(), 1, "Should parse indented start markers");
        assert_eq!(patches[0].file_path, PathBuf::from("file1.rs"));

        let polluted = format!(
            "\n    file2.rs\n    some_code {}\n    old\n    {}\n    new\n    {}\n    ",
            MARKER_SEARCH_START, MARKER_DIVIDER, MARKER_REPLACE_END
        );
        let patches_bad = parse(&polluted);
        assert_eq!(
            patches_bad.len(),
            0,
            "Should ignore markers preceded by text"
        );
    }

    #[test]
    fn test_parse_move_delete() {
        let content = format!(
            "file_to_delete.rs {}\nsrc/old.rs {} src/new.rs",
            MARKER_DELETE, MARKER_MOVE
        );
        let patches = parse(&content);
        assert_eq!(patches.len(), 2);
        assert_eq!(patches[0].op, PatchOp::Delete);
        if let PatchOp::Move(dest) = &patches[1].op {
            assert_eq!(dest, &PathBuf::from("src/new.rs"));
        } else {
            panic!("Expected Move op");
        }
    }
}
