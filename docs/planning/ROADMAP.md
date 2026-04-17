# UniLang — Development Roadmap

**Version:** 1.3.0  
**Last Updated:** 2026-04-17

---

## Phase Overview

```
Phase 0: Foundation        ██████████████████░░  [Mostly Complete]
Phase 1: Core Compiler     ████████████████████  [Complete ✓]
Phase 2: Runtime & VM      ████████████████████  [Complete ✓]
Phase 3: Toolchain         ██████████░░░░░░░░░░  [In Progress]
Phase 4: IDE & Ecosystem   ████████████████░░░░  [Mostly Complete]
Phase 5: Production Ready  ░░░░░░░░░░░░░░░░░░░░
```

---

## Phase 0: Foundation

**Goal:** Establish project infrastructure, governance, and core design documents.

| Task | Status | Owner | Notes |
|------|--------|-------|-------|
| Project repository setup | ✅ Done | Core Team | GitHub repo, directory structure, license |
| PRD and vision document | ✅ Done | Core Team | `docs/planning/PRD.md` |
| Architecture design document | ✅ Done | Core Team | `docs/architecture/ARCHITECTURE.md` |
| Language specification (draft) | ✅ Done | Core Team | `docs/specifications/LANGUAGE_SPEC.md` |
| Design decisions document | ✅ Done | Core Team | `docs/design/DESIGN_DECISIONS.md` |
| Contributing guidelines | ✅ Done | Core Team | `CONTRIBUTING.md` (includes driver guide) |
| Apache License 2.0 setup | ✅ Done | Core Team | LICENSE, NOTICE, file headers |
| Build system setup (Cargo) | ✅ Done | Core Team | 8-crate Rust workspace |
| Driver development guide | ✅ Done | Core Team | `docs/DRIVERS.md` |
| Formal grammar (EBNF) | ✅ Done | Core Team | `docs/specifications/GRAMMAR.ebnf` — 136 rules, full Python+Java coverage |
| CI/CD pipeline | ✅ Done | Core Team | GitHub Actions — build + test on push |
| Community infrastructure | 🔲 TODO | | Issue templates, GitHub Discussions |
| Apache incubation proposal draft | 🔲 TODO | | |

**Exit Criteria:** All design docs reviewed, build system compiles, CI green. ✅

---

## Phase 1: Core Compiler

**Goal:** Build the compiler frontend — lexer, parser, and semantic analyzer handling both Python and Java syntax.

### Milestone 1.1: Lexer ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| Unified token type definitions | ✅ Done | Union of Python + Java tokens in `unilang-lexer` |
| Python tokenization rules | ✅ Done | `def`, `class`, `import`, `from`, indentation-aware |
| Java tokenization rules | ✅ Done | `public`, `void`, `class`, braces, semicolons |
| String literal handling | ✅ Done | Single/double quoted, f-strings (`f"..."`) |
| Comment handling | ✅ Done | `//`, `#`, `/* */` |
| Lexer error recovery | ✅ Done | Unknown character diagnostic |
| Lexer test suite | 🔲 TODO | Formal coverage report |

### Milestone 1.2: Parser ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| Unified AST node definitions | ✅ Done | `unilang-parser`: Expr, Stmt, Module |
| Expression parser (Pratt) | ✅ Done | Full precedence climbing, binary/unary/call/index/member |
| Statement parser | ✅ Done | Assign, If, While, For, Return, Break, Continue, Delete |
| Block parser (indent + brace) | ✅ Done | Both `{...}` and `:` + indent styles |
| Class declaration parser | ✅ Done | Java-style field decls + Python-style `def` methods |
| Function declaration parser | ✅ Done | Both `def fn():` and `void fn() {}` |
| Import statement parser | ✅ Done | `import x`, `from x import y` |
| Control flow parser | ✅ Done | `if/elif/else`, `while`, `for`, `try/except/finally` |
| Exception handling parser | ✅ Done | `try/except` (Python) and `try/catch` (Java) |
| Parser error recovery | ✅ Done | Resync on statement boundary |
| Parser test suite | 🔲 TODO | Formal coverage report |

