pub mod matcher;
pub mod operations;
pub mod parser;
pub mod types;

pub use operations::{apply_patch, run_preflight_checks};
pub use parser::parse;
pub use types::{Hunk, HunkLine, Patch, PatchOp};
