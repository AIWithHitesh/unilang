# UniLang Incubation Proposal

**Submitted to:** Apache Software Foundation Incubator  
**Date:** 2026-04-17  
**Version:** 1.5.0  
**Champion/Sponsor:** (to be identified — seeking an Apache champion)

---

## Abstract

UniLang is an open-source, statically-analysed, hybrid programming language that seamlessly
unifies Python and Java syntax, semantics, and idioms into a single language. Programs can
freely mix Python-style `def` functions, indentation-based blocks, and comprehensions with
Java-style class declarations, brace-delimited blocks, and typed field annotations — in the
same file, on adjacent lines.

UniLang is implemented in Rust as a 12-crate workspace. It ships a full compiler pipeline
(lexer → parser → semantic analyser → bytecode codegen → stack VM), a standard library of
113 functions, 13 production-ready database and service drivers, a Language Server Protocol
(LSP) server, IDE plugins for VS Code, JetBrains, Eclipse, and a standalone Electron-based
IDE.

---

## Proposal

### Background

Two ecosystems dominate modern software development: Python — beloved for data science,
scripting, and rapid prototyping — and Java — trusted for large-scale enterprise systems.
Developers who work in mixed teams or maintain polyglot codebases carry constant cognitive
overhead switching between the two languages: different syntax, different idioms, different
toolchains, and no common type system.

UniLang addresses this by defining a single language whose grammar is a strict superset of
both Python and Java surface syntax. A UniLang program that looks entirely Python-like is
valid; so is one that looks entirely Java-like; so is one that mixes both. The compiler
resolves ambiguity through context — indented blocks vs. brace blocks, `def` vs. typed
method declarations — and produces a common IR (bytecode) that runs on the Rust-native
UniLang VM.

The project was started by AIWithHitesh in late 2025 and reached a stable v1.0 release in
February 2026 after four months of intensive development. It is currently hosted on GitHub
under a personal account and is seeking a neutral, community-governed home to encourage
broader adoption and contribution.

### Rationale

The Apache Software Foundation is the right home for UniLang for the following reasons:

1. **Alignment with the Apache mission.** The ASF exists to produce software for the public
   good. A unified language that lowers the barrier to polyglot programming serves a broad
   audience of developers.

2. **Neutral governance.** Moving the project to `apache/unilang` removes the dependency on
   a single GitHub account and allows any contributor to become a committer through merit,
   consistent with the Apache meritocracy model.

3. **License compatibility.** UniLang is already released under the Apache License 2.0.
   No relicensing is required.

4. **Infrastructure.** The ASF provides mailing lists, CI resources (via GitHub Actions
   under the Apache org), and release infrastructure aligned with the project's existing
   workflow.

5. **Community trust.** The Apache brand signals to enterprise adopters and open-source
   contributors alike that the project is governed openly and sustainably.

---

### Current Status

UniLang v1.5.0 (as of this proposal) includes:

**Compiler Pipeline**
- Unified lexer handling all Python + Java token forms (INDENT/DEDENT, braces, f-strings)
- Pratt parser producing a unified AST (`Expr`, `Stmt`, `Module`, `ClassDecl`, `FuncDecl`)
- Semantic analyser with nested scope stack, gradual type inference, overload resolution,
  and generic type checking (`List<T>`, `Map<K,V>`, `Option<T>`)
- Bytecode codegen with 40+ opcode instruction set and constant pool
- Disassembler for bytecode inspection

**Runtime**
- Stack-based Rust VM with call frames, closures, class instantiation, exception handling
- Standard library: 113 functions across I/O, math, string, collections, JSON, HTTP, env,
  file, time, random, and type-conversion modules
- Execution sandbox (`ExecutionLimits` API) with default, sandboxed, and development profiles

**Driver Ecosystem — 13 drivers**

