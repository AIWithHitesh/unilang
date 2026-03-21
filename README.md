# UniLang

**A unified programming language that seamlessly integrates Python and Java syntax, enabling developers to leverage the best of both ecosystems in a single codebase.**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-planning-yellow.svg)]()
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

UniLang is in **active development** with a working compiler pipeline:

- [x] Language specification
- [x] Lexer (hand-written, full Python+Java token support)
- [x] Parser (Pratt expression parser + recursive descent statements)
- [x] Semantic analyzer (gradual type system, scope resolution, name binding)
- [x] Code generation (stack-based bytecode with 40+ opcodes)
- [x] Runtime VM (stack-based interpreter with call frames)
- [x] Standard library (35+ built-in functions: math, collections, strings, I/O)
- [x] CLI toolchain (`unilang run`, `unilang check`, `unilang compile`, `unilang lex`)
- [x] Language Server Protocol (LSP) server
- [x] IDE support (VS Code, JetBrains, Eclipse, standalone IDE)
- [x] ML framework (custom Tensor, autograd, layers, optimizers)

## Downloads & Installation

### UniLang CLI (Compiler + Language Server)
Download from [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases):

| Platform | Download | Install |
|----------|----------|---------|
| macOS (Apple Silicon) | `unilang-cli-macos-arm64.dmg` | Open DMG, drag to `/usr/local/bin/` |
| macOS (Intel) | `unilang-cli-macos-x86_64.tar.gz` | `tar xzf *.tar.gz && sudo cp bin/unilang /usr/local/bin/` |
| Windows | `unilang-cli-windows-x86_64.zip` | Extract, add `bin/` to PATH |
| Linux | `unilang-cli-linux-x86_64.tar.gz` | `tar xzf *.tar.gz && sudo cp bin/unilang /usr/local/bin/` |

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
| [ML Framework](examples/ml-framework/) | Neural network framework built from scratch with custom Tensor, layers, and UniNN model |
| [Library Management](examples/library-mgmt/) | Full-stack app with REST API, 10K book dataset, ML prediction engine, and dashboard |
| [Hello World](examples/basic/hello.uniL) | Simple mixed Python/Java syntax |
| [ML Pipeline](examples/ml/ml_pipeline.uniL) | Java thread pools + scikit-learn ML |
| [Threading](examples/threading/concurrent_processing.uniL) | Multi-threaded data processing |
| [Web Service](examples/advanced/web_service.uniL) | Spring Boot + Python ML REST API |

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

- Java 21+ (JDK)
- Python 3.11+
- Rust 1.75+ (for compiler)
- LLVM 17+ (optional, for native compilation)

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

```
┌─────────────────────────────────────────────────┐
│                  .uniL Source                    │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│          Unified Lexer / Tokenizer              │
│  (Handles both Python & Java token grammars)    │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│         Context-Aware Parser (AST)              │
│  (Resolves ambiguity via context analysis)      │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│          Semantic Analyzer                       │
│  (Type inference, scope resolution, interop)    │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│      UniLang Intermediate Representation (UIR)  │
└──────┬──────────────────────────┬───────────────┘
       │                          │
┌──────▼──────┐          ┌───────▼───────┐
│ JVM Backend │          │ Python Backend│
│ (Bytecode)  │          │ (CPython/AST) │
└──────┬──────┘          └───────┬───────┘
       │                          │
┌──────▼──────────────────────────▼───────────────┐
│         UniLang Runtime (Bridge Layer)          │
│  (JVM ↔ CPython interop, shared memory, GIL    │
│   management, thread synchronization)           │
└─────────────────────────────────────────────────┘
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
