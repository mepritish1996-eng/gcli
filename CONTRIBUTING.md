# Contributing

## Before adding a verb or a flag

A new verb or flag needs to actually fit the existing grammar — in
particular, check whether what you're adding is secretly a hidden
`--type` fork of something that already exists before adding a new
command (see the existing verbs in `crates/gcli-core/src/verbs/` for the
pattern). Also worth cross-checking against
[clig.dev](https://clig.dev/) (Command Line Interface Guidelines) — a
real, community-maintained reference this project deliberately follows.
Open an issue first for anything nontrivial, using the feature/verb
request template, so the shape can be discussed before code is written.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) — `feat:`,
`fix:`, `docs:`, `refactor:`, with `BREAKING CHANGE:` in the footer where
relevant. This isn't a style preference — the version bump for a release is
derived from this history (see [`VERSIONING.md`](VERSIONING.md)), so an
inaccurate commit type produces an inaccurate release.

## Building & testing locally

```
cargo build
cargo test
cargo run -- info path/to/data.geojson
```

## Code shape

- `crates/gcli` stays a thin shell — argv parsing and I/O plumbing only.
  Real verb logic belongs in `crates/gcli-core`, one module per verb under
  `src/verbs/`.
- Every verb returns `GcliResult<T>` (`crates/gcli-core/src/error.rs`) —
  never a bare `String` error or a panic for an expected failure (bad
  input, missing file, unsupported format). Panics are for genuine bugs
  only.
- New error cases use the existing `ErrorCode` vocabulary
  (`E_INPUT_NOT_FOUND`, `E_INVALID_GEOMETRY`, etc.) where one already fits.
  Adding a new code is fine when none of the existing ones honestly
  describe the failure — don't force-fit one that doesn't.

## Telemetry

`gcli` collects no telemetry as of `v0.1.0` — a deliberate choice, not an
oversight (see `README.md`). If this ever changes, it needs its own
explicit, documented, opt-out-by-default decision — not something added
quietly inside an unrelated change.
