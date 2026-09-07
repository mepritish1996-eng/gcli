//! Real stdin/file input handling — docs/UNIX_grammar.md §6 (piping): every
//! verb reads from stdin when no input file is given, so verbs compose via
//! `|` the same way real Unix tools do.

use std::fs;
use std::io::Read;

use crate::error::{ErrorCode, GcliError};

/// Reads raw bytes from either a named file, or stdin when `path` is `None`
/// (or the conventional `-`). This is the one real input-resolution rule
/// every verb shares — never reimplemented per-verb.
pub fn read_input(verb: &'static str, path: Option<&str>) -> Result<String, GcliError> {
    match path {
        None | Some("-") => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| GcliError::new(verb, ErrorCode::InputNotFound, format!("failed to read stdin: {e}")))?;
            Ok(buf)
        }
        Some(p) => fs::read_to_string(p)
            .map_err(|e| GcliError::new(verb, ErrorCode::InputNotFound, format!("\"{p}\": {e}"))),
    }
}
