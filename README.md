# gcli

**The single command-line tool for GIS. Everything included.**

![status](https://img.shields.io/badge/status-early%20%2F%20v0.1.0-orange)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

GIS has always meant assembling a pile of separate, incompatible tools —
different commands, different syntax, nothing that pipes into anything
else, each one with its own install story. `gcli` is one real, Unix-shaped
command language for the whole workflow — data, processing, and rendering —
with everything it needs already included. Install it once; there's
nothing else to assemble.

```sh
$ gcli info rivers.geojson
format:          GeoJSON
crs:             EPSG:4326 (WGS84, per RFC 7946 default)
feature count:   2
geometry types:  LineString
bbox:            [77, 24, 88, 30.5]

$ gcli --json info rivers.geojson
{"format":"GeoJSON","geometry_types":["LineString"],"feature_count":2,"bbox":[77.0,24.0,88.0,30.5],"crs":"EPSG:4326 (WGS84, per RFC 7946 default)"}

$ cat rivers.geojson | gcli info    # reads stdin — every verb pipes
```

## Why

Every real GIS platform — QGIS, ArcGIS, any serious pipeline — is the same
three things underneath: **data**, **processing**, and rendering (which is
really just processing whose output happens to be visual). Nothing in that
space was ever built to compose — each real GIS operation is a genuinely
different *shape* of computation, and nobody defined one language spanning
all of them. `gcli` does, and follows real, established convention wherever
one already exists rather than inventing new syntax: SQL for predicates
(`--where`), CSS's `color()` function for color values, real streaming
formats for piping, real POSIX argument conventions for flags. One
consistent grammar, one install, no separate toolchain to assemble.

## Install

Real, current option today — build from source:

```sh
git clone <this-repo>
cd gcli
cargo build --release
# binary at target/release/gcli(.exe)
```

Package-manager distribution (conda-forge, winget, Scoop, npm) is planned,
not live yet — one install, everything `gcli` needs included, no separate
toolchain to set up first.

## Status

Early, honestly: one real verb (`info`, GeoJSON only) proves the grammar
end to end — argv parsing, `--json` machine-readable output, stdin piping,
the real `gcli: <verb>: <LEVEL>: <CODE>: <message>` stderr contract with
sysexits.h-style exit codes. Real processing isn't wired up yet — that's
the next real milestone. Full verb catalogue, the `port` system (QGIS/
ArcGIS/remote-DB plugins), the AI branch, and the native renderer are all
designed and on the roadmap, not yet built. See
[`CHANGELOG.md`](CHANGELOG.md) for what's actually shipped.

## Contributing

Real contributions welcome — start with [`CONTRIBUTING.md`](CONTRIBUTING.md)
(commit conventions, code shape, how to build/test). Also cross-check
against [clig.dev](https://clig.dev/) (Command Line Interface Guidelines),
a real, community-maintained reference this project deliberately follows.

## Project governance

- [`VERSIONING.md`](VERSIONING.md) — semver policy and the real, staged
  deprecation lifecycle (nothing is ever silently removed).
- [`CHANGELOG.md`](CHANGELOG.md) — what changed, release by release.
- [`SECURITY.md`](SECURITY.md) — how to report a real vulnerability.
- **License**: dual [MIT](LICENSE-MIT) / [Apache-2.0](LICENSE-APACHE),
  the standard Rust-ecosystem convention — use either.
- **Telemetry**: none, as of `v0.1.0`. A deliberate choice, not an
  oversight — `gcli` handles real, often-sensitive geospatial data
  (property boundaries, government/infrastructure data), and real
  precedent shows silently-added CLI telemetry causes real user backlash.
  If this ever changes, it gets its own explicit, documented,
  opt-out-by-default decision (see `CONTRIBUTING.md`), never a quiet
  addition inside an unrelated change.

## Workspace layout

```
crates/
  gcli/         binary crate — argv parsing + I/O plumbing only
  gcli-core/    the real grammar/dispatch library — verbs, errors, I/O
```
