# ADR-010: Dependency Management Strategy

## Status
Accepted

## Date
2026-02-18

## Context
The project has 500+ transitive dependencies (typical for a GTK4 Rust application). Unmanaged dependencies lead to security vulnerabilities, license compliance issues, duplicate crate versions increasing compile times, and breaking changes on update.

## Decision
Implement a multi-layered dependency management strategy:

### Automated Updates
- **Dependabot** configured for weekly PRs (Cargo, GitHub Actions, npm)
- Patch and minor updates grouped to reduce PR noise
- CI must pass before merging any dependency update

### Auditing
- **`cargo deny`** runs in CI on every push/PR:
  - **Advisories**: Deny known vulnerabilities (RustSec database)
  - **Licenses**: Allowlist of permissive licenses (MIT, Apache-2.0, BSD, etc.)
  - **Bans**: Warn on duplicate crate versions
  - **Sources**: Only allow crates.io registry
- **`cargo audit`** for vulnerability scanning
- **`cargo-udeps`** for detecting unused dependencies (periodic, nightly-only)

### Workspace Dependencies
- Common dependencies declared in workspace `[workspace.dependencies]`
- Sub-crates reference via `{ workspace = true }`
- Ensures version consistency across all workspace members

### Update Policy
- **Patch updates**: Apply automatically via Dependabot PRs
- **Minor updates**: Review changelog, apply if backward-compatible
- **Major updates**: Plan migration, test thoroughly, document in CHANGELOG
- **Security fixes**: Apply immediately regardless of version bump

### Configuration Files
- `deny.toml` — cargo-deny rules (advisories, licenses, bans, sources)
- `.cargo/mutants.toml` — mutation testing configuration
- `.github/dependabot.yml` — automated update schedule

## Consequences

**Positive:**
- Security vulnerabilities caught early via automated scanning
- License compliance verified on every CI run
- Unused dependencies don't accumulate
- Consistent versions across workspace crates

**Negative:**
- Weekly Dependabot PRs require review attention
- `cargo deny` may require allowlist updates when adding new dependencies
- Nightly toolchain needed for `cargo-udeps` (not in regular CI)
- Some transitive duplicates are unavoidable (e.g., bitflags v1 + v2)

## Alternatives Considered
- **No automated scanning**: Less maintenance but security risks go unnoticed
- **Renovate instead of Dependabot**: More configurable but Dependabot is native to GitHub
- **Vendoring dependencies**: Maximum reproducibility but enormous repo size and update burden
