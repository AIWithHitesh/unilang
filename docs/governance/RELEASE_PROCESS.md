# UniLang Release Process

This document describes the end-to-end process for cutting a UniLang release, from branch
freeze through binary distribution. All maintainers are expected to follow this process.

---

## Version Numbering

UniLang follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

| Segment | When to bump |
|---------|-------------|
| `MAJOR` | Breaking changes to the language grammar, VM bytecode format, or public Rust API |
| `MINOR` | New features, new CLI commands, new drivers, new stdlib functions |
| `PATCH` | Bug fixes, documentation corrections, security patches |

Pre-release labels: `-rc1`, `-rc2`, … `-beta.1`, `-alpha.1`

---

## Release Cadence

| Type  | Frequency | Trigger |
|-------|-----------|---------|
| Patch | As needed | Critical bugs, CVE fixes |
| Minor | Monthly   | Accumulated features in `main` |
| Major | Quarterly (at most) | When breaking changes are ready and agreed |

---

## Branches and Tags

| Ref | Purpose |
|-----|---------|
| `main` | Trunk — always releasable |
| `release/vX.Y` | Stabilisation branch for a specific minor version |
| `hotfix/vX.Y.Z` | Patch-only branch off a release tag |
| `vX.Y.Z-rcN` | Release candidate annotated tag |
| `vX.Y.Z` | Final release annotated tag |

---

## Release Candidate Process

### Step 1: Freeze and Branch

1. Confirm all planned features for the release are merged to `main`.
2. Create a stabilisation branch from `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b release/v1.6
   git push origin release/v1.6
   ```
3. Post a freeze notice in the `#releases` channel (or GitHub Discussion) stating that
   only bug fixes will be merged to the release branch from this point forward.
4. Bump the version in `Cargo.toml` on the release branch:
   ```toml
   # workspace Cargo.toml
   [workspace.package]
   version = "1.6.0"
   ```
5. Update `CHANGELOG.md` — move items from `[Unreleased]` into the new `[1.6.0]` section
   with the planned release date (leave as placeholder if unknown).

### Step 2: RC Tag

Create and push the first release candidate tag:

```bash
git checkout release/v1.6
git pull origin release/v1.6
git tag -a v1.6.0-rc1 -m "Release candidate 1 for v1.6.0"
git push origin v1.6.0-rc1
```

The `release.yml` GitHub Actions workflow triggers automatically on tags matching
`v*.*.*-rc*`, builds all platform binaries, and attaches them to a GitHub pre-release.

### Step 3: Testing the Release Candidate

All of the following must be completed before promoting an RC to final release.

#### Automated (CI)

- [ ] All unit tests pass: `cargo test --workspace`
- [ ] All integration tests pass: `cargo test --workspace --test '*'`
- [ ] Stress tests pass: `cargo test --workspace --test stress`
- [ ] Benchmarks run without regression: `cargo bench --workspace 2>&1 | tee bench-rc1.txt`
- [ ] CI matrix green: ubuntu-latest, macos-13, macos-latest, windows-latest
- [ ] Memory safety: Valgrind + LSAN CI job exits clean
- [ ] Clippy: `cargo clippy --workspace -- -D warnings`
- [ ] Format: `cargo fmt --check --all`

#### Manual

- [ ] Download the RC binary installer on a **clean** Linux machine and run:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/AIWithHitesh/unilang/v1.6.0-rc1/install.sh | sh
  unilang --version   # expect: unilang 1.6.0-rc1
  unilang new hello
  cd hello
  unilang run src/main.uniL
  ```
- [ ] Repeat installer test on macOS (arm64) and Windows (PowerShell installer)
- [ ] VS Code extension: install the `.vsix` from the RC release, verify highlighting,
  hover, go-to-definition, and the REPL launch command all work
- [ ] JetBrains plugin: install the `.zip` in IntelliJ, verify syntax highlighting and
  basic LSP completion
- [ ] Run all five example projects end-to-end:
  - `examples/web-service/`
  - `examples/data-pipeline/`
  - `examples/library-mgmt/`
  - `examples/ecommerce/`
  - `examples/ml-framework/`
- [ ] Verify `unilang repl` starts and accepts multi-line input
- [ ] Verify `unilang test` discovers and runs `def test_*()` functions
- [ ] Verify `unilang fmt --write` and `unilang lint` work on all examples

#### Security

- [ ] Review `SECURITY.md` — confirm `ExecutionLimits` sandbox profiles are accurate
- [ ] Check for new `cargo audit` advisories: `cargo audit`
- [ ] Confirm no credentials, private keys, or internal hostnames in the release artifact

### Step 4: RC Iteration

If issues are found during RC testing:

1. Fix on `main` first (unless the bug is release-branch-specific).
2. Cherry-pick the fix to the release branch:
   ```bash
   git checkout release/v1.6
   git cherry-pick <commit-sha>
   git push origin release/v1.6
   ```
3. Tag a new RC: `v1.6.0-rc2`, `v1.6.0-rc3`, …
4. Re-run all manual checks from Step 3.

Aim for no more than three RC iterations. If more are needed, schedule a team retrospective
to identify the root cause of instability.

### Step 5: Final Release

Once an RC passes all checks with no blocking issues open for 48 hours:

1. Update `CHANGELOG.md` with the final release date:
   ```markdown
   ## [1.6.0] - 2026-05-17
   ```
2. Commit and push the changelog update:
   ```bash
   git add CHANGELOG.md
   git commit -m "chore: finalize changelog for v1.6.0"
   git push origin release/v1.6
   ```
3. Tag the final release:
   ```bash
   git tag -a v1.6.0 -m "UniLang v1.6.0"
   git push origin v1.6.0
   ```
4. The `release.yml` workflow triggers on `v*.*.*` (non-RC) tags:
   - Builds all platform binaries (Linux x86_64/arm64, macOS arm64/x86_64, Windows x86_64)
   - Signs macOS binaries with `codesign`
   - Generates SHA-256 checksums and uploads as release assets
   - Publishes the VS Code extension to the Marketplace
   - Publishes the JetBrains plugin to the JetBrains Marketplace
   - Creates a GitHub Release with the release notes from `CHANGELOG.md`
5. Merge the release branch back into `main`:
   ```bash
   git checkout main
   git merge --no-ff release/v1.6
   git push origin main
   ```
6. Bump `main` to the next development version (e.g., `1.7.0-dev`).

---

## Release Checklist

Copy this checklist into the GitHub Release issue for each release.

```
### Pre-release
- [ ] All tests passing (CI green on ubuntu, macos, windows)
- [ ] cargo audit — no unresolved advisories
- [ ] cargo clippy — zero warnings
- [ ] CHANGELOG.md updated with all entries for this version
- [ ] Version bumped in workspace Cargo.toml
- [ ] ROADMAP.md phase statuses updated
- [ ] docs/planning/ROADMAP.md Last Updated field updated