### Milestone 1.3: Semantic Analyzer ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| Symbol table implementation | ✅ Done | Nested scope stack in `unilang-semantic` |
| Type inference engine | ✅ Done | Gradual typing: Int, Float, String, Bool, Array, Dynamic |
| Prelude / standard function resolution | ✅ Done | 35+ stdlib + all driver function names |
| Import resolution | ✅ Done | Marks imports as dynamic for interop |
| Overload resolution | ✅ Done | Core Team | Multiple same-name functions; best-match scoring; gradual fallback to Dynamic |
| Generic type checking | ✅ Done | Core Team | `List<T>`, `Map<K,V>`, `Option<T>`; element-type checking for append; gradual |
| Semantic error reporting | ✅ Done | Span-based diagnostics with labels |
| Semantic analyzer tests | 🔲 TODO | Formal coverage report |

**Exit Criteria:** Compiler parses and type-checks 500-line mixed-syntax programs without errors. ✅

---

## Phase 2: Runtime & VM

**Goal:** Build the bytecode compiler, stack-based VM, standard library, and driver ecosystem.

> **Architecture note:** UniLang uses a Rust-native stack-based VM (not JVM + CPython). The dual-VM JVM/CPython bridge is a long-term v2 goal. The current Rust VM delivers a fully working runtime today with zero external runtime dependencies.

### Milestone 2.1: Bytecode Compiler ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| Instruction set design (40+ opcodes) | ✅ Done | `unilang-codegen`: OpCode enum |
| AST → bytecode lowering | ✅ Done | Expressions, statements, functions, classes |
| Constant pool | ✅ Done | String/int/float constants |
| Function compilation | ✅ Done | Params, locals, closures |
| Class compilation | ✅ Done | Field declarations, method dispatch |
| Disassembler (`unilang compile`) | ✅ Done | Human-readable bytecode dump |

### Milestone 2.2: Stack-Based VM ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| VM core (stack, call frames) | ✅ Done | `unilang-runtime`: VM struct |
| Arithmetic & logic opcodes | ✅ Done | Int, Float, String coercion |
| Variable load/store | ✅ Done | Globals dict + local slots |
| Function call dispatch | ✅ Done | User-defined + builtins |
| Class instantiation | ✅ Done | `self`, field access, method calls |
| Exception handling (`try/except`) | ✅ Done | |
| Builtin registry | ✅ Done | `vm.register_builtin(name, fn)` |
| HTTP server builtin (`serve`) | ✅ Done | `serve(port, router)` — blocks, handles requests |

### Milestone 2.3: Standard Library ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| I/O: `print`, `input`, `read_file`, `write_file` | ✅ Done | `unilang-stdlib` |
| Math: `abs`, `round`, `floor`, `ceil`, `sqrt`, `min`, `max`, `pow` | ✅ Done | |
| String: `len`, `upper`, `lower`, `strip`, `split`, `join`, `replace`, `contains`, `starts_with`, `ends_with`, `format` | ✅ Done | |
| Collections: `append`, `pop`, `keys`, `values`, `has_key`, `range`, `sorted`, `reversed` | ✅ Done | |
| JSON: `to_json`, `from_json` | ✅ Done | |
| Type conversions: `int`, `float`, `str`, `bool`, `type_of` | ✅ Done | |
| Time: `now`, `sleep` | ✅ Done | |
| HTTP: `http_get`, `http_post`, `http_put`, `http_delete` | ✅ Done | |
| Env: `env_get`, `env_set` | ✅ Done | |
| File: `file_exists`, `file_size`, `list_dir` | ✅ Done | |
| Random: `random`, `random_int` | ✅ Done | |

### Milestone 2.4: Driver Ecosystem ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| `UniLangDriver` trait + `DriverRegistry` | ✅ Done | `unilang-drivers/src/lib.rs` |
| SQLite driver (`db_connect`, `db_query`, `db_exec`) | ✅ Done | via `rusqlite` (bundled) |
| Redis driver (13 functions: `redis_*`) | ✅ Done | via `redis` crate |
| Kafka driver (in-memory: `kafka_produce`, `kafka_events`) | ✅ Done | No external broker needed |
| Elasticsearch driver (8 functions: `es_*`) | ✅ Done | HTTP via `ureq` |
| MySQL driver (`mysql_*`) | ✅ Done | Feature-gated (`mysql-driver`) |
| PostgreSQL driver (`pg_*`) | ✅ Done | Feature-gated (`postgres-driver`) |
| MongoDB driver (`mongo_*`) | ✅ Done | Feature-gated (`mongodb-driver`) |
| Memcached driver (`memcached_*`) | ✅ Done | Feature-gated (`memcached-driver`) |
| Driver documentation | ✅ Done | `docs/DRIVERS.md` |