| Driver | Transport | Status |
|--------|-----------|--------|
| SQLite | `rusqlite` (bundled) | Core (always on) |
| Redis | `redis` crate | Feature-gated |
| Kafka | In-memory simulation | Feature-gated |
| Elasticsearch | HTTP/`ureq` | Feature-gated |
| MySQL | `mysql_async` | Feature-gated |
| PostgreSQL | `tokio-postgres` | Feature-gated |
| MongoDB | `mongodb` crate | Feature-gated |
| Memcached | `memcache` crate | Feature-gated |
| RabbitMQ | `lapin` crate | Feature-gated |
| NATS | `nats` crate | Feature-gated |
| Prometheus | HTTP/`ureq` | Feature-gated |
| InfluxDB | HTTP/`ureq` | Feature-gated |
| SMTP | `lettre` crate | Feature-gated |

Community drivers can be added by dropping a single `.rs` file in `src/community/` — a
`build.rs` script auto-discovers and registers them without requiring changes to `lib.rs` or
`Cargo.toml`.

**JVM/CPython Bridge**
- JNI bridge (`jni 0.21`): static/instance calls, field access, class loading, jar loading
- CPython bridge (`pyo3 0.22`): module import, function/method calls, attribute access, eval/exec
- Full type marshaling: `RuntimeValue` ↔ `BridgeValue` ↔ `JValue`/`PyObject`
- Zero-copy array bridge (`SharedArrayBuffer`, JNI typed arrays, numpy buffer protocol)
- Java thread pool integration via `ExecutorService`

**Toolchain**
- `unilang run`, `check`, `compile`, `lex`, `parse`, `fmt`, `lint`, `test`, `repl`, `new`,
  `driver list|new`, `pack`, `build --incremental`, `config show|init`, `lock`

**IDE & LSP**
- LSP server (`tower-lsp 0.20`): hover for 80+ keywords + 60 stdlib functions,
  go-to-definition, formatting
- VS Code extension: syntax highlighting, 15 snippets, DAP debugger, LSP client
- JetBrains plugin: IntelliJ / PyCharm support
- Eclipse plugin
- Standalone UniLang IDE (Electron-based)

**Testing & Quality**
- 500+ unit, integration, and end-to-end tests across all crates
- Criterion performance benchmarks for compiler pipeline and VM throughput
- Stress tests: 5 programs of 300–500 lines each
- CI matrix: ubuntu-latest, macos-13, macos-latest, windows-latest
- Memory safety: Valgrind + LSAN CI job

**Documentation**
- Language specification (136 EBNF grammar rules)
- Architecture and compiler pipeline docs
- API reference (113 stdlib functions)
- Driver reference
- Quickstart guide and cookbook (8 recipes)
- Migration guides: Java → UniLang, Python → UniLang
- 5 complete example projects

---

### Meritocracy

UniLang adopts the Apache meritocracy model from the moment it enters the incubator:

- All contributions are evaluated on technical merit, code quality, and alignment with the
  project's goals — regardless of the contributor's employer or affiliation.
- Commit access is granted to contributors who demonstrate sustained, high-quality
  contributions over time, as voted on by existing committers.
- Major design decisions (new syntax forms, bytecode format changes, driver API) are
  discussed on the public `dev@` mailing list and decided by lazy consensus or formal vote.
- A `CONTRIBUTING.md` document specifies the contribution process; a `CONTRIBUTORS` file
  will track all contributors.
- The Project Management Committee (PMC) will be seeded from the initial set of committers
  and grow through the standard Apache nomination process.

---

### Community

**Current state:**
- Primary development: GitHub (`https://github.com/AIWithHitesh/unilang`)
- Issues and feature requests: GitHub Issues
- Releases: GitHub Releases

**Proposed Apache infrastructure:**
- Dev mailing list: `dev@unilang.apache.org`
- User mailing list: `users@unilang.apache.org`
- Announcements: `announce@unilang.apache.org`
- Issue tracking: GitHub Issues (via the GitHub → JIRA bridge or directly under `apache/unilang`)
- CI: GitHub Actions under the `apache/` organisation

**Community health goals (Year 1 in incubation):**
- Recruit at least 3 additional committers from outside the founding team
- Achieve at least 20 external contributors (non-committer PRs merged)
- Conduct monthly community calls, recorded and published
- Graduate from incubation within 18 months

---

