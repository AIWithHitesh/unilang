# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.5.0] - 2026-04-17

### Added
- Phase 3 Toolchain completions: `unilang test`, `unilang fmt`, `unilang lint`, `unilang repl`,
  `unilang pack`, `unilang build --incremental`
- `unilang config show|init` — project configuration management via `unilang.toml`
- `unilang lock` — dependency lock file generator with SHA-256 per-dep checksums
- `unilang repl` — interactive loop with block-continuation detection (`>>>` / `...` prompts)
- `unilang pack [--out <file>]` — ZIP-based `.uniLpkg` archive of `.uniL` files + `unilang.toml`
- `unilang build --incremental` — SHA-256 hash cache in `.unilang_cache/`; skips unchanged files
- Phase 4 IDE: LSP hover for 80+ keywords and 60+ stdlib functions
- LSP `goto_definition` — scans for `def`/`class`/`val`/`var` declarations across the workspace
- LSP `textDocument/formatting` — in-process formatter returns full-document `TextEdit`
- VS Code: 15 new snippets (while, switch, match, sqlite, redis, httpserver, test, dataclass, etc.)
- VS Code: DAP debugger contribution with launch config for `.uniL` files
- Phase 5: `ExecutionLimits` sandbox API — default/sandboxed/development profiles
- Phase 5: Comprehensive test suite (parser, semantic, runtime tests; 500+ test cases)
- Phase 5: Criterion benchmarks for compiler pipeline and VM throughput
- Phase 5: Stress programs in `tests/stress/` (300–500 line programs each)
- Phase 5: CI matrix expanded to ubuntu-latest, macos-13, macos-latest, windows-latest
- Phase 5: Valgrind + LSAN CI job for memory leak detection
- SHYNX e-commerce example (`examples/ecommerce/`) — SQLite + Redis + Kafka + AI recommendations
- `SECURITY.md` with vulnerability disclosure policy and sandbox profile documentation
- Release automation: `release.yml` — lite + full editions, installers, signed macOS binaries
- `docs/governance/RELEASE_PROCESS.md` — RC tagging, checklist, hotfix process
- `docs/governance/INCUBATION_PROPOSAL.md` — full Apache Software Foundation proposal draft
- `docs/announcement/LAUNCH_POST.md` — launch blog post

### Changed
- Phase 5 progress bar updated to Complete in ROADMAP.md
- Version bumped to 1.5.0 across workspace

### Fixed
- ML predictions tab and train model button in library-mgmt example
- Prediction engine and book model logic in server

---

## [1.4.0] - 2026-04-16

### Added
- Community driver auto-discovery: drop a `.rs` file in `src/community/` — `build.rs` auto-registers
  it; no `lib.rs` / `Cargo.toml` edits required
- `unilang driver list` — table of all registered drivers with status and version
- `unilang driver new <name>` — generates community driver template
- JVM/CPython bridge (`crates/unilang-bridge`):
  - JNI bridge (`jni 0.21`): `call_static`, `call_instance`, `import_class`, `load_jar`,
    `get_field`, `new_instance`
  - CPython C API bridge (`pyo3 0.22`): `import_module`, `call_function`, `call_method`,
    `get_attribute`, `eval`, `exec`
  - Full type marshaling (`RuntimeValue` ↔ `BridgeValue` ↔ `JValue`/`PyObject`)
  - Zero-copy array bridge (`SharedArrayBuffer`, JNI typed arrays, numpy buffer protocol)
  - Cross-VM exception handling (`JNI`/`PyErr` → `BridgeError::CrossVmException`)
  - Java thread pool integration via `ExecutorService` (`submit`/`await`/`shutdown`)
  - `py_path_add` builtin — prepends path to `sys.path` for PyPI packages
  - Bridge performance benchmarks (Criterion): int/string/list/dict marshal + round-trips
