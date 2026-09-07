# Unix CLI Grammar — Reference for `gcli`

Working notes on real Unix/CLI conventions, built up lesson by lesson, kept here as
the actual grammar spec `gcli` (the hand-built universal GIS CLI) must follow.
The point of following these conventions strictly: a human at a terminal and the
agent driving `gcli` through its `bash` tool should both be able to predict how
every verb behaves without memorizing it case by case, because it's the same
grammar every other real Unix tool already uses.

## Part 1 — Fundamentals

### 1. The basic shape of a command

Every command, no matter how complicated it looks, has the same three parts:

```
program-name   [optional settings]   [the thing it acts on]
```

Example: `cp file1.txt file2.txt` — copies `file1.txt` to `file2.txt`.
- `cp` — the **program name**. Which tool is running.
- `file1.txt file2.txt` — the **positional arguments**, the things it acts on.
  Order matters here — `cp file2.txt file1.txt` does the opposite.

A fancier one: `cp -r Documents Backup` — copies an entire folder.
- `-r` — an **optional setting** ("flag"). Changes *how* the tool behaves.
  `-r` means "recursive" — go inside folders too. Without it, `cp` refuses to
  copy a whole folder.

In plain words: *run this program, here are some optional switches to change
its behavior, here are the actual things to do it to.* That's the entire shape
— everything below is refinement on top of these three parts.

### 2. Short flags vs. long flags

`-r` is a **short flag** (one dash, one letter). `--recursive` is the **long
flag** meaning the exact same thing. Both exist for different reasons:

- **Short flags** are for typing fast — convenience for someone who already
  knows the tool by heart.
- **Long flags** are self-documenting — clear to someone reading a script, or
  to someone who doesn't have the tool memorized. `-r` alone is a guess
  (recursive? reverse? something else?) unless you already know; `--recursive`
  tells you outright.

Short flags can be combined: `-la` means `-l -a`. Long flags never squash
together like that — `--long --all` always stays separate.

**For `gcli`**: since both a human and the agent generate these commands, long
forms matter more than usual — there's no "muscle memory" benefit for a model,
only a clarity benefit for whoever reads the command afterward in a log.

### 3. Switch flags vs. value flags

Two different kinds of flags:

- **Switch flag (boolean)** — no value, just on/off by presence or absence.
  `-r` is one: either it's there (ON) or it isn't (OFF). There's no `-r yes`.
- **Value flag** — needs an actual piece of information attached. Example:
  `gcli buffer --dist 500` — `--dist` is meaningless without a number.

Two equally valid ways to write a value flag: `--dist 500` (space-separated)
or `--dist=500` (equals-separated). Both mean the same thing; `=` is sometimes
preferred in scripts because it visually glues flag and value into one token.

**Range values** — when a single flag's value is a range rather than one
number, two real patterns exist:
1. **One flag, multiple values right after it**: `gdalwarp -te xmin ymin xmax
   ymax` — the flag consumes the next 4 numbers as one bounding box.
2. **One flag, one delimited string**: `--dates 2020-01-01:2023-12-31` or
   `--zoom 5-12` — a single string the tool splits itself.

Pick one convention and apply it consistently across every `gcli` verb (e.g.
always colon-separated ranges) so nobody has to guess per-command.

**For `gcli`**: every verb must clearly declare, per flag, whether it's a
switch or a value, and if a value, what kind (number? CRS code? file path?
range?). That declaration is what lets the agent generate a correct command
instead of guessing, and what a `--help` output should communicate to a human.

### 4. Telling flags apart from positional data, and `--`

The rule almost every CLI follows: **anything starting with `-` is a flag;
anything that doesn't is positional data** (a file, a name, a number). The
parser sorts left to right purely on that leading-dash signal. Order between
a flag and positional data mostly doesn't matter — `cp Documents Backup -r`
usually works the same as `cp -r Documents Backup` (value flags are the one
exception: `--dist` and its number must stay adjacent).

**The gotcha this creates**: a file genuinely named `-r`, or starting with a
dash, gets misread as a flag. The fix, decades old: a bare `--` means
*"everything past this point is positional data, no matter what it looks
like."* Example: `rm -- -myfile.txt` deletes a file actually named
`-myfile.txt` instead of trying to parse it as a flag.

**For `gcli`**: geospatial filenames (agent-generated temp names, paths with
dashes) can genuinely trigger this — `gcli` supports `--` from day one, same
convention every real Unix tool honors.

### 5. stdin, stdout, and stderr

Every running program has three invisible tubes connected by default:

- **stdin** — where it reads input from (keyboard by default; can be
  redirected from a file, or from another program's output — the basis of
  piping).
- **stdout** — where it prints its real, intended result (screen by default).
- **stderr** — a *separate* tube just for errors/warnings/progress (also
  prints to the screen by default, which is why it's easy to miss it's a
  different tube — until you redirect).

Why two output tubes: they behave differently under redirection. `gcli
reproject data.geojson --to EPSG:3857 > out.geojson` sends stdout into the
file; a progress message or warning goes to stderr instead, which still shows
on-screen rather than corrupting the saved file. If both were mixed into one
stream, a redirected file could get polluted with log noise, and an error
could vanish into a file you never look at.

**For `gcli`**: the agent reads command output through its bash tool to decide
what happened. Strict separation — real result on stdout, everything else on
stderr — is what makes it safe for the agent to trust "whatever's on stdout is
the real thing," with no filtering needed.

### 6. Piping

`|` takes the **stdout of the command on its left** and feeds it as the
**stdin of the command on its right**, with nothing touching disk in between.

```
cat data.geojson | gcli reproject --to EPSG:3857 | gcli buffer --dist 500 > out.geojson
```

- `cat data.geojson` reads the file, writes it to stdout.
- First `|` hands that to `gcli reproject`'s stdin.
- `gcli reproject` reads stdin (no filename given), reprojects, writes to
  stdout.
- Second `|` hands that to `gcli buffer`'s stdin.
- `gcli buffer` reads it, buffers, writes the final result to stdout.
- `> out.geojson` saves that final stdout to a real file.

Three simple, single-purpose tools, chained by the shell — none of them need
to know the others exist. This is the real Unix philosophy: not "one tool
does everything," but "small tools, composed."

**The hard requirement this places on every `gcli` verb**: it must read from
stdin when no input file is given, and write to stdout by default. A tool that
only reads/writes named files is a dead end in a pipe chain — it can't
participate. This is why every `gcli` verb speaks **GeoJSONSeq** (newline-
delimited GeoJSON — one Feature per line, already a real GDAL streaming
format) on stdin/stdout by default: it's what makes piping actually work for
structured geodata, the same way plain text lines make `cat | grep | sort`
work for text.