### Core Developers

| Name | GitHub | Role |
|------|--------|------|
| AIWithHitesh | @AIWithHitesh | Project Lead — compiler, VM, stdlib, drivers |

Additional contributors and committers will be nominated through the standard Apache process
during incubation.

---

### Alignment with Apache's Mission

The Apache Software Foundation's mission is to provide software for the public good.
UniLang aligns with this mission in the following ways:

1. **Lowers barriers to entry.** Developers already fluent in Python or Java can write
   UniLang code immediately — no new syntax to learn for their preferred style.

2. **Reduces ecosystem fragmentation.** Organisations maintaining both Python and Java
   services can use a single language, shared toolchain, and shared driver ecosystem across
   their stack.

3. **Open and neutral governance.** By joining the ASF, the project commits to open
   development, transparent decision-making, and community ownership — not vendor control.

4. **Educational value.** UniLang's documented compiler pipeline (lexer → parser → semantic →
   codegen → VM) serves as a pedagogical resource for developers learning compiler
   construction in Rust.

5. **Enterprise readiness.** Apache certification signals to enterprise adopters that the
   project is governed, licensed, and distributed in a way that is safe to use.

---

### Known Risks and Mitigating Factors

| Risk | Mitigation |
|------|-----------|
| Small initial team | Open contribution model, clear `CONTRIBUTING.md`, community driver guide, and `unilang driver new` scaffold lower the bar for first contributions |
| Language adoption in a crowded market | Unique value proposition (unified Python+Java) targets a well-defined underserved audience; strong IDE support and migration guides ease adoption |
| Rust expertise required for contributions | Core language semantics are well-documented; many contributions (drivers, examples, docs, tests) require only UniLang or Rust novice-level knowledge |
| Bytecode format stability | Bytecode format changes are MAJOR version bumps per the semantic versioning policy; format is documented in `docs/specifications/` |
| JVM/CPython bridge maintenance | Bridge is isolated in `crates/unilang-bridge`; feature-gated so projects not needing it carry zero overhead; versioned against specific `jni` and `pyo3` crates |
| Dependency on `pyo3`/`jni` C-FFI crates | Both crates are mature, widely used in the Rust ecosystem, and are themselves Apache-2.0 or MIT licensed |
| Compliance with ASF IP policy | All code is authored by the initial contributor or is from dependencies with Apache-2.0, MIT, or BSD licenses; a full IP clearance will be performed during incubation |

---

### Documentation

- `README.md` — project overview, quick install, quick start example
- `docs/planning/PRD.md` — product requirements
- `docs/planning/ROADMAP.md` — phased roadmap
- `docs/specifications/LANGUAGE_SPEC.md` — language specification
- `docs/specifications/GRAMMAR.ebnf` — 136-rule formal grammar
- `docs/architecture/ARCHITECTURE.md` — system architecture
- `docs/architecture/COMPILER_PIPELINE.md` — compiler pipeline deep-dive
- `docs/design/DESIGN_DECISIONS.md` — key design decisions and rationale
- `docs/guides/QUICKSTART.md` — tutorial and quick start
- `docs/guides/API_REFERENCE.md` — 113 stdlib functions reference
- `docs/DRIVERS.md` — driver architecture and development guide
- `CONTRIBUTING.md` — contribution workflow
- `CONTRIBUTING_DRIVERS.md` — driver contribution and auto-discovery guide
- `SECURITY.md` — vulnerability disclosure and sandbox policy
- `CHANGELOG.md` — version history (keepachangelog format)

---

### Initial Source

- **Repository:** `https://github.com/AIWithHitesh/unilang`
- **Primary language:** Rust (compiler, VM, drivers, LSP)
- **Supporting languages:** TypeScript (VS Code extension), Kotlin (JetBrains plugin), Java
  (Eclipse plugin), JavaScript/Electron (standalone IDE)
- **Test programs:** UniLang (`.uniL` files in `examples/` and `tests/`)

The source will be transferred to `https://github.com/apache/unilang` upon acceptance into
the incubator. An IP clearance audit will be conducted to verify all dependencies are
compatible with the Apache License 2.0.

