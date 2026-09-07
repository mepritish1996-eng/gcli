//! Real error/exit-code contract — docs/UNIX_grammar.md §16 (stderr) + §research
//! (sysexits.h-style numeric exit classes). Every verb returns a `GcliError`
//! instead of an ad hoc string; the binary crate is the only place that
//! actually prints it and calls `std::process::exit`.

use std::fmt;
use std::process::ExitCode;

/// The fixed, shared error-code vocabulary (§16). Extend deliberately —
/// this is a contract the agent and scripts depend on, not a free-text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InputNotFound,
    InvalidGeometry,
    UnsupportedFormat,
    CrsUnknown,
    PermissionDenied,
    Timeout,
    Internal,
}

impl ErrorCode {
    /// The stable, machine-readable string form (`E_*`), used both in the
    /// human-readable stderr line and the `--json` JSON-lines form.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InputNotFound => "E_INPUT_NOT_FOUND",
            ErrorCode::InvalidGeometry => "E_INVALID_GEOMETRY",
            ErrorCode::UnsupportedFormat => "E_UNSUPPORTED_FORMAT",
            ErrorCode::CrsUnknown => "E_CRS_UNKNOWN",
            ErrorCode::PermissionDenied => "E_PERMISSION_DENIED",
            ErrorCode::Timeout => "E_TIMEOUT",
            ErrorCode::Internal => "E_INTERNAL",
        }
    }

    /// Real sysexits.h-style numeric class (§research) — lets a caller
    /// branch on failure *class* via the process exit code alone, without
    /// parsing stderr, same as BSD tools already do.
    pub fn exit_code(self) -> u8 {
        match self {
            ErrorCode::InputNotFound => 66,   // EX_NOINPUT
            ErrorCode::InvalidGeometry => 65, // EX_DATAERR
            ErrorCode::UnsupportedFormat => 65, // EX_DATAERR
            ErrorCode::CrsUnknown => 65,      // EX_DATAERR
            ErrorCode::PermissionDenied => 77, // EX_NOPERM
            ErrorCode::Timeout => 75,         // EX_TEMPFAIL (retry-safe)
            ErrorCode::Internal => 70,        // EX_SOFTWARE
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
        }
    }
}

/// One real stderr line: `gcli: <verb>: <LEVEL>: <CODE>: <message>` (§16).
#[derive(Debug, Clone)]
pub struct GcliError {
    pub verb: &'static str,
    pub level: Level,
    pub code: ErrorCode,
    pub message: String,
}

impl GcliError {
    pub fn new(verb: &'static str, code: ErrorCode, message: impl Into<String>) -> Self {
        GcliError { verb, level: Level::Error, code, message: message.into() }
    }

    pub fn warn(verb: &'static str, code: ErrorCode, message: impl Into<String>) -> Self {
        GcliError { verb, level: Level::Warn, code, message: message.into() }
    }

    /// The real, numeric process exit code this error should produce.
    /// A WARN never changes the exit code — only ERROR does (§16: "the
    /// process still succeeds and exits 0" for warnings).
    pub fn process_exit_code(&self) -> ExitCode {
        match self.level {
            Level::Warn | Level::Info => ExitCode::SUCCESS,
            Level::Error => ExitCode::from(self.code.exit_code()),
        }
    }

    /// JSON-lines form for `--json` mode (§16) — one JSON object per line,
    /// same discipline as stdout's GeoJSONSeq, never free-text prose.
    pub fn to_json_line(&self) -> String {
        serde_json::json!({
            "level": self.level.as_str(),
            "code": self.code.as_str(),
            "verb": self.verb,
            "message": self.message,
        })
        .to_string()
    }
}

impl fmt::Display for GcliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gcli: {}: {}: {}: {}",
            self.verb,
            self.level.as_str(),
            self.code.as_str(),
            self.message
        )
    }
}

pub type GcliResult<T> = Result<T, GcliError>;
