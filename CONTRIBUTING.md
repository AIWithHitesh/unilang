# Contributing to UniLang

Thank you for your interest in contributing to UniLang! This document provides guidelines and information for contributors.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Writing a Driver](#writing-a-driver)
6. [Coding Standards](#coding-standards)
7. [Commit Guidelines](#commit-guidelines)
8. [Pull Request Process](#pull-request-process)
9. [RFC Process](#rfc-process)
10. [Issue Guidelines](#issue-guidelines)
11. [Community](#community)
12. [License](#license)

---

## Code of Conduct

This project follows the [Apache Software Foundation Code of Conduct](https://www.apache.org/foundation/policies/conduct.html). By participating, you agree to uphold this code. Report unacceptable behavior to conduct@unilang.apache.org.

---

## Getting Started

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Compiler development |
| Cargo | Latest | Rust build system |
| JDK | 21+ | JVM runtime & testing |
| Python | 3.11+ | Python runtime & testing |
| Git | 2.40+ | Version control |
| Make | Any | Build orchestration |

### First-time Setup

```bash
# 1. Fork the repository on GitHub

# 2. Clone your fork
git clone https://github.com/<your-username>/unilang.git
cd unilang

# 3. Add upstream remote
git remote add upstream https://github.com/apache/unilang.git

# 4. Install development dependencies
make dev-setup

# 5. Verify your setup
make check

# 6. Run the test suite
make test
```

---

## Development Setup

### Project Structure

```
unilang/
├── src/
│   ├── compiler/           # Compiler implementation (Rust)
│   │   ├── lexer/          # Tokenization
│   │   ├── parser/         # Parsing and AST
│   │   ├── semantic/       # Type checking and analysis
│   │   ├── codegen/        # Code generation (JVM + Python)
│   │   └── optimizer/      # Optimization passes
│   ├── runtime/            # Runtime implementation
│   │   ├── jvm/            # JVM integration
│   │   ├── python/         # CPython integration
│   │   ├── threading/      # Thread management
│   │   └── interop/        # Bridge layer
│   └── stdlib/             # Standard library
├── tests/
│   ├── unit/               # Unit tests
│   ├── integration/        # Integration tests
│   ├── e2e/                # End-to-end tests
│   └── benchmarks/         # Performance benchmarks
├── tools/
│   ├── cli/                # CLI tool
│   ├── vscode-extension/   # VS Code extension
│   ├── formatter/          # Code formatter
│   └── linter/             # Linter
├── examples/               # Example programs
├── docs/                   # Documentation
└── config/                 # Configuration files
```

### Building

```bash
# Full build
make build

# Debug build (faster, with debug symbols)
make build-debug

# Build specific component
make build-lexer
make build-parser
make build-runtime

# Run all tests
make test

# Run specific test suite
make test-lexer
make test-parser
make test-integration

# Run benchmarks
make bench

# Format code
make fmt

# Lint
make lint

# Full CI check (fmt + lint + test + build)
make ci
```

---

## How to Contribute

### Types of Contributions

| Type | Description | Good First Issue? |
|------|-------------|-------------------|
| **Bug fix** | Fix a confirmed bug | Often yes |
| **Feature** | Implement a new language feature | Usually no |
| **Documentation** | Improve docs, tutorials, examples | Yes |
| **Test** | Add missing test coverage | Yes |
| **Performance** | Improve compilation or runtime speed | No |
| **Tooling** | Improve CLI, formatter, linter | Sometimes |
| **RFC** | Propose a design change | No |

### Finding Work

1. Check [GitHub Issues](https://github.com/apache/unilang/issues) labeled `good-first-issue`
2. Check the [Roadmap](docs/planning/ROADMAP.md) for TODO items
3. Run the test suite and look for gaps in coverage
4. Try writing UniLang programs and report issues you find

### Contribution Workflow

```
1. Pick an issue (or create one)
         │
2. Comment on the issue to claim it
         │
3. Create a feature branch
         │  git checkout -b feature/my-feature
         │
4. Make your changes
         │
5. Write/update tests
         │
6. Run `make ci` locally
         │
7. Commit with conventional message
         │
8. Push and create a Pull Request
         │
9. Address review feedback
         │
10. Merge (maintainer)
```

---

## Writing a Driver

UniLang's driver ecosystem (`crates/unilang-drivers/`) lets you add new database, cache, queue, or search-engine connectivity without touching VM or standard-library code.

For the full walkthrough, see **[docs/DRIVERS.md](docs/DRIVERS.md)**. The quick version:

### 1. Create your driver file

```rust
// crates/unilang-drivers/src/mydb.rs
use std::sync::{Arc, Mutex};
use unilang_runtime::vm::VM;
use crate::{DriverCategory, UniLangDriver};

pub struct MyDbDriver {
    conn: Arc<Mutex<Option<mydb::Connection>>>,
}

impl MyDbDriver {
    pub fn new() -> Self {
        Self { conn: Arc::new(Mutex::new(None)) }
    }
}

impl UniLangDriver for MyDbDriver {
    fn name(&self)        -> &'static str { "mydb" }
    fn version(&self)     -> &'static str { "1.0.0" }
    fn description(&self) -> &'static str { "MyDB driver for UniLang" }
    fn category(&self)    -> DriverCategory { DriverCategory::Database }

    fn exported_functions(&self) -> &'static [&'static str] {
        &["mydb_connect", "mydb_query", "mydb_close"]
    }

    fn register(&self, vm: &mut VM) {
        let conn = Arc::clone(&self.conn);
        vm.register_builtin("mydb_connect", move |args| {
            // open connection, store in conn
            Ok(unilang_runtime::value::Value::Bool(true))
        });
        // ... register mydb_query, mydb_close
    }
}
```

### 2. Add the dependency (optional — feature-gate it)

```toml
# crates/unilang-drivers/Cargo.toml
mydb = { version = "1", optional = true }

[features]
mydb-driver = ["dep:mydb"]
all-drivers = [..., "mydb-driver"]
```

### 3. Wire it into lib.rs

```rust
// crates/unilang-drivers/src/lib.rs
#[cfg(feature = "mydb-driver")]
mod mydb;

pub fn default_registry() -> DriverRegistry {
    let mut r = DriverRegistry::new();
    // ...
    #[cfg(feature = "mydb-driver")]
    r.register(Box::new(mydb::MyDbDriver::new()));
    r
}
```

### 4. Add function names to the semantic analyzer

Open `crates/unilang-semantic/src/analyzer.rs` and add your new function names to the `PRELUDE_FUNCS` static slice so the type-checker knows they exist at compile time.

### 5. Open a PR

The PR description should include:
- What database/service is targeted
- Which Cargo feature flag enables it
- A minimal `.uniL` snippet showing usage

---

## Coding Standards

### Rust (Compiler & Runtime)

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with the project's `.rustfmt.toml` configuration
- Use `clippy` with no warnings: `cargo clippy -- -D warnings`
- All public APIs must have doc comments (`///`)
- Prefer `Result<T, E>` over panics; compiler must never panic on user code
- Use meaningful error types, not `String` errors
- Test coverage target: >90% for compiler phases

### Documentation

- API documentation follows rustdoc conventions
- User-facing docs use Markdown
- Code examples in docs must compile and pass tests
- Grammar changes must update `LANGUAGE_SPEC.md`

### Testing

```
Unit tests:        Test individual functions and modules
Integration tests: Test compiler phases working together
End-to-end tests:  Test full compilation + execution of .uniL programs
Benchmarks:        Track performance over time
Snapshot tests:    AST/UIR output stability
```

Every PR must include tests for the changes it introduces.

---

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `test` | Adding or updating tests |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvement |
| `ci` | CI/CD changes |
| `build` | Build system changes |
| `chore` | Other changes that don't modify src or test files |

### Scopes

| Scope | Component |
|-------|-----------|
| `lexer` | Lexer/tokenizer |
| `parser` | Parser/AST |
| `semantic` | Semantic analyzer |
| `codegen` | Code generation |
| `runtime` | Runtime/bridge |
| `cli` | CLI tool |
| `vscode` | VS Code extension |
| `spec` | Language specification |

### Examples

```
feat(lexer): add f-string tokenization support
fix(parser): handle mixed indent/brace blocks correctly
docs(spec): clarify semicolon insertion rules
test(semantic): add cross-VM type inference tests
perf(codegen): optimize bridge call stub generation
```

---

## Pull Request Process

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass (`make test`)
- [ ] Code is formatted (`make fmt`)
- [ ] Linter passes (`make lint`)
- [ ] New code has tests
- [ ] Documentation is updated (if applicable)
- [ ] Commit messages follow convention
- [ ] PR description explains the "why"

### PR Template

```markdown
## Summary
Brief description of what this PR does and why.

## Changes
- Bullet list of specific changes

## Testing
How was this tested? What tests were added?

## Related Issues
Closes #123
```

### Review Process

1. All PRs require **at least 2 approvals** from maintainers
2. CI must pass (build, test, lint, fmt)
3. No unresolved conversations
4. Maintainer merges via **squash merge** (clean history)

### Review SLA

| PR Size | Target Review Time |
|---------|--------------------|
| Small (< 100 LOC) | 2 business days |
| Medium (100-500 LOC) | 5 business days |
| Large (> 500 LOC) | 10 business days |

---

## RFC Process

Major changes to the language, compiler architecture, or runtime require a Request for Comments (RFC):

1. **Create RFC document** in `docs/rfcs/NNNN-title.md`
2. **Open a GitHub Discussion** tagged `rfc`
3. **Community review period:** minimum 2 weeks
4. **Decision:** requires 3 maintainer approvals
5. **Implementation:** reference RFC in the implementing PR

### RFC Template

```markdown
# RFC NNNN: Title

## Summary
One-paragraph explanation.

## Motivation
Why are we doing this? What problem does it solve?

## Detailed Design
Technical details of the proposed change.

## Alternatives Considered
What other approaches were evaluated?

## Drawbacks
What are the trade-offs?

## Unresolved Questions
What needs further discussion?
```

---

## Issue Guidelines

### Bug Reports

Use the **Bug Report** template and include:
- UniLang version
- OS and architecture
- Minimal `.uniL` reproduction case
- Expected vs actual behavior
- Error message (full output)

### Feature Requests

Use the **Feature Request** template and include:
- Use case description
- Proposed syntax/API
- Alternatives considered

---

## Community

| Channel | Purpose |
|---------|---------|
| GitHub Issues | Bug reports, feature requests |
| GitHub Discussions | Questions, RFCs, general discussion |
| Mailing List (planned) | dev@unilang.apache.org |

### Maintainers

Core maintainers are listed in the `MAINTAINERS.md` file. To become a maintainer:

1. Sustained, high-quality contributions over 6+ months
2. Demonstrated understanding of the project's goals and architecture
3. Nomination by an existing maintainer
4. Vote by the maintainer team (majority approval)

---

## License

By contributing to UniLang, you agree that your contributions will be licensed under the Apache License 2.0. You must sign the [Apache Individual Contributor License Agreement (ICLA)](https://www.apache.org/licenses/icla.pdf) before your first contribution can be merged.

All source files must include the Apache license header:

```
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.
```

---

*Thank you for contributing to UniLang! Every contribution, no matter how small, helps make this project better.*
