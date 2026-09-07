//! gcli-core — the real grammar/dispatch layer, independent of the `gcli`
//! binary crate's argv parsing. Spec: `docs/UNIX_grammar.md` in this repo's
//! root — read that before adding a verb or a flag; this crate exists to
//! implement that spec, not to reinvent it ad hoc per verb.
//!
//! Scope note (2026, v1 scaffold): this crate implements the grammar
//! contract (error/exit-code format, stdin/file input resolution) and one
//! real verb (`info`, GeoJSON-only) to prove the pattern end to end. Real
//! engine dispatch (GDAL/PDAL/PostGIS/pgRouting/OSRM underneath the rest of
//! the verb catalogue) is deliberately not wired yet — see the verb
//! catalogue in the grammar doc for the full intended surface.

pub mod error;
pub mod io;
pub mod verbs;

pub use error::{ErrorCode, GcliError, GcliResult, Level};