### Milestone 2.5: JVM/CPython Bridge ✅ Complete

| Task | Status | Notes |
|------|--------|-------|
| JNI bridge implementation | ✅ Done | `crates/unilang-bridge/src/jvm.rs` — `jni 0.21`; call_static, call_instance, import_class, load_jar, get_field, new_instance |
| CPython C API bridge | ✅ Done | `crates/unilang-bridge/src/cpython.rs` — `pyo3 0.22`; import_module, call_function, call_method, get_attribute, eval, exec |
| Type marshaling (all types) | ✅ Done | `crates/unilang-bridge/src/types.rs` — full RuntimeValue ↔ BridgeValue ↔ JValue/PyObject, no panics |
| Zero-copy array bridge | ✅ Done | `crates/unilang-bridge/src/arrays.rs` — SharedArrayBuffer, JNI typed arrays, numpy buffer protocol |
| Cross-VM exception handling | ✅ Done | JNI/PyErr → `BridgeError::CrossVmException { source, message }` in all bridge calls |
| Bridge performance benchmarks | ✅ Done | `crates/unilang-bridge/benches/bridge_bench.rs` — Criterion: int/string/list/dict marshal + round-trips |
| Java thread pool integration | ✅ Done | `crates/unilang-bridge/src/thread_pool.rs` — JavaThreadPool via ExecutorService (submit/await/shutdown) |
| Python `import` resolution | ✅ Done | `CpythonBridge::add_to_sys_path` + `py_path_add` builtin — prepends path to `sys.path` for PyPI packages |

**Exit Criteria:** Full end-to-end execution of real-world apps (DB + cache + HTTP + AI). ✅

---

## Phase 3: Toolchain

**Goal:** Complete developer-facing CLI tools and build infrastructure.

| Task | Status | Notes |
|------|--------|-------|
| `unilang lex <file>` | ✅ Done | Token stream dump |
| `unilang parse <file>` | ✅ Done | AST pretty-print |
| `unilang check <file>` | ✅ Done | Diagnostics only, no execution |
| `unilang compile <file>` | ✅ Done | Bytecode disassembly |
| `unilang run <file>` | ✅ Done | Full pipeline: lex → parse → analyze → compile → execute |
| `unilang test` command | 🔲 TODO | Pytest-style test runner |
| `unilang fmt` command | 🔲 TODO | Source formatter |
| `unilang lint` command | 🔲 TODO | Static analysis rules |
| `unilang new` command | ✅ Done | Core Team | Interactive TUI wizard + flags; generates `unilang.toml`, `src/main.uniL`, `.gitignore`, `README.md` |
| `unilang driver` command | ✅ Done | Core Team | `driver list` (table of all registered drivers) + `driver new <name>` (generates community driver template) |
| Community driver auto-discovery | ✅ Done | Core Team | Drop a `.rs` file in `src/community/` — `build.rs` auto-registers it; no `lib.rs` / `Cargo.toml` edits required |
| `unilang.toml` parser | 🔲 TODO | Project config, deps, features |
| `unilang.lock` generator | 🔲 TODO | Reproducible builds |
| Artifact packaging | 🔲 TODO | ZIP-based `.uniLpkg` archive |
| Incremental compilation | 🔲 TODO | File hash tracking |
| REPL / interactive mode | 🔲 TODO | `unilang repl` — v1.1 target |

**Exit Criteria:** Developer can `unilang init`, write code, and `unilang run` a new project from scratch.

---

## Phase 4: IDE & Ecosystem

**Goal:** IDE support, documentation, and real-world examples.

### IDE & Language Server

| Task | Status | Notes |
|------|--------|-------|
| Language Server Protocol (LSP) | ✅ Done | `crates/unilang-lsp/` |
| VS Code extension | ✅ Done | `tools/vscode-extension/` — syntax highlighting, snippets |
| JetBrains plugin | ✅ Done | `tools/jetbrains-plugin/` — IntelliJ/PyCharm |
| Eclipse plugin | ✅ Done | `tools/eclipse-plugin/` |
| Standalone UniLang IDE | ✅ Done | `tools/unilang-ide/` (Electron-based) |
| Hover documentation | 🔲 TODO | LSP hover provider |
| Go to definition | 🔲 TODO | LSP definition provider |
| Code formatting integration | 🔲 TODO | LSP formatting (needs `unilang fmt`) |
| Debugging support (DAP) | 🔲 TODO | v1.1 target |
| Snippet library (expanded) | 🔲 TODO | More patterns beyond basics |