- ML framework example (`examples/ml-framework/`) — custom Tensor, layers, UniNN neural net
- Library management example (`examples/library-mgmt/`) — REST API, 10K books, ML predictions
- Hover documentation expanded to 80+ language keywords
- Additional drivers: RabbitMQ, NATS, Prometheus, InfluxDB, SMTP, S3, WebSocket
- `CONTRIBUTING_DRIVERS.md` — auto-discovery walkthrough and driver template guide

### Changed
- `unilang-drivers` crate restructured: all drivers now feature-gated for lean default build
- Bridge replaces placeholder stub in `unilang-bridge` with full production implementation

### Fixed
- Symbol resolution for cross-module imports in semantic analyzer
- Stack frame cleanup on exception unwind in VM

---

## [1.3.0] - 2026-04-10

### Added
- LSP server (`crates/unilang-lsp`) using `tower-lsp 0.20`
- VS Code extension (`tools/vscode-extension/`) — syntax highlighting, initial snippet library
- JetBrains plugin (`tools/jetbrains-plugin/`) — IntelliJ / PyCharm support
- Eclipse plugin (`tools/eclipse-plugin/`)
- Standalone UniLang IDE (`tools/unilang-ide/`) — Electron-based editor
- Language tutorial and Quick Start guide (`docs/guides/QUICKSTART.md`)
- API reference for all 113 stdlib functions (`docs/guides/API_REFERENCE.md`)
- Compiler pipeline documentation (`docs/architecture/COMPILER_PIPELINE.md`)
- Cookbook with 8 recipes: auth, pagination, caching, rate limiting, Kafka jobs, validation,
  error middleware, env config (`docs/guides/COOKBOOK.md`)
- Migration guide: Java → UniLang (`docs/guides/MIGRATION_JAVA.md`)
- Migration guide: Python → UniLang (`docs/guides/MIGRATION_PYTHON.md`)
- Web service example (`examples/web-service/`) — task management REST API, SQLite, 7 endpoints
- Data pipeline example (`examples/data-pipeline/`) — ETL pipeline, CSV parse, SQLite load
- `unilang new` command — interactive TUI wizard; generates `unilang.toml`, `src/main.uniL`,
  `.gitignore`, `README.md`

### Changed
- Semantic analyzer prelude expanded from 35 to 50+ recognized stdlib function names
- LSP server replaces the `diagnostics_only` stub with full request/response handling

### Fixed
- Parser error recovery on malformed class body no longer swallows subsequent declarations
- `for` loop variable leaking into outer scope in semantic analyzer

---

## [1.2.0] - 2026-03-28

### Added
- PostgreSQL driver (`pg_connect`, `pg_query`, `pg_exec`, `pg_transaction`) via `tokio-postgres`
- MongoDB driver (`mongo_connect`, `mongo_find`, `mongo_insert`, `mongo_update`, `mongo_delete`)
  via `mongodb` crate
- Memcached driver (`memcached_connect`, `memcached_get`, `memcached_set`, `memcached_delete`)
- MySQL driver (`mysql_connect`, `mysql_query`, `mysql_exec`) via `mysql_async`
- Overload resolution in semantic analyzer — multiple same-name functions; best-match scoring;
  gradual fallback to `Dynamic`
- Generic type checking: `List<T>`, `Map<K,V>`, `Option<T>`; element-type checking for `append`
- `unilang check <file>` — diagnostics only, no execution (fast feedback loop)
- `unilang compile <file>` — bytecode disassembly output for inspection
- HTTP builtins: `http_get`, `http_post`, `http_put`, `http_delete` via `ureq`
- Env builtins: `env_get`, `env_set`
- File builtins: `file_exists`, `file_size`, `list_dir`
- Random builtins: `random`, `random_int`
- Time builtins: `now`, `sleep`
- JSON builtins: `to_json`, `from_json`

### Changed
- `DriverRegistry` now supports feature-gated optional drivers; core build only requires SQLite
- Semantic analyzer error messages now include span labels with column ranges

### Fixed
- Integer overflow in constant pool indexing for programs with many string literals
- Incorrect precedence for unary minus in Pratt parser

---

## [1.1.0] - 2026-03-14

