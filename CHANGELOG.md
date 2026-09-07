# Changelog

Format follows HashiCorp's real, established convention (chosen because it
separates upgrade-impacting changes from routine ones, which matters most
for a tool real scripts and agent-generated commands depend on):

- **NOTES** — anything with real upgrade impact: breaking changes,
  deprecations, behavior changes worth reading before you upgrade.
- **FEATURES** — major new capability (a new verb, a new primitive).
- **IMPROVEMENTS** — smaller additions/refinements to something that
  already existed.
- **BUG FIXES** — a real fix, no interface change.

See [`VERSIONING.md`](VERSIONING.md) for what actually triggers a
MAJOR/MINOR/PATCH bump.

## v0.1.0 — unreleased

**NOTES**
- Pre-1.0: the grammar and verb catalogue are still being proven against
  real usage — breaking changes may still happen between `0.x` releases
  without a full deprecation cycle. The
  staged deprecation guarantee in `VERSIONING.md` takes effect at `1.0.0`.

**FEATURES**
- Initial workspace scaffold: `gcli` (binary) + `gcli-core` (grammar/verb
  library).
- Real grammar plumbing: POSIX-shaped argv parsing, `--json` machine-
  readable output contract, the `gcli: <verb>: <LEVEL>: <CODE>: <message>`
  stderr format with sysexits.h-style numeric exit codes, stdin/file input
  resolution for piping.
- First real verb: `gcli info <file.geojson>` — real GeoJSON parsing,
  geometry-type/feature-count/bbox/CRS reporting.
