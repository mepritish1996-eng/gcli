# Security policy

`gcli` shells out to real external engines (GDAL/PDAL/PostGIS/pgRouting/
OSRM) and reads/writes real user data and files — a real vulnerability here
(argument injection when building a subprocess call, path traversal when
resolving a positional-arg file, unsafe deserialization of untrusted input)
has real consequences, not just a bug report.

## Reporting a vulnerability

Do **not** open a public GitHub issue for a security concern. Report it
privately instead (via GitHub's private security advisory feature on this
repo once it's hosted there, or directly to the maintainer) so a fix can
ship before the issue is public.

Include: what version, what command/input triggers it, and what the real,
concrete impact is (not just "this looks unsafe" — what can an attacker
actually do).

## Scope

Real, in-scope concerns for a tool like this:
- Command/argument injection when `gcli` constructs a call to an external
  engine (GDAL, etc.) from user/agent-supplied input.
- Path traversal or symlink-following when resolving a positional-arg file
  path.
- Unsafe parsing of untrusted GeoJSON/data files (a malformed or malicious
  input file should produce a real `E_INVALID_GEOMETRY`/`E_UNSUPPORTED_FORMAT`
  error, never a crash or memory-safety issue).
- Anything in the eventual `port` mechanism (§Part 4-adjacent design,
  companion plugins talking to QGIS/ArcGIS/etc. over a local channel) that
  could let an untrusted process on the same machine send commands it
  shouldn't be able to.

## Response

No formal SLA yet at this pre-1.0 stage — real, good-faith effort to
acknowledge quickly and fix before any public disclosure. This gets a real
SLA once `gcli` has real external users depending on it.