### Added
- SQLite driver (`db_connect`, `db_query`, `db_exec`, `db_close`) via `rusqlite` (bundled)
- Redis driver: 13 functions (`redis_connect`, `redis_get`, `redis_set`, `redis_del`,
  `redis_lpush`, `redis_rpush`, `redis_lrange`, `redis_hset`, `redis_hget`, `redis_hgetall`,
  `redis_expire`, `redis_ttl`, `redis_exists`) via `redis` crate
- Kafka driver (in-memory implementation): `kafka_produce`, `kafka_consume`, `kafka_events`,
  `kafka_commit` — no external broker required for development
- Elasticsearch driver (8 functions: `es_connect`, `es_index`, `es_search`, `es_get`,
  `es_update`, `es_delete`, `es_bulk`, `es_count`) via HTTP/`ureq`
- `UniLangDriver` trait and `DriverRegistry` in `unilang-drivers/src/lib.rs`
- HTTP server builtin (`serve(port, router)`) — blocks, handles requests, routes by path/method
- `serve` builtin supports `GET`, `POST`, `PUT`, `DELETE` handler registration
- `unilang run <file>` — full pipeline: lex → parse → analyze → compile → execute
- `unilang lex <file>` — token stream dump for debugging
- `unilang parse <file>` — AST pretty-print

### Changed
- VM now supports a `register_builtin(name, fn)` API for driver self-registration
- Stack frame size increased to 4096 slots (was 256) for real-world programs

### Fixed
- Class method dispatch incorrectly binding `self` for inherited methods
- `try/except` block not restoring stack depth on clean exit path

---

## [1.0.0] - 2026-02-28

### Added
- Stack-based VM (`crates/unilang-runtime`) — core evaluation engine
  - VM struct with stack, call frames, globals dict
  - Arithmetic and logic opcodes with Int/Float/String coercion
  - Variable load/store (globals dict + local slots)
  - Function call dispatch (user-defined + builtins)
  - Class instantiation, field access, method calls, `self` binding
  - Exception handling (`try/except/finally`)
  - Builtin function registry
- Bytecode compiler (`crates/unilang-codegen`):
  - 40+ opcode instruction set (`OpCode` enum)
  - AST → bytecode lowering for all expression and statement forms
  - Constant pool (string/int/float constants)
  - Function and class compilation
  - Human-readable disassembler
- Standard library (`crates/unilang-stdlib`):
  - I/O: `print`, `input`, `read_file`, `write_file`
  - Math: `abs`, `round`, `floor`, `ceil`, `sqrt`, `min`, `max`, `pow`
  - String: `len`, `upper`, `lower`, `strip`, `split`, `join`, `replace`,
    `contains`, `starts_with`, `ends_with`, `format`
  - Collections: `append`, `pop`, `keys`, `values`, `has_key`,
    `range`, `sorted`, `reversed`
  - Type conversions: `int`, `float`, `str`, `bool`, `type_of`
- Semantic analyzer (`crates/unilang-semantic`):
  - Nested scope stack with symbol table
  - Gradual type inference (Int, Float, String, Bool, Array, Dynamic)
  - Prelude / standard function resolution (35+ stdlib names)
  - Import resolution (marked as Dynamic for interop)
  - Span-based diagnostics with labels
- `unilang-common` — shared `Span`, `Diagnostic`, `DiagnosticLevel` types
- Basic examples: `examples/basic/`, `examples/advanced/`

### Changed
- Workspace version aligned to `1.0.0` in `Cargo.toml`

---

## [0.4.0] - 2026-01-31

### Added
- Full expression parser (Pratt / precedence climbing):
  - Binary operators with correct precedence and associativity
  - Unary operators (`-`, `not`, `!`)
  - Function calls, index access (`a[i]`), member access (`a.b`)
  - Lambda expressions (`lambda x: x + 1`)
  - List and dict literals
  - Ternary / conditional expressions (`x if cond else y`)
- Control flow parsing: `if/elif/else`, `while`, `for ... in ...`,
  `break`, `continue`, `return`