**For the agent**: asked to "reproject and buffer this layer," it doesn't need
a combined tool call — it constructs one pipeline string and runs it once
through its existing bash driver. New combinations of behavior come free from
composition, without a new command needed for every combination someone asks
for.

### 7. Exit codes

When a program finishes, it hands back one small number to whatever called it
— the **exit code**. Invisible by default; `echo $?` right after a command
shows the exit code of whatever just ran.

**The universal rule**: `0` = success, always, no exceptions. Any non-zero
number = failure (different tools use different non-zero numbers for
different specific failures, but zero-vs-nonzero is universal).

Why a number, separate from stdout/stderr text: a script or the agent needs to
check "did that work?" programmatically, without guessing from printed words
— scanning for the word "error" is unreliable if the real data could contain
that word incidentally.

This also drives command chaining: `&&` runs the next command only if the
previous one exited 0; `||` runs the next command only if the previous one
failed.

**The connection worth noticing**: this is the exact same concept as the
platform's own `tool_call` status (`'completed' | 'error'`) already built into
`AgentContext.tsx`'s `MessagePart` type. Exit code 0 = `completed`, non-zero =
`error` — Unix had this idea decades before "agent tool status" was a phrase.

**For the agent**: it must check the real exit code to decide if a pipeline
step worked, not just glance at whether stdout looks non-empty — an empty-but-
successful result and a silently-failed command can look identical on stdout
alone.

## Part 2 — Advanced Conventions

### 8. `--help` and `--version`

Every real Unix tool supports `--help` (usually also `-h`) and `--version`.
This isn't a nicety — it's load-bearing, because both humans and the agent
need a reliable way to discover a verb's exact contract without reading source
code. A well-formed `--help` output follows a predictable shape:

```
USAGE:
    gcli buffer [OPTIONS] <INPUT>

OPTIONS:
    --dist <METERS>     Buffer distance in meters (required)
    --crs <EPSG>        Output CRS (default: same as input)
    --json              Machine-readable output
    -h, --help          Show this help

EXAMPLES:
    gcli buffer --dist 500 data.geojson
```

`--version` prints the tool's version string alone, nothing else — scripts
depend on this being exactly one line, parseable without ceremony.

**For `gcli`**: every verb needs this shape identically. It's also the
mechanism that lets the agent self-discover a verb's flags at runtime — asking
"what does `gcli route` actually accept?" via `--help` instead of relying on a
possibly-stale hardcoded tool description.

### 9. Human-readable output vs. machine-readable output

A tool's *default* output is usually formatted for a person to read — column-
aligned, maybe colored, sometimes summarized. But that same tool typically
offers a `--json` (or `-f json`) flag that switches to strict, parseable
structured output instead — no color codes, no summarizing, no formatting
flourishes, just data.

These are not the same output with cosmetic differences — they're two
different contracts. A human-readable table can silently change column order
or wording between versions without breaking anything; a `--json` contract is
a promise that a script parsing it won't break.

**For `gcli`**: this matters more than usual because the agent is the CLI's
heaviest consumer — it should always request `--json` and never attempt to
screen-scrape human-formatted text. The human-readable mode still matters for
you, testing verbs directly at a terminal.

### 10. Verbosity flags

`-v`, `-vv`, `-vvv` (each repetition increases detail) or `--verbose`/
`--quiet` control how much goes to stderr — remember, real results always
stay on stdout regardless of verbosity; verbosity only ever affects the
stderr side channel. Default verbosity should be quiet enough that a script
piping stdout doesn't drown in noise on the screen, but `-v` should be enough
to diagnose what a stuck or failed command was actually doing.

**For `gcli`**: relevant to actual production debugging — a long-running
geoprocessing job invoked by the agent should be silent by default and only
verbose when explicitly asked, especially since stderr output is still
visible to a human watching a terminal even while stdout is being piped
onward.

### 11. `--dry-run`