### Release candidate
- [ ] RC tag pushed: v{VERSION}-rc1
- [ ] Binary installer tested on clean Linux machine
- [ ] Binary installer tested on macOS (arm64)
- [ ] Binary installer tested on Windows (PowerShell)
- [ ] VS Code extension .vsix installed and smoke-tested
- [ ] JetBrains plugin .zip installed and smoke-tested
- [ ] All 5 example projects run end-to-end
- [ ] Stress tests completed
- [ ] Memory safety (Valgrind/LSAN) CI job passed
- [ ] No blocking issues open for 48h

### Final release
- [ ] Final release tag pushed: v{VERSION}
- [ ] GitHub Release created with correct release notes
- [ ] SHA-256 checksums attached to release
- [ ] VS Code extension published to Marketplace
- [ ] JetBrains plugin published to JetBrains Marketplace
- [ ] install.sh and install.ps1 updated to point to new version
- [ ] Release announcement published (HN, Reddit r/programming, r/rust)
- [ ] release/v{MAJOR}.{MINOR} branch merged back to main
- [ ] main version bumped to next dev version
```

---

## Hotfix Process

For critical bugs or security issues discovered in a released version:

1. Create a hotfix branch from the release tag (not from `main`):
   ```bash
   git checkout v1.6.0
   git checkout -b hotfix/v1.6.1
   git push origin hotfix/v1.6.1
   ```
2. Apply the minimal fix:
   ```bash
   # Fix the issue, add tests
   git add -p
   git commit -m "fix: <describe the fix>"
   ```
3. Bump the patch version in `Cargo.toml` (`1.6.0` → `1.6.1`).
4. Update `CHANGELOG.md` with a `[1.6.1]` section.
5. Tag and push:
   ```bash
   git tag -a v1.6.1 -m "UniLang v1.6.1 — hotfix"
   git push origin v1.6.1
   ```
6. Cherry-pick the fix to `main`:
   ```bash
   git checkout main
   git cherry-pick <fix-commit-sha>
   git push origin main
   ```
7. Delete the hotfix branch after the release publishes.

For security issues, coordinate via the private security channel described in `SECURITY.md`
before any public commit or tag. Embargo the fix until the release is available.

---

## Signing and Verification

### macOS

macOS binaries are signed with `codesign` in CI using a Developer ID Application certificate
stored in GitHub Actions secrets (`MACOS_SIGNING_CERT`, `MACOS_SIGNING_CERT_PASSWORD`).
Verification:
```bash
codesign --verify --verbose=4 /usr/local/bin/unilang
spctl --assess --verbose /usr/local/bin/unilang
```

### Linux / Windows

SHA-256 checksums are provided alongside each release binary. Verify before installing:
```bash
# Linux
sha256sum -c unilang-v1.6.0-linux-x86_64.tar.gz.sha256

# Windows (PowerShell)
(Get-FileHash unilang-v1.6.0-windows-x86_64.zip -Algorithm SHA256).Hash
```

---

## Communication

| Event | Channel |
|-------|---------|
| Freeze notice | GitHub Discussions + `#releases` |
| RC available | GitHub Discussions + `#releases` |
| Final release | GitHub Release + GitHub Discussions + HN + Reddit |
| Security hotfix | SECURITY.md embargo process → coordinated disclosure |