- Exception handling: `try/except/finally` (Python) and `try/catch/finally` (Java)
- Class declarations: Java-style field declarations + Python-style `def` methods
- Function declarations: `def fn():` (Python) and `void fn() {}` (Java)
- Import statements: `import x`, `from x import y`, `from x import *`
- Parser error recovery: re-sync on statement boundary to continue reporting errors
- AST node definitions: `Expr`, `Stmt`, `Module`, `ClassDecl`, `FuncDecl`

### Fixed
- Indentation tokenizer treating blank lines inside blocks as block-end
- f-string parsing failing on nested curly braces

---

## [0.3.0] - 2026-01-17

### Added
- Unified lexer (`crates/unilang-lexer`) supporting both Python and Java token sets:
  - Python keywords: `def`, `class`, `import`, `from`, `pass`, `lambda`, `with`, `yield`
  - Java keywords: `public`, `private`, `protected`, `void`, `static`, `new`, `extends`,
    `implements`, `interface`, `abstract`
  - Shared keywords: `if`, `else`, `while`, `for`, `return`, `break`, `continue`, `try`,
    `except`/`catch`, `finally`, `true`/`false`/`null`/`None`
  - Indentation-aware tokenization (INDENT/DEDENT tokens for Python blocks)
  - String literal handling: single/double quoted, triple-quoted, f-strings (`f"..."`)
  - Comment handling: `//`, `#`, `/* ... */`
  - Number literals: integer and floating-point
  - Unknown character diagnostic with span
- `unilang-common` crate with `Span`, `SourceFile`, `Diagnostic` types
- `unilang-parser` crate scaffolding — statement parser stub

### Changed
- Repository structure reorganised into `crates/` workspace layout
- Build system updated to Rust workspace with `resolver = "2"`

---

## [0.2.0] - 2026-01-05

### Added
- Formal grammar specification (`docs/specifications/GRAMMAR.ebnf`) — 136 EBNF rules covering
  the full Python + Java language surface
- Architecture design document (`docs/architecture/ARCHITECTURE.md`)
- Compiler pipeline documentation (`docs/architecture/COMPILER_PIPELINE.md`)
- Language specification draft (`docs/specifications/LANGUAGE_SPEC.md`)
- Design decisions document (`docs/design/DESIGN_DECISIONS.md`)
- `CONTRIBUTING.md` with code of conduct, development workflow, and driver contribution guide
- `docs/DRIVERS.md` — driver architecture and development guide
- GitHub Actions CI pipeline (build + test on push/PR)
- Install scripts: `install.sh` (Linux/macOS), `install.ps1` (Windows PowerShell)
- `Makefile` with `build`, `test`, `fmt`, `lint`, `clean` targets

### Changed
- License confirmed as Apache License 2.0; NOTICE and file headers added

---

## [0.1.0] - 2025-12-20

### Added
- Initial project scaffolding: Rust workspace, `Cargo.toml`, directory structure
- Project README with vision statement and high-level feature overview
- `docs/planning/PRD.md` — product requirements document
- `docs/planning/ROADMAP.md` — phased development roadmap (Phase 0 through Phase 5)
- Apache License 2.0 (`LICENSE`, `NOTICE`)
- `unilang.toml` — project manifest format definition
- Initial `src/` stub — entry point for the `unilang` binary
- Git repository initialised, `.gitignore` configured for Rust projects

[Unreleased]: https://github.com/AIWithHitesh/unilang/compare/v1.5.0...HEAD
[1.5.0]: https://github.com/AIWithHitesh/unilang/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/AIWithHitesh/unilang/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/AIWithHitesh/unilang/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/AIWithHitesh/unilang/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/AIWithHitesh/unilang/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/AIWithHitesh/unilang/compare/v0.4.0...v1.0.0
[0.4.0]: https://github.com/AIWithHitesh/unilang/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/AIWithHitesh/unilang/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/AIWithHitesh/unilang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/AIWithHitesh/unilang/releases/tag/v0.1.0