For any verb that *mutates* something (writes a file, overwrites data, runs a
paid/expensive operation), a `--dry-run` flag should describe what *would*
happen without actually doing it — same inputs validated, same plan printed,
zero side effects. This matters doubly for an agent-driven tool: it gives the
agent (or the human reviewing an agent's proposed plan) a safe way to preview
a mutating command before actually approving it — directly analogous to this
platform's own Plan Mode, which already exists specifically to preview
mutations before they run.

### 12. Config file, environment variable, and flag precedence

Real tools accept configuration from three layers, with a fixed precedence
order: an explicit **flag** on the command line wins over an **environment
variable**, which wins over a **config file**, which wins over a **built-in
default**. Example: a default output CRS might be set in a project config
file, overridden by an environment variable for one shell session, overridden
again by an explicit `--crs` flag for one specific call.

**For `gcli`**: useful for project-level defaults (a project's working CRS,
its default output directory) without forcing every single invocation to
restate them — while still letting any individual call override anything.

### 13. Global flags vs. subcommand-specific flags

In a multi-subcommand tool (`gcli <verb> ...`), some flags apply no matter
which verb runs (`--config`, `--verbose`, `--json`) — these are **global
flags** and conventionally go *before* the verb name. Flags specific to one
verb's behavior (`--dist` for `buffer`) go *after* the verb name. Example:
`gcli --json buffer --dist 500 data.geojson` — `--json` is global (affects
every verb's output format), `--dist` only means something to `buffer`.

**For `gcli`**: keeping this distinction clean means adding a new verb never
requires re-deciding how output format or config-file loading works — those
stay handled once, globally, above the verb layer.

### 14. Signal handling (Ctrl+C)

Pressing Ctrl+C sends a signal called `SIGINT` to the running program. A
well-behaved long-running tool catches this and shuts down *cleanly* — closing
open files, not leaving partial/corrupt output behind — rather than being
killed mid-write. For a tool that can be interrupted mid-operation (a large
reprojection, a routing graph build), handling `SIGINT` gracefully is not
optional politeness, it's what prevents a half-written output file from being
mistaken for a complete one afterward.

**For `gcli`**: directly relevant to this platform's own job-cancellation
feature (compute-governance already supports cancelling a running job) — a
`gcli` verb invoked as part of a long job needs to behave the same way when
cancelled: clean up, don't leave corrupt partial output silently sitting where
a complete result is expected.

### 15. Idempotency and safe re-running

Running the same command twice with the same inputs should behave
predictably — either it produces the identical result again (safe to re-run,
e.g. a pure `reproject`), or it clearly refuses to silently clobber existing
output without an explicit `--force`/`--overwrite` flag. A command that
silently does something different the second time (partial overwrite,
appending instead of replacing) is a common, dangerous source of bugs —
especially for an agent that might legitimately retry a failed step, and needs
to trust that retrying doesn't quietly corrupt state from a partially-
succeeded first attempt.

### 16. Standardized stderr

An exit code says *whether* a verb failed. It says nothing about *why*. Left
unstandardized, stderr is just free-text prose — which puts the agent right
back to screen-scraping to explain a failure, the exact problem `--json`
already solves on the stdout side. stderr needs the same discipline.

**Fixed line shape, default (human-readable) mode:**

```
gcli: <verb>: <LEVEL>: <CODE>: <human message>
```

Example: `gcli: reproject: ERROR: E_CRS_UNKNOWN: "EPSG:99999" is not a
recognized CRS`

- **`<verb>`** — which subcommand emitted this line, so a merged stderr stream
  (several verbs piped together, all writing to the same terminal) stays
  traceable back to its source. Same convention real tools already use —
  `grep: file.txt: No such file or directory`.
- **`<LEVEL>`** — `ERROR` (fatal, exit code will be non-zero), `WARN`
  (non-fatal — e.g. "3 features had invalid geometry, skipped" — the process
  still succeeds and exits 0), `INFO` (progress, shown only at `-v`).
- **`<CODE>`** — a small, fixed, shared vocabulary across every verb:
  `E_INPUT_NOT_FOUND`, `E_INVALID_GEOMETRY`, `E_UNSUPPORTED_FORMAT`,
  `E_CRS_UNKNOWN`, `E_PERMISSION_DENIED`, `E_TIMEOUT`, `E_INTERNAL` — plus
  verb-specific codes only where genuinely needed. The code is what a caller
  (the agent, or future retry logic) branches on programmatically; the
  message after it is for a human reading the terminal.

**Under `--json`**, stderr mirrors the same discipline as stdout instead of
falling back to prose: one JSON object per line, e.g.
`{"level":"ERROR","code":"E_CRS_UNKNOWN","verb":"reproject","message":"..."}`
— so a `--json` caller never has to switch parsing strategy between
structured stdout and free-text stderr mid-command.

Why colons specifically, not some other separator: it's the existing Unix
convention for chaining context before a message (`grep: file.txt: No such
file or directory`) — easy for a human to scan left to right, and easy to
split on programmatically without needing a JSON parser for the plain-text
default mode. Not arbitrary; copying what already works.

Without a fixed code vocabulary, the only thing anyone can ever say about a
failure is "it failed." With codes, the agent can react differently by class
of failure — retry on `E_TIMEOUT`, don't retry on `E_INVALID_GEOMETRY`, ask
the user to fix the CRS on `E_CRS_UNKNOWN` — instead of guessing from prose.

---

**Design decisions already locked in for `gcli` from these lessons:**
- Every verb: reads stdin / writes stdout when no file is named (piping works
  by default).
- Wire format between verbs: GeoJSONSeq (streaming, one Feature per line).
- Every verb: real exit codes (0/non-zero), strict stdout/stderr separation.
- Every verb: `--json` machine-readable mode, alongside a human-readable
  default.
- Long-flag-first convention in agent-generated commands (clarity over typing
  speed, since the agent has no "muscle memory" benefit).
- Consistent `--` support for positional data that looks like a flag.
- stderr follows `gcli: <verb>: <LEVEL>: <CODE>: <message>`, with a fixed,
  shared error-code vocabulary and a JSON-lines form under `--json`.

## Part 3 — The verb catalogue

Two real pillars, not three — Rendering collapsed into Processing, since every
"rendering" verb has the same input-data → transform → output-artifact shape
as any other processing verb; the only thing that differs is the output
format and who consumes it next (see the Data/Processing/Rendering
discussion). Each verb below passed the self-critique check: does it secretly
fork into different behavior based on a hidden `--type`? If so, it was split
into separate verbs instead of staying one with a mode flag — `convert` split
off raster conversion into `warp`, `join` split into `join`/`spatial-join`,
`terrain` split into `slope`/`aspect`/`hillshade`, and `tile` split into
`tile`/`raster-tile`.

### Data

| Verb | Job |
|---|---|
| `info` | Inspect metadata — format, CRS, extent, schema, feature/band count. Output naturally varies by input type, same as Unix's `file`, but it's one job: describe what this dataset is. |
| `convert` | Vector format conversion (Shapefile → GeoJSON, etc.). Vector-only — raster conversion lives under `warp`, different engine, different flags entirely. |
| `validate` | Geometry/schema sanity check — catches bad input before it silently breaks a later Processing step. |

### Processing — vector geometry

`reproject`, `buffer`, `clip`, `intersect`, `union`, `difference`, `dissolve`,
`simplify`, `centroid`, `merge` — each a single, real spatial operation:
input geometry in, transformed geometry out.

### Processing — attributes / tabular

| Verb | Job |
|---|---|
| `filter` | Select features by an attribute predicate (e.g. `pop > 1000`). |
| `join` | Attribute join — match rows by a key column, no geometry involved. |
| `spatial-join` | Match rows by a spatial predicate (e.g. "point is within polygon") — split from `join`, genuinely different algorithm and required inputs. |
| `aggregate` | Group by attribute, summarize (count/sum/mean per group). |

### Processing — raster

| Verb | Job |
|---|---|
| `warp` | Reproject/resample/format-convert a raster (the `gdalwarp`/`gdal_translate` territory). |
| `mosaic` | Combine multiple rasters into one. |
| `calc` | Band math. |
| `slope` | Terrain slope from a DEM. |
| `aspect` | Terrain aspect from a DEM. |
| `hillshade` | Terrain hillshade from a DEM. |
| `zonal-stats` | Statistics of a raster within vector polygons. |
| `sample` | Sample raster values at given points. |

### Processing — point cloud

`pc-info`, `pc-filter` (by classification/ground/return-number — one
predicate-in, matching-subset-out shape regardless of which field), `pc-to-raster`
(DEM/DSM generation), `pc-to-mesh`.

### Processing — network / routing

`route` (shortest path), `isochrone` (reachability polygon), `matrix`
(travel time/distance between many points), `network-build` (prepare a
routing graph from road data — required before `route`/`isochrone`/`matrix`
can run against it).

### Processing — spatial indexing

`h3`, `s2` — hex/cell indexing of points or polygons.

### Processing — rendering-as-processing

| Verb | Job |
|---|---|
| `style` | Apply a general style expression (opacity, line width, data-driven expressions), produce styled output. |
| `colour` | Set fill/stroke color, optionally scoped by `--where` (only matched features change; everything else keeps its existing color untouched). Deliberately narrow and separate from `style` — same relationship as `mkdir` staying its own tool even where a more general filesystem tool also exists. |
| `tile` | Generate vector tiles (MVT/PMTiles via Tippecanoe). |
| `raster-tile` | Generate raster/image tile pyramids — split from `tile`, different engine (gdal2tiles), different output. |
| `render` | Export a static image (PNG) of styled data. |

### Dispatch — not Data, not Processing

| Verb | Job |
|---|---|
| `open` | Hand a dataset off to the actual renderer (the browser/map view) to display. No transform, no artifact produced — analogous to Unix's `open`/`xdg-open`, and the CLI-callable equivalent of this platform's existing "click to load onto the map" feature. |

`open` doesn't fit either pillar because it doesn't touch data at all — it's a
UI action, not a data operation. Worth keeping as its own small category
rather than forcing it into Processing just to preserve a clean two-pillar
story.

## Part 4 — Platform boundary and value conventions

### 17. `gcli`'s output contract vs. the platform's representation pipeline

The platform (this web app today, potentially a desktop app tomorrow)
distinguishes **raw data** from its **representation** — a lightweight,
tiled, browser-safe render format (PMTiles or equivalent) that's what
actually loads onto the map, so an arbitrarily large dataset never risks
OOM-ing the browser. This distinction is entirely platform-internal: neither
`gcli` nor the agent ever knows representations exist.

The full cascade when a geometry-altering verb runs: its stdout becomes a
**new raw data asset** → the platform automatically derives a **new
representation** from that raw data → that representation auto-loads onto
the map. All three of these are platform-only pipeline stages, strictly
**downstream of a successful `gcli` exit** — `gcli` never participates in
them and has no flag or concept for any of it.

This is why `gcli`'s own contract stays simple regardless of how elaborate
the platform's pipeline gets:

- **`gcli` never mutates its input in place.** Every verb reads from stdin/a
  file and writes a brand-new result to stdout — this was already guaranteed
  by the pure input→transform→output shape from Part 1, nothing new required.
- **`gcli`'s stdout is always raw geometry (GeoJSONSeq), never a
  representation.** Tiling/representation-generation begins only after
  `gcli` has already exited 0 with real geometry on stdout.
- **`gcli` has no concept of "layer," "project," "asset," or "representation"
  at all.** Deciding what to do with a verb's output (register it as a new
  layer, discard it, pipe it into another verb) is 100% the calling
  platform's policy, never the CLI's.
- **Exception**: verbs that only change *how* something is drawn without
  touching geometry (`style`, `colour`) do **not** trigger this cascade —
  no new raw data or representation is created, since nothing about the
  underlying data changed, only how it's rendered.

Why this is the right boundary for a future desktop migration: a desktop app
might not need tiling at all (a native engine could render raw geometry
directly without a browser's OOM risk) — its representation step could be a
no-op or something else entirely. That swap happens **entirely in the
platform-side pipeline that consumes `gcli`'s stdout** — `gcli` itself never
changes, because it was never aware representations existed in the first
place.

### 18. Color values — one canonical annotation across all color spaces

Real color spaces (RGB, HSL, OKLCH, etc.) are different mathematical models
for the same underlying color, each with its own notation
(`rgb(255,0,0)`, `hsl(0,100%,50%)`, `oklch(0.63 0.26 29)`). Hex (`#FF0000`) is
not a separate color space — it's just RGB's numbers written in base-16, and
it can't natively express spaces like OKLCH or wide-gamut spaces
(`display-p3`) without a lossy conversion into legacy sRGB first.

**`gcli` standardizes on CSS's generic function syntax as the one canonical
color annotation**: `color(<space> <values>)` — e.g. `color(srgb 1 0 0)` or
`color(oklch 0.63 0.26 29)` for the same red. One consistent wrapper shape,
parametrized by which space you're in, rather than a different function name
(or hex's RGB-only notation) per space. Any color flag (`--fill`, future
`--stroke`, etc.) accepts this form as the canonical value; `--json` output
always reports color values in this form too.
# Unix CLI Grammar — Reference for `gcli`

Working notes on real Unix/CLI conventions, built up lesson by lesson, kept here as
the actual grammar spec `gcli` (the hand-built universal GIS CLI) must follow.
The point of following these conventions strictly: a human at a terminal and the
agent driving `gcli` through its `bash` tool should both be able to predict how
every verb behaves without memorizing it case by case, because it's the same
grammar every other real Unix tool already uses.

## Part 1 — Fundamentals

### 1. The basic shape of a command

Every command, no matter how complicated it looks, has the same three parts:

```
program-name   [optional settings]   [the thing it acts on]
```

Example: `cp file1.txt file2.txt` — copies `file1.txt` to `file2.txt`.
- `cp` — the **program name**. Which tool is running.
- `file1.txt file2.txt` — the **positional arguments**, the things it acts on.
  Order matters here — `cp file2.txt file1.txt` does the opposite.

A fancier one: `cp -r Documents Backup` — copies an entire folder.
- `-r` — an **optional setting** ("flag"). Changes *how* the tool behaves.
  `-r` means "recursive" — go inside folders too. Without it, `cp` refuses to
  copy a whole folder.

In plain words: *run this program, here are some optional switches to change
its behavior, here are the actual things to do it to.* That's the entire shape
— everything below is refinement on top of these three parts.

### 2. Short flags vs. long flags

`-r` is a **short flag** (one dash, one letter). `--recursive` is the **long
flag** meaning the exact same thing. Both exist for different reasons:

- **Short flags** are for typing fast — convenience for someone who already
  knows the tool by heart.
- **Long flags** are self-documenting — clear to someone reading a script, or
  to someone who doesn't have the tool memorized. `-r` alone is a guess
  (recursive? reverse? something else?) unless you already know; `--recursive`
  tells you outright.

Short flags can be combined: `-la` means `-l -a`. Long flags never squash
together like that — `--long --all` always stays separate.

**For `gcli`**: since both a human and the agent generate these commands, long
forms matter more than usual — there's no "muscle memory" benefit for a model,
only a clarity benefit for whoever reads the command afterward in a log.

### 3. Switch flags vs. value flags

Two different kinds of flags:

- **Switch flag (boolean)** — no value, just on/off by presence or absence.
  `-r` is one: either it's there (ON) or it isn't (OFF). There's no `-r yes`.
- **Value flag** — needs an actual piece of information attached. Example:
  `gcli buffer --dist 500` — `--dist` is meaningless without a number.

Two equally valid ways to write a value flag: `--dist 500` (space-separated)
or `--dist=500` (equals-separated). Both mean the same thing; `=` is sometimes
preferred in scripts because it visually glues flag and value into one token.

**Range values** — when a single flag's value is a range rather than one
number, two real patterns exist:
1. **One flag, multiple values right after it**: `gdalwarp -te xmin ymin xmax
   ymax` — the flag consumes the next 4 numbers as one bounding box.
2. **One flag, one delimited string**: `--dates 2020-01-01:2023-12-31` or
   `--zoom 5-12` — a single string the tool splits itself.

Pick one convention and apply it consistently across every `gcli` verb (e.g.
always colon-separated ranges) so nobody has to guess per-command.

**For `gcli`**: every verb must clearly declare, per flag, whether it's a
switch or a value, and if a value, what kind (number? CRS code? file path?
range?). That declaration is what lets the agent generate a correct command
instead of guessing, and what a `--help` output should communicate to a human.

### 4. Telling flags apart from positional data, and `--`

The rule almost every CLI follows: **anything starting with `-` is a flag;
anything that doesn't is positional data** (a file, a name, a number). The
parser sorts left to right purely on that leading-dash signal. Order between
a flag and positional data mostly doesn't matter — `cp Documents Backup -r`
usually works the same as `cp -r Documents Backup` (value flags are the one
exception: `--dist` and its number must stay adjacent).

**The gotcha this creates**: a file genuinely named `-r`, or starting with a
dash, gets misread as a flag. The fix, decades old: a bare `--` means
*"everything past this point is positional data, no matter what it looks
like."* Example: `rm -- -myfile.txt` deletes a file actually named
`-myfile.txt` instead of trying to parse it as a flag.

**For `gcli`**: geospatial filenames (agent-generated temp names, paths with
dashes) can genuinely trigger this — `gcli` supports `--` from day one, same
convention every real Unix tool honors.

### 5. stdin, stdout, and stderr

Every running program has three invisible tubes connected by default:

- **stdin** — where it reads input from (keyboard by default; can be
  redirected from a file, or from another program's output — the basis of
  piping).
- **stdout** — where it prints its real, intended result (screen by default).
- **stderr** — a *separate* tube just for errors/warnings/progress (also
  prints to the screen by default, which is why it's easy to miss it's a
  different tube — until you redirect).

Why two output tubes: they behave differently under redirection. `gcli
reproject data.geojson --to EPSG:3857 > out.geojson` sends stdout into the
file; a progress message or warning goes to stderr instead, which still shows
on-screen rather than corrupting the saved file. If both were mixed into one
stream, a redirected file could get polluted with log noise, and an error
could vanish into a file you never look at.

**For `gcli`**: the agent reads command output through its bash tool to decide
what happened. Strict separation — real result on stdout, everything else on
stderr — is what makes it safe for the agent to trust "whatever's on stdout is
the real thing," with no filtering needed.

### 6. Piping

`|` takes the **stdout of the command on its left** and feeds it as the
**stdin of the command on its right**, with nothing touching disk in between.

```
cat data.geojson | gcli reproject --to EPSG:3857 | gcli buffer --dist 500 > out.geojson
```

- `cat data.geojson` reads the file, writes it to stdout.
- First `|` hands that to `gcli reproject`'s stdin.
- `gcli reproject` reads stdin (no filename given), reprojects, writes to
  stdout.
- Second `|` hands that to `gcli buffer`'s stdin.
- `gcli buffer` reads it, buffers, writes the final result to stdout.
- `> out.geojson` saves that final stdout to a real file.

Three simple, single-purpose tools, chained by the shell — none of them need
to know the others exist. This is the real Unix philosophy: not "one tool
does everything," but "small tools, composed."

**The hard requirement this places on every `gcli` verb**: it must read from
stdin when no input file is given, and write to stdout by default. A tool that
only reads/writes named files is a dead end in a pipe chain — it can't
participate. This is why every `gcli` verb speaks **GeoJSONSeq** (newline-
delimited GeoJSON — one Feature per line, already a real GDAL streaming
format) on stdin/stdout by default: it's what makes piping actually work for
structured geodata, the same way plain text lines make `cat | grep | sort`
work for text.

**For the agent**: asked to "reproject and buffer this layer," it doesn't need
a combined tool call — it constructs one pipeline string and runs it once
through its existing bash driver. New combinations of behavior come free from
composition, without a new command needed for every combination someone asks
for.

### 7. Exit codes

When a program finishes, it hands back one small number to whatever called it
— the **exit code**. Invisible by default; `echo $?` right after a command
shows the exit code of whatever just ran.

**The universal rule**: `0` = success, always, no exceptions. Any non-zero
number = failure (different tools use different non-zero numbers for
different specific failures, but zero-vs-nonzero is universal).

Why a number, separate from stdout/stderr text: a script or the agent needs to
check "did that work?" programmatically, without guessing from printed words
— scanning for the word "error" is unreliable if the real data could contain
that word incidentally.

This also drives command chaining: `&&` runs the next command only if the
previous one exited 0; `||` runs the next command only if the previous one
failed.

**The connection worth noticing**: this is the exact same concept as the
platform's own `tool_call` status (`'completed' | 'error'`) already built into
`AgentContext.tsx`'s `MessagePart` type. Exit code 0 = `completed`, non-zero =
`error` — Unix had this idea decades before "agent tool status" was a phrase.

**For the agent**: it must check the real exit code to decide if a pipeline
step worked, not just glance at whether stdout looks non-empty — an empty-but-
successful result and a silently-failed command can look identical on stdout
alone.

## Part 2 — Advanced Conventions

### 8. `--help` and `--version`

Every real Unix tool supports `--help` (usually also `-h`) and `--version`.
This isn't a nicety — it's load-bearing, because both humans and the agent
need a reliable way to discover a verb's exact contract without reading source
code. A well-formed `--help` output follows a predictable shape:

```
USAGE:
    gcli buffer [OPTIONS] <INPUT>

OPTIONS:
    --dist <METERS>     Buffer distance in meters (required)
    --crs <EPSG>        Output CRS (default: same as input)
    --json              Machine-readable output
    -h, --help          Show this help

EXAMPLES:
    gcli buffer --dist 500 data.geojson
```

`--version` prints the tool's version string alone, nothing else — scripts
depend on this being exactly one line, parseable without ceremony.

**For `gcli`**: every verb needs this shape identically. It's also the
mechanism that lets the agent self-discover a verb's flags at runtime — asking
"what does `gcli route` actually accept?" via `--help` instead of relying on a
possibly-stale hardcoded tool description.

### 9. Human-readable output vs. machine-readable output

A tool's *default* output is usually formatted for a person to read — column-
aligned, maybe colored, sometimes summarized. But that same tool typically
offers a `--json` (or `-f json`) flag that switches to strict, parseable
structured output instead — no color codes, no summarizing, no formatting
flourishes, just data.

These are not the same output with cosmetic differences — they're two
different contracts. A human-readable table can silently change column order
or wording between versions without breaking anything; a `--json` contract is
a promise that a script parsing it won't break.

**For `gcli`**: this matters more than usual because the agent is the CLI's
heaviest consumer — it should always request `--json` and never attempt to
screen-scrape human-formatted text. The human-readable mode still matters for
you, testing verbs directly at a terminal.

### 10. Verbosity flags

`-v`, `-vv`, `-vvv` (each repetition increases detail) or `--verbose`/
`--quiet` control how much goes to stderr — remember, real results always
stay on stdout regardless of verbosity; verbosity only ever affects the
stderr side channel. Default verbosity should be quiet enough that a script
piping stdout doesn't drown in noise on the screen, but `-v` should be enough
to diagnose what a stuck or failed command was actually doing.

**For `gcli`**: relevant to actual production debugging — a long-running
geoprocessing job invoked by the agent should be silent by default and only
verbose when explicitly asked, especially since stderr output is still
visible to a human watching a terminal even while stdout is being piped
onward.

### 11. `--dry-run`

For any verb that *mutates* something (writes a file, overwrites data, runs a
paid/expensive operation), a `--dry-run` flag should describe what *would*
happen without actually doing it — same inputs validated, same plan printed,
zero side effects. This matters doubly for an agent-driven tool: it gives the
agent (or the human reviewing an agent's proposed plan) a safe way to preview
a mutating command before actually approving it — directly analogous to this
platform's own Plan Mode, which already exists specifically to preview
mutations before they run.

### 12. Config file, environment variable, and flag precedence

Real tools accept configuration from three layers, with a fixed precedence
order: an explicit **flag** on the command line wins over an **environment
variable**, which wins over a **config file**, which wins over a **built-in
default**. Example: a default output CRS might be set in a project config
file, overridden by an environment variable for one shell session, overridden
again by an explicit `--crs` flag for one specific call.

**For `gcli`**: useful for project-level defaults (a project's working CRS,
its default output directory) without forcing every single invocation to
restate them — while still letting any individual call override anything.

### 13. Global flags vs. subcommand-specific flags

In a multi-subcommand tool (`gcli <verb> ...`), some flags apply no matter
which verb runs (`--config`, `--verbose`, `--json`) — these are **global
flags** and conventionally go *before* the verb name. Flags specific to one
verb's behavior (`--dist` for `buffer`) go *after* the verb name. Example:
`gcli --json buffer --dist 500 data.geojson` — `--json` is global (affects
every verb's output format), `--dist` only means something to `buffer`.

**For `gcli`**: keeping this distinction clean means adding a new verb never
requires re-deciding how output format or config-file loading works — those
stay handled once, globally, above the verb layer.

### 14. Signal handling (Ctrl+C)

Pressing Ctrl+C sends a signal called `SIGINT` to the running program. A
well-behaved long-running tool catches this and shuts down *cleanly* — closing
open files, not leaving partial/corrupt output behind — rather than being
killed mid-write. For a tool that can be interrupted mid-operation (a large
reprojection, a routing graph build), handling `SIGINT` gracefully is not
optional politeness, it's what prevents a half-written output file from being
mistaken for a complete one afterward.

**For `gcli`**: directly relevant to this platform's own job-cancellation
feature (compute-governance already supports cancelling a running job) — a
`gcli` verb invoked as part of a long job needs to behave the same way when
cancelled: clean up, don't leave corrupt partial output silently sitting where
a complete result is expected.

### 15. Idempotency and safe re-running

Running the same command twice with the same inputs should behave
predictably — either it produces the identical result again (safe to re-run,
e.g. a pure `reproject`), or it clearly refuses to silently clobber existing
output without an explicit `--force`/`--overwrite` flag. A command that
silently does something different the second time (partial overwrite,
appending instead of replacing) is a common, dangerous source of bugs —
especially for an agent that might legitimately retry a failed step, and needs
to trust that retrying doesn't quietly corrupt state from a partially-
succeeded first attempt.

### 16. Standardized stderr

An exit code says *whether* a verb failed. It says nothing about *why*. Left
unstandardized, stderr is just free-text prose — which puts the agent right
back to screen-scraping to explain a failure, the exact problem `--json`
already solves on the stdout side. stderr needs the same discipline.

**Fixed line shape, default (human-readable) mode:**

```
gcli: <verb>: <LEVEL>: <CODE>: <human message>
```

Example: `gcli: reproject: ERROR: E_CRS_UNKNOWN: "EPSG:99999" is not a
recognized CRS`

- **`<verb>`** — which subcommand emitted this line, so a merged stderr stream
  (several verbs piped together, all writing to the same terminal) stays
  traceable back to its source. Same convention real tools already use —
  `grep: file.txt: No such file or directory`.
- **`<LEVEL>`** — `ERROR` (fatal, exit code will be non-zero), `WARN`
  (non-fatal — e.g. "3 features had invalid geometry, skipped" — the process
  still succeeds and exits 0), `INFO` (progress, shown only at `-v`).
- **`<CODE>`** — a small, fixed, shared vocabulary across every verb:
  `E_INPUT_NOT_FOUND`, `E_INVALID_GEOMETRY`, `E_UNSUPPORTED_FORMAT`,
  `E_CRS_UNKNOWN`, `E_PERMISSION_DENIED`, `E_TIMEOUT`, `E_INTERNAL` — plus
  verb-specific codes only where genuinely needed. The code is what a caller
  (the agent, or future retry logic) branches on programmatically; the
  message after it is for a human reading the terminal.

**Under `--json`**, stderr mirrors the same discipline as stdout instead of
falling back to prose: one JSON object per line, e.g.
`{"level":"ERROR","code":"E_CRS_UNKNOWN","verb":"reproject","message":"..."}`
— so a `--json` caller never has to switch parsing strategy between
structured stdout and free-text stderr mid-command.

Why colons specifically, not some other separator: it's the existing Unix
convention for chaining context before a message (`grep: file.txt: No such
file or directory`) — easy for a human to scan left to right, and easy to
split on programmatically without needing a JSON parser for the plain-text
default mode. Not arbitrary; copying what already works.

Without a fixed code vocabulary, the only thing anyone can ever say about a
failure is "it failed." With codes, the agent can react differently by class
of failure — retry on `E_TIMEOUT`, don't retry on `E_INVALID_GEOMETRY`, ask
the user to fix the CRS on `E_CRS_UNKNOWN` — instead of guessing from prose.

---

**Design decisions already locked in for `gcli` from these lessons:**
- Every verb: reads stdin / writes stdout when no file is named (piping works
  by default).
- Wire format between verbs: GeoJSONSeq (streaming, one Feature per line).
- Every verb: real exit codes (0/non-zero), strict stdout/stderr separation.
- Every verb: `--json` machine-readable mode, alongside a human-readable
  default.
- Long-flag-first convention in agent-generated commands (clarity over typing
  speed, since the agent has no "muscle memory" benefit).
- Consistent `--` support for positional data that looks like a flag.
- stderr follows `gcli: <verb>: <LEVEL>: <CODE>: <message>`, with a fixed,
  shared error-code vocabulary and a JSON-lines form under `--json`.

## Part 3 — The verb catalogue

Two real pillars, not three — Rendering collapsed into Processing, since every
"rendering" verb has the same input-data → transform → output-artifact shape
as any other processing verb; the only thing that differs is the output
format and who consumes it next (see the Data/Processing/Rendering
discussion). Each verb below passed the self-critique check: does it secretly
fork into different behavior based on a hidden `--type`? If so, it was split
into separate verbs instead of staying one with a mode flag — `convert` split
off raster conversion into `warp`, `join` split into `join`/`spatial-join`,
`terrain` split into `slope`/`aspect`/`hillshade`, and `tile` split into
`tile`/`raster-tile`.

### Data

| Verb | Job |
|---|---|
| `info` | Inspect metadata — format, CRS, extent, schema, feature/band count. Output naturally varies by input type, same as Unix's `file`, but it's one job: describe what this dataset is. |
| `convert` | Vector format conversion (Shapefile → GeoJSON, etc.). Vector-only — raster conversion lives under `warp`, different engine, different flags entirely. |
| `validate` | Geometry/schema sanity check — catches bad input before it silently breaks a later Processing step. |

### Processing — vector geometry

`reproject`, `buffer`, `clip`, `intersect`, `union`, `difference`, `dissolve`,
`simplify`, `centroid`, `merge` — each a single, real spatial operation:
input geometry in, transformed geometry out.

### Processing — attributes / tabular

| Verb | Job |
|---|---|
| `filter` | Select features by an attribute predicate (e.g. `pop > 1000`). |
| `join` | Attribute join — match rows by a key column, no geometry involved. |
| `spatial-join` | Match rows by a spatial predicate (e.g. "point is within polygon") — split from `join`, genuinely different algorithm and required inputs. |
| `aggregate` | Group by attribute, summarize (count/sum/mean per group). |

### Processing — raster

| Verb | Job |
|---|---|
| `warp` | Reproject/resample/format-convert a raster (the `gdalwarp`/`gdal_translate` territory). |
| `mosaic` | Combine multiple rasters into one. |
| `calc` | Band math. |
| `slope` | Terrain slope from a DEM. |
| `aspect` | Terrain aspect from a DEM. |
| `hillshade` | Terrain hillshade from a DEM. |
| `zonal-stats` | Statistics of a raster within vector polygons. |
| `sample` | Sample raster values at given points. |

### Processing — point cloud

`pc-info`, `pc-filter` (by classification/ground/return-number — one
predicate-in, matching-subset-out shape regardless of which field), `pc-to-raster`
(DEM/DSM generation), `pc-to-mesh`.

### Processing — network / routing

`route` (shortest path), `isochrone` (reachability polygon), `matrix`
(travel time/distance between many points), `network-build` (prepare a
routing graph from road data — required before `route`/`isochrone`/`matrix`
can run against it).

### Processing — spatial indexing

`h3`, `s2` — hex/cell indexing of points or polygons.

### Processing — rendering-as-processing

| Verb | Job |
|---|---|
| `style` | Apply a general style expression (opacity, line width, data-driven expressions), produce styled output. |
| `colour` | Set fill/stroke color, optionally scoped by `--where` (only matched features change; everything else keeps its existing color untouched). Deliberately narrow and separate from `style` — same relationship as `mkdir` staying its own tool even where a more general filesystem tool also exists. |
| `tile` | Generate vector tiles (MVT/PMTiles via Tippecanoe). |
| `raster-tile` | Generate raster/image tile pyramids — split from `tile`, different engine (gdal2tiles), different output. |
| `render` | Export a static image (PNG) of styled data. |

### Dispatch — not Data, not Processing

| Verb | Job |
|---|---|
| `open` | Hand a dataset off to the actual renderer (the browser/map view) to display. No transform, no artifact produced — analogous to Unix's `open`/`xdg-open`, and the CLI-callable equivalent of this platform's existing "click to load onto the map" feature. |

`open` doesn't fit either pillar because it doesn't touch data at all — it's a
UI action, not a data operation. Worth keeping as its own small category
rather than forcing it into Processing just to preserve a clean two-pillar
story.

## Part 4 — Platform boundary and value conventions

### 17. `gcli`'s output contract vs. the platform's representation pipeline

The platform (this web app today, potentially a desktop app tomorrow)
distinguishes **raw data** from its **representation** — a lightweight,
tiled, browser-safe render format (PMTiles or equivalent) that's what
actually loads onto the map, so an arbitrarily large dataset never risks
OOM-ing the browser. This distinction is entirely platform-internal: neither
`gcli` nor the agent ever knows representations exist.

The full cascade when a geometry-altering verb runs: its stdout becomes a
**new raw data asset** → the platform automatically derives a **new
representation** from that raw data → that representation auto-loads onto
the map. All three of these are platform-only pipeline stages, strictly
**downstream of a successful `gcli` exit** — `gcli` never participates in
them and has no flag or concept for any of it.

This is why `gcli`'s own contract stays simple regardless of how elaborate
the platform's pipeline gets:

- **`gcli` never mutates its input in place.** Every verb reads from stdin/a
  file and writes a brand-new result to stdout — this was already guaranteed
  by the pure input→transform→output shape from Part 1, nothing new required.
- **`gcli`'s stdout is always raw geometry (GeoJSONSeq), never a
  representation.** Tiling/representation-generation begins only after
  `gcli` has already exited 0 with real geometry on stdout.
- **`gcli` has no concept of "layer," "project," "asset," or "representation"
  at all.** Deciding what to do with a verb's output (register it as a new
  layer, discard it, pipe it into another verb) is 100% the calling
  platform's policy, never the CLI's.
- **Exception**: verbs that only change *how* something is drawn without
  touching geometry (`style`, `colour`) do **not** trigger this cascade —
  no new raw data or representation is created, since nothing about the
  underlying data changed, only how it's rendered.

Why this is the right boundary for a future desktop migration: a desktop app
might not need tiling at all (a native engine could render raw geometry
directly without a browser's OOM risk) — its representation step could be a
no-op or something else entirely. That swap happens **entirely in the
platform-side pipeline that consumes `gcli`'s stdout** — `gcli` itself never
changes, because it was never aware representations existed in the first
place.

### 18. Color values — one canonical annotation across all color spaces

Real color spaces (RGB, HSL, OKLCH, etc.) are different mathematical models
for the same underlying color, each with its own notation
(`rgb(255,0,0)`, `hsl(0,100%,50%)`, `oklch(0.63 0.26 29)`). Hex (`#FF0000`) is
not a separate color space — it's just RGB's numbers written in base-16, and
it can't natively express spaces like OKLCH or wide-gamut spaces
(`display-p3`) without a lossy conversion into legacy sRGB first.

**`gcli` standardizes on CSS's generic function syntax as the one canonical
color annotation**: `color(<space> <values>)` — e.g. `color(srgb 1 0 0)` or
`color(oklch 0.63 0.26 29)` for the same red. One consistent wrapper shape,
parametrized by which space you're in, rather than a different function name
(or hex's RGB-only notation) per space. Any color flag (`--fill`, future
`--stroke`, etc.) accepts this form as the canonical value; `--json` output
always reports color values in this form too.

## Part 5 — Distribution & launch strategy

Not grammar — a real product/sequencing decision, kept here because it
determines what actually needs to be built first.

### 19. Launch sequencing: `gcli` ships before the platform

Real precedent (researched, not assumed): Claude Code launched as a pure
terminal CLI — no VS Code extension, no JetBrains plugin, no desktop app, no
web version. Those surfaces came later, once the core engine was already
real and proven. Same pattern elsewhere: `git` was a CLI for years before
GitHub Desktop or any IDE integration; Docker was a CLI before Docker
Desktop; `kubectl` before any dashboard. **Prove the engine standalone
before building a platform on top of it.**

Decision: `gcli` ships to real users, gets real feedback on whether it's
actually useful, and gets iterated on **in parallel with** building this
web platform — not sequentially blocking it. The platform's own *launch* is
gated on `gcli` earning real user approval; the platform's *development*
is not blocked waiting for `gcli` to be perfect. "Ships before the
platform" means solid enough for honest feedback, not fully
production-grade — those are different, deliberately separate bars (a full
production-grade, publicly packaged, licensed, documented standalone
product is a much larger, later commitment than "real enough that early
users' feedback is trustworthy").

### 20. Distribution channels

Priority order, driven by where the *real* target audience (GIS
professionals, not just developers) actually already is:

- **conda-forge** — the highest-priority, most strategic channel, not just
  another checkbox. Real GIS users already get GDAL itself via
  `conda install -c conda-forge gdal` (confirmed via research into how GDAL
  actually reaches Windows users — OSGeo4W and conda-forge are the two real,
  battle-tested mechanisms, manual PATH editing is explicitly not the
  answer). A user who already trusts conda for their GIS toolchain gets
  `gcli` through the exact same channel, not a new one to learn.
- **winget** / **Scoop** — the real, current Windows package-manager
  convention (confirmed via research into how `gh`/`kubectl`-class tools
  actually ship for Windows). Both are typically automated as CI side
  effects of a GitHub Release (e.g. via `goreleaser`'s winget block, or the
  `winget-releaser` GitHub Action), never manual submission.
- **npm** — near-zero extra effort given this codebase is already Node/TS,
  and reaches developer-leaning early adopters who might script against
  `gcli`. Real limitation: requires Node.js pre-installed, which most real
  GIS analysts (the audience whose feedback actually validates usefulness)
  won't have — npm alone is not sufficient reach on its own.
- **Homebrew** — the real, essentially-mandatory channel for macOS.
- **Chocolatey**, **apt**/`.deb`, **dnf**/`.rpm`, a **Docker image** (for
  headless/server/CI use), and plain **GitHub Releases** (a zip/tarball,
  the baseline every tool has regardless of package managers) — fill in as
  the ecosystem matures, not day one.

### 21. Code signing / Windows SmartScreen

A real, concrete blocker worth planning for, not discovering late: an
unsigned `.exe` downloaded directly (outside a package manager) triggers
Windows SmartScreen's "Windows protected your PC" warning, which damages
trust for a new, unknown tool. Real fix: a code-signing certificate — a
standard (OV) certificate still gets flagged until the binary accumulates
enough download/run reputation with Microsoft's SmartScreen system over
time; an **EV certificate** costs more but grants immediate trust, no
reputation-building wait.

For the early-feedback stage specifically, this doesn't need to be solved
yet — a small trusted alpha group can click "More Info → Run Anyway" past
the warning, completely normal for early beta software. It becomes a real
requirement closer to public launch, not before usefulness has even been
validated.

**This is also a second, independent reason winget/Scoop/Chocolatey/
conda-forge outrank a raw downloaded `.exe`** in the priority order above:
distribution through a package manager's own review/manifest system sidesteps
the raw-unsigned-binary SmartScreen problem entirely, rather than requiring
a paid certificate before any of those channels can be used.

## Part 6 — External reference & cross-check

### 22. Command Line Interface Guidelines (clig.dev)

A real, community-maintained reference this project deliberately follows
alongside its own grammar decisions above — built from real experience
running tools like Docker Compose, not theoretical. Two concrete additions
this doc didn't already have, worth locking in explicitly:

- **`NO_COLOR`** — any non-empty `NO_COLOR` environment variable disables
  ANSI color in `gcli`'s human-readable output. Not optional politeness —
  clig.dev treats this as a hard requirement, and it costs nothing to honor.
- **Don't go fully silent on success.** Traditional Unix philosophy favors
  no output on success — but for a human-facing tool, total silence reads
  as "did this hang, or did it actually work?" `gcli`'s real convention:
  brief, real confirmation output on success (not a wall of text — a short
  real summary), never nothing at all. `--quiet`/`-q` (§10) is the explicit
  opt-out for scripts that genuinely want traditional Unix silence.

Cross-check any new verb or flag against clig.dev directly
(https://clig.dev/), not just this document, before finalizing it.