---

### External Dependencies

The following key runtime dependencies will be reviewed for license compatibility during
incubation (all are MIT or Apache-2.0 unless noted):

| Crate | License | Purpose |
|-------|---------|---------|
| `rusqlite` | MIT | SQLite driver (bundled) |
| `redis` | BSD-3-Clause | Redis driver |
| `mongodb` | Apache-2.0 | MongoDB driver |
| `mysql_async` | MIT / Apache-2.0 | MySQL driver |
| `tokio-postgres` | MIT | PostgreSQL driver |
| `pyo3` | Apache-2.0 | CPython C API bridge |
| `jni` | MIT | JVM JNI bridge |
| `tower-lsp` | MIT | LSP server framework |
| `tower` | MIT | Async service abstraction (LSP) |
| `tokio` | MIT | Async runtime |
| `ureq` | MIT | HTTP client (Elasticsearch, InfluxDB) |
| `clap` | MIT / Apache-2.0 | CLI argument parsing |
| `thiserror` | MIT / Apache-2.0 | Error type derivation |
| `criterion` | Apache-2.0 | Benchmarking |
| `serde` + `serde_json` | MIT / Apache-2.0 | Serialisation |
| `zip` | MIT | `.uniLpkg` packaging |
| `sha2` | MIT / Apache-2.0 | Incremental build cache hashing |
| `walkdir` | MIT | Directory traversal (auto-discovery) |

All Rust crates are obtained from `crates.io`. No vendored native code beyond `rusqlite`'s
bundled SQLite amalgamation (public domain).

---

### Required Resources

- **GitHub repository:** Migration from `AIWithHitesh/unilang` to `apache/unilang`
- **Mailing lists:**
  - `dev@unilang.apache.org` — developer discussion
  - `users@unilang.apache.org` — user support
  - `commits@unilang.apache.org` — automated commit notifications
- **Issue tracking:** GitHub Issues under `apache/unilang`
- **CI:** GitHub Actions (existing workflows will be migrated)
- **Release infrastructure:** GitHub Releases + Apache distribution mirrors (if needed)
- **Website:** `unilang.apache.org` (initially a redirect to the GitHub pages site)

---

### Initial Committers

| Name | GitHub | Apache ID (to be assigned) |
|------|--------|---------------------------|
| AIWithHitesh | @AIWithHitesh | tbd |

Additional committers will be recruited during incubation from contributors who demonstrate
sustained engagement with the project.

---

### Sponsors

This proposal is seeking an Apache champion and mentors. The following ASF members have been
contacted (or will be contacted) regarding this proposal:

- Champion: **(to be identified)**
- Nominated mentors:
  - **(to be identified)** — experience with language projects
  - **(to be identified)** — experience with Rust projects in ASF
  - **(to be identified)** — experience with toolchain/IDE projects

If you are an ASF member interested in championing or mentoring this project, please reach
out via the `general@incubator.apache.org` list or directly to `hitesh11.kumar@ril.com`.

---

## ASF Incubator Checklist

The following items from the standard ASF incubation checklist will be addressed during
the incubation process:

- [ ] Project name search confirms no trademark conflicts with `UniLang`
- [ ] All source files have Apache License 2.0 headers
- [ ] Full list of external dependencies and their licenses compiled (`LICENSE`, `NOTICE`)
- [ ] IP clearance: contributor agreement (ICLA) signed by all committers
- [ ] Corporate CLA (CCLA) signed if applicable
- [ ] No GPL or LGPL dependencies in the default build
- [ ] No binary artifacts checked into the repository
- [ ] Release process follows ASF release policy (signed, voted on `dev@`)
- [ ] At least three +1 votes from IPMC members for each release during incubation
- [ ] Project website up at `unilang.apache.org` (or GitHub Pages redirect)
- [ ] Mailing list archives publicly accessible
- [ ] Board reporting cadence established (monthly during first year)

---

*This proposal was drafted by the UniLang core team on 2026-04-17. It is a living document
and will be updated in response to feedback from the Apache Incubator PMC.*