### Documentation & Examples

| Task | Status | Notes |
|------|--------|-------|
| Language tutorial / Quick Start | ✅ Done | `docs/guides/QUICKSTART.md` |
| API reference (stdlib) | ✅ Done | `docs/guides/API_REFERENCE.md` — 113 functions, all modules, examples |
| Driver reference | ✅ Done | `docs/DRIVERS.md` |
| Community driver contribution guide | ✅ Done | `CONTRIBUTING_DRIVERS.md` — auto-discovery walkthrough + template |
| Compiler pipeline docs | ✅ Done | `docs/architecture/COMPILER_PIPELINE.md` |
| Example: ML framework | ✅ Done | `examples/ml-framework/` — custom Tensor, layers, UniNN |
| Example: Library management | ✅ Done | `examples/library-mgmt/` — REST API, 10K books, ML predictions |
| Example: SHYNX e-commerce | ✅ Done | `examples/ecommerce/` — SQLite + Redis + Kafka + AI recs |
| Example: Web service | ✅ Done | `examples/web-service/` — task management REST API, SQLite, 7 endpoints |
| Example: Data processing | ✅ Done | `examples/data-pipeline/` — ETL pipeline, CSV parse, SQLite load, aggregates |
| Cookbook / recipes | ✅ Done | `docs/guides/COOKBOOK.md` — 8 recipes: auth, pagination, caching, rate limiting, Kafka jobs, validation, error middleware, env config |
| Migration guide (Java → UniLang) | ✅ Done | `docs/guides/MIGRATION_JAVA.md` — syntax table, stdlib mapping, Spring Boot example |
| Migration guide (Python → UniLang) | ✅ Done | `docs/guides/MIGRATION_PYTHON.md` — what works unchanged, stdlib mapping, Flask example |

**Exit Criteria:** VS Code extension published; 3+ complete example projects. ✅

---

## Phase 5: Production Ready

**Goal:** Stability, performance, and public release.

| Task | Status | Notes |
|------|--------|-------|
| Test suite (compiler + VM) | 🔲 TODO | Unit + integration + e2e |
| Performance benchmark suite | 🔲 TODO | Compilation speed, VM throughput |
| Stress testing (large programs) | 🔲 TODO | 10K+ LOC programs |
| Security audit (VM sandbox) | 🔲 TODO | |
| Memory leak testing | 🔲 TODO | Long-running HTTP servers |
| Cross-platform testing | 🔲 TODO | Linux, macOS, Windows |
| Release candidate process | 🔲 TODO | Semantic versioning + GitHub releases |
| Binary distribution | 🔲 TODO | Pre-built binaries for all platforms |
| Apache incubation submission | 🔲 TODO | |
| Public announcement / launch | 🔲 TODO | Blog post, HN, Reddit |

**Exit Criteria:** v1.0.0 release published; all tests green; Apache incubation proposal submitted.

---

## Long-term Vision (Post v1.0)

| Feature | Target Version | Notes |
|---------|----------------|-------|
| REPL / interactive mode | v1.1 | `unilang repl` |
| Debug Adapter Protocol (DAP) | v1.1 | Breakpoints in `.uniL` files |
| Jupyter kernel | v1.2 | Run UniLang cells in notebooks |
| JVM backend (real Java interop) | v2.0 | Emit `.class` files, call JVM libraries |
| CPython bridge (real Python interop) | v2.0 | `import numpy`, `import sklearn` |
| GraalVM AOT compilation | v2.0 | Single native binary, no JVM needed |
| Package registry | v2.0 | `unilang.dev` — publish/install packages |
| Custom IDE | v2.0 | Based on Phase 4 research |
| Cloud deployment tools | v2.0 | Deploy UniLang apps to cloud |
| WebAssembly target | v2.x | Run in browser |
| JavaScript/TypeScript interop | v3.0 | Full three-language support |

---

*Dates are estimates and will be adjusted based on community capacity. Status updated 2026-04-16.*
