# Versioning & deprecation policy

`gcli` follows [Semantic Versioning](https://semver.org/) —
`MAJOR.MINOR.PATCH`:

- **MAJOR** — a breaking change: a verb removed, a flag's meaning changed,
  a default behavior changed, a stdout/`--json` shape changed in a way that
  would break an existing script or agent-generated command.
- **MINOR** — a new verb, a new flag, a new capability. Never breaks an
  existing command.
- **PATCH** — a real bug fix, no interface change at all.

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `BREAKING CHANGE:` in the footer, etc.) so the version
bump for a release is derived from real commit history, not decided by
hand each time.

## Real, staged deprecation — never a silent removal

Nothing in `gcli`'s grammar (a verb, a flag, a default) is ever removed in
one step. The lifecycle, modeled on Terraform's real, documented version
policy:

1. **Available** — the normal, supported state.
2. **Deprecated** — still works exactly as before, but emits a real
   `WARN`-level line on stderr naming what's deprecated, why, and what to
   use instead. Deprecation is announced in a MINOR release, never
   silently.
3. **End of support** — removed only in the *next* MAJOR release after
   deprecation, never sooner. A user or the agent always has at least one
   full MAJOR version's worth of real warning before anything actually
   breaks.

This matters more than usual for `gcli` specifically because the agent
generates commands against this same grammar — a silent behavior change
would break agent-driven workflows exactly as badly as a human's saved
script.

## Pre-1.0

Per real semver convention, `0.x` releases may break things between MINOR
bumps while the core verb catalogue and primitives are still being proven
against real usage — the staged deprecation policy above becomes a hard
guarantee starting at `1.0.0`, not before.
