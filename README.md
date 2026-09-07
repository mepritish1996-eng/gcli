# gcli

**One grammar for GDAL, PostGIS, PDAL, pgRouting, and OSRM.**

![status](https://img.shields.io/badge/status-early%20%2F%20v0.1.0-orange)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

GIS has five real, excellent, decades-proven engines — and five completely
different, incompatible ways of talking to them. `ogr2ogr -t_srs`,
`ST_Transform(geom, srid)`, `pgr_dijkstra(...)`, an OSRM HTTP endpoint — no
shared grammar, no way to pipe one into the next. `gcli` is one real,
Unix-shaped command language sitting in front of all of them — the engines
underneath are swappable implementation details; the grammar is the one
thing that stays constant.

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

Every real GIS platform — QGIS, ArcGIS, a raw PostGIS+GDAL pipeline — is
the same three things underneath: **data**, **processing**, and rendering
(which is really just processing whose output happens to be visual). None
of the tools that actually do the processing were ever built to compose
with each other, because each solves a genuinely different *shape* of
computation — a stateless transform (`buffer`, `reproject`), a build-once/
query-many index (a routing graph), or a relational query (SQL) — and
nobody ever defined one abstraction spanning all three. `gcli` does, and
follows real, established convention everywhere one already exists rather
than inventing new syntax: SQL for predicates (`--where`), CSS's `color()`
function for color values, GDAL's own GeoJSONSeq for streaming, real POSIX
argument conventions for flags.

## Install

Real, current option today — build from source:

```sh
git clone <this-repo>
cd gcli
cargo build --release
# binary at target/release/gcli(.exe)
```

Package-manager distribution (conda-forge, winget, Scoop, npm) is planned,
not live yet — conda-forge is the real priority, since that's already how
most real GIS users get GDAL onto their machine.

## Status

Early, honestly: one real verb (`info`, GeoJSON only) proves the grammar
end to end — argv parsing, `--json` machine-readable output, stdin piping,
the real `gcli: <verb>: <LEVEL>: <CODE>: <message>` stderr contract with
sysexits.h-style exit codes. No engine dispatch (GDAL/PostGIS/etc.) wired
yet — that's the next real milestone. Full verb catalogue, the `port`
system (QGIS/ArcGIS/remote-DB plugins), the AI branch, and the native
renderer are all designed and on the roadmap, not yet built. See
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
