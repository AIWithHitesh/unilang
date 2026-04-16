# UniLang

**A unified programming language that seamlessly integrates Python and Java syntax, enabling developers to leverage the best of both ecosystems in a single codebase.**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/AIWithHitesh/unilang/actions/workflows/ci.yml/badge.svg)](https://github.com/AIWithHitesh/unilang/actions/workflows/ci.yml)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg)](CONTRIBUTING.md)

---

## Overview

UniLang is an open-source programming language designed to bridge the gap between Java and Python. It allows developers to:

- **Write Java within Python** — Access Java's enterprise-grade concurrency, type system, and JVM ecosystem directly from Python-style code.
- **Write Python within Java** — Leverage Python's ML/AI libraries (NumPy, TensorFlow, PyTorch, scikit-learn) seamlessly within Java applications.
- **Mix syntax freely** — Use Python indentation-based blocks or Java brace-delimited blocks interchangeably, even within the same file.
- **Full feature parity** — All Java features (multi-threading, generics, annotations) and all Python features (decorators, comprehensions, generators) work natively.

## Quick Example

```unilang
// UniLang: Java-style class with Python-style ML integration
import java.util.concurrent.ExecutorService
import numpy as np
from sklearn.linear_model import LinearRegression

public class MLPipeline {
    def train_model(self, data):
        X = np.array(data["features"])
        y = np.array(data["labels"])
        model = LinearRegression()
        model.fit(X, y)
        return model

    public void runPipeline() {
        ExecutorService executor = Executors.newFixedThreadPool(4);
        executor.submit(() -> {
            model = self.train_model(load_data())
            print(f"Model score: {model.score(X_test, y_test)}")
        });
    }
}
```

## File Extension

UniLang source files use the `.uniL` extension.

## Project Status

UniLang is in **active development** with a fully working compiler + runtime pipeline:

**Compiler Pipeline**
- [x] Lexer — hand-written, full Python + Java token support, f-strings, error recovery
- [x] Parser — Pratt expression parser + recursive descent statements, both indent and brace blocks
- [x] Semantic analyzer — gradual type system, scope resolution, name binding
- [x] Code generation — stack-based bytecode with 40+ opcodes
- [x] Runtime VM — stack-based interpreter with call frames, exception handling, class dispatch

**Standard Library & Drivers**
- [x] Standard library — 35+ built-in functions: math, strings, collections, I/O, JSON, HTTP, time
- [x] HTTP server — `serve(port, router)` built-in for writing REST APIs directly in UniLang
- [x] Driver ecosystem (`unilang-drivers`) — SQLite, Redis, Kafka, Elasticsearch (default); MySQL, PostgreSQL, MongoDB, Memcached (optional)

**Toolchain & IDE**
- [x] CLI — `unilang run`, `unilang check`, `unilang compile`, `unilang lex`
- [x] Language Server Protocol (LSP) server
- [x] IDE support — VS Code, JetBrains, Eclipse, standalone Electron IDE
- [x] CI/CD — GitHub Actions (build + lint + e2e on Linux, macOS, Windows)

**Examples**
- [x] [SHYNX e-commerce](examples/ecommerce/) — 100 products, SQLite + Redis + Kafka + AI recommendations
- [x] [Library Management](examples/library-mgmt/) — 10K book dataset, REST API, ML prediction engine
- [x] [ML Framework](examples/ml-framework/) — custom Tensor, autograd, layers, UniNN architecture

## Downloads & Installation

### Quick Install (recommended)

**macOS / Linux** — paste in Terminal:
```bash
# macOS Apple Silicon
curl -fsSL https://github.com/AIWithHitesh/unilang/releases/latest/download/unilang-cli-macos-arm64.tar.gz \
  | tar xz && sudo cp unilang-cli-macos-arm64/bin/unilang /usr/local/bin/unilang

# macOS Intel
curl -fsSL https://github.com/AIWithHitesh/unilang/releases/latest/download/unilang-cli-macos-x86_64.tar.gz \
  | tar xz && sudo cp unilang-cli-macos-x86_64/bin/unilang /usr/local/bin/unilang

# Linux x86_64
curl -fsSL https://github.com/AIWithHitesh/unilang/releases/latest/download/unilang-cli-linux-x86_64.tar.gz \
  | tar xz && sudo cp unilang-cli-linux-x86_64/bin/unilang /usr/local/bin/unilang
```

> **Why `curl`?** Browsers tag downloaded files with a macOS quarantine flag that triggers the *"Apple could not verify..."* warning. Installing via `curl` in Terminal skips that flag entirely — no workarounds needed.

**Windows** — download `unilang-cli-windows-x86_64.zip` from [Releases](https://github.com/AIWithHitesh/unilang/releases), extract, and add the `bin\` folder to your PATH.

**Build from source** (no binary download at all):
```bash
git clone https://github.com/AIWithHitesh/unilang.git
cd unilang
cargo build --release
sudo cp target/release/unilang /usr/local/bin/unilang
```

### Manual Download

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `unilang-cli-macos-arm64.tar.gz` |
| macOS (Intel) | `unilang-cli-macos-x86_64.tar.gz` |
| Windows | `unilang-cli-windows-x86_64.zip` |
| Linux | `unilang-cli-linux-x86_64.tar.gz` |

### IDE Plugins
| IDE | Download | Installation Guide |
|-----|----------|--------------------|
| VS Code / Cursor | `unilang-vscode.vsix` | [VS Code Plugin Guide](docs/guides/VSCODE_SETUP.md) |
| IntelliJ / PyCharm | `unilang-jetbrains.zip` | [JetBrains Plugin Guide](docs/guides/JETBRAINS_SETUP.md) |
| Eclipse | `unilang-eclipse.jar` | [Eclipse Plugin Guide](docs/guides/ECLIPSE_SETUP.md) |
| UniLang IDE (Standalone) | `.dmg` / `.exe` / `.AppImage` | [IDE Setup Guide](docs/guides/IDE_SETUP.md) |

## UniLang ML Framework

Build neural networks **from scratch** in UniLang — no PyTorch, no TensorFlow.

```unilang
from models.uniNN import UniNN
from core.loss import CrossEntropyLoss
from core.optimizers import Adam

model = UniNN(inputDim=10, hiddenDim=64, outputDim=3, task="classification")
loss_fn = CrossEntropyLoss()
optimizer = Adam(model.parameters(), lr=0.001)

for epoch in range(100):
    model.zero_grad()
    predictions = model.forward(X_train)
    loss = loss_fn.compute(predictions, y_train)
    loss.backward()
    optimizer.step()
```

**What's included:**
- Custom Tensor with autograd engine
- Layers: Linear, BatchNorm, Dropout, Embedding, LSTM, Conv1D, MaxPool1D
- Loss functions: MSE, CrossEntropy, BCE, Huber
- Optimizers: SGD, Adam, RMSProp + LR schedulers
- **UniNN** — original architecture with gated residual blocks and multi-scale feature mixing
- Time series support via LSTM and Conv1D
- Java thread pool for parallel ensemble inference

[**ML Framework Documentation**](examples/ml-framework/docs/README.md) | [**Source Code**](examples/ml-framework/)

## Examples

| Example | Description |
|---------|-------------|
| [SHYNX E-Commerce](examples/ecommerce/) | Full-stack fashion store: SQLite, Redis cache, Kafka events, AI recommendations, 12 REST endpoints |
| [ML Framework](examples/ml-framework/) | Neural network framework built from scratch with custom Tensor, layers, and UniNN model |
| [Library Management](examples/library-mgmt/) | Full-stack app with REST API, 10K book dataset, ML prediction engine, and dashboard |
| [Hello World](examples/basic/hello.uniL) | Simple mixed Python/Java syntax |

## IDE & Tooling

| Tool | Description |
|------|-------------|
| [VS Code Extension](tools/vscode-extension/) | Syntax highlighting, snippets, language config for `.uniL` files |
| [JetBrains Plugin](tools/jetbrains-plugin/) | IntelliJ IDEA / PyCharm plugin with highlighting and completion |
| [Eclipse Plugin](tools/eclipse-plugin/) | Eclipse editor with syntax coloring and content assist |
| [UniLang IDE](tools/unilang-ide/) | Standalone Electron-based IDE with editor, file tree, terminal |
| [Language Server](crates/unilang-lsp/) | LSP server for real-time diagnostics in any editor |

## Documentation

### Getting Started
| Document | Description |
|----------|-------------|
| [Installation Guide](docs/guides/INSTALLATION.md) | How to install UniLang on any platform |
| [Quick Start Tutorial](docs/guides/QUICKSTART.md) | Write your first UniLang program in 5 minutes |
| [IDE Setup Guides](docs/guides/) | Step-by-step plugin installation for all IDEs |

### Language Reference
| Document | Description |
|----------|-------------|
| [Language Specification](docs/specifications/LANGUAGE_SPEC.md) | Formal grammar and semantics |
| [Interop Guide](docs/design/INTEROP_GUIDE.md) | How Python + Java code work together |
| [Type System](docs/design/TYPE_SYSTEM.md) | Gradual typing, implicit casting rules |

### Architecture
| Document | Description |
|----------|-------------|
| [Compiler Pipeline](docs/architecture/COMPILER_PIPELINE.md) | 6-stage compilation from source to execution |
| [System Architecture](docs/architecture/ARCHITECTURE.md) | Overall system design |
| [Design Decisions](docs/design/DESIGN_DECISIONS.md) | Key design choices and trade-offs |
| [Threading Model](docs/design/THREADING_MODEL.md) | Multi-threading strategy |

### ML Framework
| Document | Description |
|----------|-------------|
| [ML Framework Docs](examples/ml-framework/docs/README.md) | Build neural networks from scratch |
| [Core Concepts](examples/ml-framework/docs/01_CORE_CONCEPTS.md) | Tensors, neurons, networks |
| [UniNN Architecture](examples/ml-framework/docs/03_UNINN_MODEL.md) | Our custom model architecture |
| [API Reference](examples/ml-framework/docs/07_API_REFERENCE.md) | Complete API reference |

### Project
| Document | Description |
|----------|-------------|
| [PRD](docs/planning/PRD.md) | Product requirements |
| [Roadmap](docs/planning/ROADMAP.md) | Development phases |
| [Contributing](CONTRIBUTING.md) | How to contribute |
| [Governance](docs/planning/GOVERNANCE.md) | Apache-style governance |

## Getting Started

> **Note:** UniLang is in early development. The following instructions will be updated as the toolchain matures.

### Prerequisites

- Rust 1.80+ (the only hard requirement — the compiler is pure Rust)
- Java 21+ / Python 3.11+ are future v2.0 targets (not required today)

### Build from Source

```bash
git clone https://github.com/apache/unilang.git
cd unilang
make build
```

### Hello World

```unilang
// hello.uniL
print("Hello from UniLang!")
System.out.println("Hello from UniLang's Java side!");
```

```bash
unilang run hello.uniL
```

## Architecture Overview

The current implementation uses a Rust-native compiler and stack-based VM. JVM and CPython backends are planned for v2.0.

```
┌─────────────────────────────────────────────────┐
│              .uniL Source File                  │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│     Unified Lexer  (unilang-lexer)              │
│  Python + Java tokens, f-strings, comments      │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│     Parser  (unilang-parser)                    │
│  Pratt expressions · indent + brace blocks      │
│  Classes, functions, control flow, exceptions   │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│     Semantic Analyzer  (unilang-semantic)       │
│  Gradual type inference · scope resolution      │
│  Name binding · prelude function registry       │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│     Bytecode Compiler  (unilang-codegen)        │
│  40+ opcodes · constant pool · function/class   │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│     Stack-Based VM  (unilang-runtime)           │
│  Call frames · builtins registry · HTTP server  │
├──────────────┬──────────────────────────────────┤
│  stdlib      │  unilang-drivers                 │
│  (35+ fns)   │  SQLite · Redis · Kafka · ES     │
│              │  MySQL · Postgres · MongoDB       │
└──────────────┴──────────────────────────────────┘

── v2.0 targets (future) ───────────────────────────
  JVM Backend → emit .class files, call Java libraries
  CPython Bridge → import numpy, sklearn, torch
```

## Community

- **Mailing List:** dev@unilang.apache.org (planned)
- **Issue Tracker:** GitHub Issues
- **Discussions:** GitHub Discussions

## License

UniLang is licensed under the [Apache License 2.0](LICENSE).

```
Copyright 2026 The Apache Software Foundation

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
