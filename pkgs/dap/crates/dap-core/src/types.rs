use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    Delete,
    Move(PathBuf),
    Modify { search: String, replace: String },
    Udiff(Vec<Hunk>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hunk {
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub lines: Vec<HunkLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HunkLine {
    Context(String),
    Add(String),
    Remove(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub file_path: PathBuf,
    pub op: PatchOp,
}
