use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    Delete,
    Move(PathBuf),
    Modify { search: String, replace: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub file_path: PathBuf,
    pub op: PatchOp,
}
