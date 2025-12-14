pub mod command_parser;
pub mod diff_parser;
pub mod udiff_parser;

use crate::types::{Hunk, Patch};
use std::path::PathBuf;

pub fn parse(content: &str) -> Vec<Patch> {
    let mut patches = Vec::new();
    let mut state = ParserState::Idle;
    let mut previous_line = String::new();
    let mut file_path = PathBuf::new();
    let mut udiff_new_path: Option<PathBuf> = None;
    let mut search_lines = Vec::new();
    let mut replace_lines = Vec::new();
    let mut current_hunks: Vec<Hunk> = Vec::new();

    for line in content.split_inclusive('\n') {
        let stripped = line.trim();

        match state {
            ParserState::Idle => {
                if stripped == diff_parser::MARKER_SEARCH_START {
                    let potential_path = previous_line.trim();
                    if !potential_path.is_empty() {
                        file_path = PathBuf::from(potential_path);
                    }
                    state = ParserState::InSearch;
                    search_lines.clear();
                    replace_lines.clear();
                } else if stripped.starts_with(udiff_parser::UDIFF_OLD_FILE_PREFIX) {
                    file_path = PathBuf::from(
                        stripped
                            .trim_start_matches(udiff_parser::UDIFF_OLD_FILE_PREFIX)
                            .trim(),
                    );
                    state = ParserState::InUdiff;
                    current_hunks.clear();
                    udiff_new_path = None;
                } else if let Some(patch) = command_parser::parse_line_command(line) {
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
                if stripped == diff_parser::MARKER_DIVIDER {
                    state = ParserState::InReplace;
                } else {
                    search_lines.push(line);
                }
            }
            ParserState::InReplace => {
                if stripped == diff_parser::MARKER_REPLACE_END {
                    patches.push(Patch {
                        file_path: file_path.clone(),
                        op: crate::types::PatchOp::Modify {
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
            ParserState::InUdiff => {
                if stripped.starts_with(udiff_parser::UDIFF_OLD_FILE_PREFIX) {
                    patches.extend(udiff_parser::finalize_udiff_patch(
                        &file_path,
                        udiff_new_path.as_deref(),
                        std::mem::take(&mut current_hunks),
                    ));

                    file_path = PathBuf::from(
                        stripped
                            .trim_start_matches(udiff_parser::UDIFF_OLD_FILE_PREFIX)
                            .trim(),
                    );
                    udiff_new_path = None;
                } else if stripped.starts_with(udiff_parser::UDIFF_NEW_FILE_PREFIX) {
                    udiff_new_path = Some(PathBuf::from(
                        stripped
                            .trim_start_matches(udiff_parser::UDIFF_NEW_FILE_PREFIX)
                            .trim(),
                    ));
                } else if let Some((new_state, new_patches)) = udiff_parser::handle_udiff_line(
                    line,
                    stripped,
                    file_path.as_path(),
                    udiff_new_path.as_deref(),
                    &mut current_hunks,
                    &mut previous_line,
                ) {
                    state = new_state;
                    patches.extend(new_patches);

                    if state == ParserState::Idle {
                        let prev_stripped = previous_line.trim();
                        if prev_stripped == diff_parser::MARKER_SEARCH_START {
                        } else if let Some(patch) =
                            command_parser::parse_line_command(&previous_line)
                        {
                            patches.push(patch);
                            previous_line.clear();
                        }
                    }
                }
            }
        }
    }

    if state == ParserState::InUdiff {
        patches.extend(udiff_parser::finalize_udiff_patch(
            &file_path,
            udiff_new_path.as_deref(),
            std::mem::take(&mut current_hunks),
        ));
    }

    patches
}

#[derive(PartialEq)]
pub enum ParserState {
    Idle,
    InSearch,
    InReplace,
    InUdiff,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::command_parser::*;
    use crate::parser::diff_parser::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_diff_fenced() {
        let patch_text = format!(
            "src/main.rs\n{}\nold\n{}\nnew\n{}\n",
            MARKER_SEARCH_START, MARKER_DIVIDER, MARKER_REPLACE_END
        );
        let patches = parse(&patch_text);
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].file_path, PathBuf::from("src/main.rs"));
        if let crate::types::PatchOp::Modify { search, .. } = &patches[0].op {
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
        assert_eq!(patches[0].op, crate::types::PatchOp::Delete);
        if let crate::types::PatchOp::Move(dest) = &patches[1].op {
            assert_eq!(dest, &PathBuf::from("src/new.rs"));
        } else {
            panic!("Expected Move op");
        }
    }

    #[test]
    fn test_parse_mixed_formats() {
        let mixed_content = format!(
            r#"--- file1.py
+++ file1.py
@@ -1,1 +1,2 @@
 print("hello")
+print("world")

file2.rs
{}
old code
{}
new code
{}

file3.txt {}
"#,
            MARKER_SEARCH_START, MARKER_DIVIDER, MARKER_REPLACE_END, MARKER_DELETE
        );

        let patches = parse(&mixed_content);
        assert_eq!(patches.len(), 3);

        match &patches[0].op {
            crate::types::PatchOp::Udiff(_) => {}
            _ => panic!("Expected Udiff op"),
        }

        match &patches[1].op {
            crate::types::PatchOp::Modify { .. } => {}
            _ => panic!("Expected Modify op"),
        }

        match &patches[2].op {
            crate::types::PatchOp::Delete => {}
            _ => panic!("Expected Delete op"),
        }
    }
}
