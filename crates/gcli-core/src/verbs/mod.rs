//! One real module per verb (docs/UNIX_grammar.md §Part 3 — "one job per
//! verb", never a shared `--type` mega-verb). `gcli-core` owns the real
//! logic; the `gcli` binary crate owns only argv parsing and I/O plumbing.

pub mod info;
