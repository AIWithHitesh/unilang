# Introducing UniLang: Write Python and Java in the Same File

*Published 2026-04-17 — UniLang Core Team*

---

Today we are announcing the general availability of **UniLang 1.0** — a programming
language that lets you write Python-style and Java-style code side-by-side, in the same
source file, with zero impedance mismatch. One compiler. One VM. One `unilang run`.

## Why UniLang?

For the past decade, developers have been choosing sides. Python for data science, scripting,
and startup MVPs. Java for enterprise services, Android, and anything that needs to run for
years without maintenance. The two ecosystems have grown enormous, each with thousands of
libraries, frameworks, and established idioms.

But the wall between them is costly.

- A data team writes Python models; the backend team writes Java APIs. Calling one from the
  other means REST calls, gRPC, or hand-rolled JNI glue code.
- A new hire fluent in Python joins a Java shop. Three weeks of ramp-up just to understand
  the syntax, the build system, and the idioms.
- A polyglot codebase means two sets of linters, two formatters, two LSP servers, two IDE
  configurations, and two mental models active at the same time.

UniLang collapses that wall.

---

## What UniLang Looks Like

The core idea is simple: UniLang's grammar is a strict superset of both Python and Java
surface syntax. Any valid Python-style program is valid UniLang. Any valid Java-style program
is valid UniLang. And you can mix them freely.

Here is a taste:

```unilang
# Python-style function — def, colon, indent
def greet(name):
    return "Hello, " + name

# Java-style class — braces, typed fields, void methods
class Counter {
    int count = 0

    void increment() {
        count = count + 1
    }

    int value() {
        return count
    }
}

# Mix freely in the same file
c = Counter()
c.increment()
c.increment()
print(greet("world") + " — count is " + str(c.value()))
```

Output:
```
Hello, world — count is 2
```

No adapter layer. No import gymnastics. The UniLang compiler understands both block styles
and unifies them into a single AST.

### A Real-World Example: HTTP API with SQLite

Here is a complete web service — the kind you would write in Flask or Spring Boot — in pure
UniLang:

```unilang
from unilang import serve, db_connect, db_exec, db_query
from unilang import to_json, from_json, print

# Initialise database
db = db_connect("tasks.db")
db_exec(db, """
    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        done INTEGER DEFAULT 0
    )
""")

# Java-style route handler class
class TaskHandler {
    def get_all(req) {
        rows = db_query(db, "SELECT id, title, done FROM tasks")
        return {"status": 200, "body": to_json(rows)}
    }

    def create(req) {
        body = from_json(req["body"])
        db_exec(db, "INSERT INTO tasks (title) VALUES (?)", [body["title"]])
        return {"status": 201, "body": "{\"message\": \"created\"}"}
    }
}

handler = TaskHandler()

# Python-style router function
def router(req):
    if req["method"] == "GET" and req["path"] == "/tasks":
        return handler.get_all(req)
    elif req["method"] == "POST" and req["path"] == "/tasks":
        return handler.create(req)
    return {"status": 404, "body": "Not Found"}

print("Listening on :8080")
serve(8080, router)
```

That is the entire application. `serve` is a stdlib builtin — no framework import, no
annotation processing, no `@SpringBootApplication`.

### Python-Style Data Processing

```unilang
import json

# Comprehensions, f-strings, all standard Python idioms work
def process_records(records):
    totals = {}
    for r in records:
        key = r["category"]
        if not has_key(totals, key):
            totals[key] = 0
        totals[key] = totals[key] + r["amount"]

    summary = sorted(keys(totals))
    for cat in summary:
        print(f"  {cat}: ${totals[cat]:.2f}")

    return totals

data = from_json(read_file("sales.json"))
print("Sales by category:")
results = process_records(data["records"])
write_file("summary.json", to_json(results))
```

### Mixing Java-Style Types with Python-Style Logic

```unilang
# Java-style type annotations on class fields
class Product {
    String name
    float price
    int stock

    boolean is_available() {
        return stock > 0
    }

    String display() {
        return f"{name} — ${price:.2f} ({stock} in stock)"
    }
}

# Python-style list operations
inventory = [
    Product(name="Widget A", price=9.99, stock=42),
    Product(name="Widget B", price=24.99, stock=0),
    Product(name="Gadget X", price=149.99, stock=7),
]

available = [p for p in inventory if p.is_available()]
print(f"Available products ({len(available)}):")
for p in available:
    print("  " + p.display())
```

---

## The Compiler

UniLang's compiler is written entirely in Rust, structured as a 12-crate workspace. Each
stage of the pipeline is a separate crate, making it easy to use (for example) the parser
as a library without pulling in the VM.

```
Source (.uniL)
    │
    ▼
┌─────────────┐
│  unilang-   │  Unified lexer — Python tokens (INDENT/DEDENT, f-strings,
│  lexer      │  #-comments) + Java tokens (braces, semicolons, //-comments)
└──────┬──────┘
       │ Token stream
       ▼
┌─────────────┐
│  unilang-   │  Pratt (precedence-climbing) parser — unified AST
│  parser     │  (Expr, Stmt, Module, ClassDecl, FuncDecl)
└──────┬──────┘
       │ AST
       ▼
┌─────────────┐
│  unilang-   │  Semantic analyser — nested scopes, gradual type inference,
│  semantic   │  overload resolution, generic type checking (List<T>, Map<K,V>)
└──────┬──────┘
       │ Typed AST
       ▼
┌─────────────┐
│  unilang-   │  Bytecode codegen — 40+ opcode instruction set, constant pool,
│  codegen    │  function/class compilation
└──────┬──────┘
       │ Bytecode
       ▼
┌─────────────┐
│  unilang-   │  Stack-based VM — call frames, closures, exception handling,
│  runtime    │  builtin registry, ExecutionLimits sandbox
└─────────────┘
```

### Performance

Early Criterion benchmarks (on an Apple M2, single core):

| Benchmark | Result |
|-----------|--------|
| Lex 1000-line program | ~1.2 ms |
| Parse 1000-line program | ~3.8 ms |
| Full compile (lex+parse+semantic+codegen) 1000 lines | ~8.5 ms |
| VM: fibonacci(30) | ~28 ms |
| VM: 10,000 HTTP request simulations | ~190 ms |
| VM: SQLite insert 10,000 rows | ~420 ms |

Compile times are fast enough that `unilang run` feels instantaneous for typical programs.
The VM is interpreted (no JIT yet), so compute-heavy numeric workloads will be slower than
CPython with NumPy — but for I/O-bound services, the performance is well within acceptable
range.

---

## The Driver Ecosystem

One of the biggest pain points when writing services is connecting to databases and
message queues. UniLang ships **13 built-in drivers** as part of the standard distribution:

| Driver | Builtin prefix | Backend |
|--------|---------------|---------|
| SQLite | `db_*` | `rusqlite` (bundled — no install required) |
| Redis | `redis_*` | Redis crate |
| Kafka | `kafka_*` | In-memory (dev) |
| Elasticsearch | `es_*` | HTTP |
| MySQL | `mysql_*` | `mysql_async` |
| PostgreSQL | `pg_*` | `tokio-postgres` |
| MongoDB | `mongo_*` | Official driver |
| Memcached | `memcached_*` | `memcache` crate |
| RabbitMQ | `rmq_*` | `lapin` |
| NATS | `nats_*` | NATS crate |
| Prometheus | `prom_*` | HTTP pushgateway |
| InfluxDB | `influx_*` | HTTP line protocol |
| SMTP | `smtp_*` | `lettre` |

Using a driver is one line:

```unilang
db = db_connect("myapp.db")
rows = db_query(db, "SELECT * FROM users WHERE active = 1")

cache = redis_connect("redis://localhost:6379")
redis_set(cache, "session:42", to_json(user), 3600)

pg = pg_connect("postgresql://localhost/mydb")
result = pg_query(pg, "SELECT count(*) FROM orders")
```

### Community Drivers

Need a driver that is not in the standard distribution? Drop a single Rust file in
`src/community/` and it is automatically discovered and registered at build time — no
`lib.rs` edits, no `Cargo.toml` additions.

```bash
unilang driver new my-clickhouse-driver
# Creates src/community/my_clickhouse_driver.rs with the UniLangDriver trait scaffold
# Edit the file, rebuild — the driver appears in `unilang driver list`
```

---

## IDE Support

UniLang ships first-class IDE support out of the box.

### VS Code Extension

Install from the marketplace (`unilang.unilang-vscode`) and get:
- Syntax highlighting for `.uniL` files
- 15 snippets (http server, sqlite, redis, dataclass, test, match, builder pattern, observer, …)
- LSP-powered hover: hover any stdlib function name and see its signature and description
- Go-to-definition: jump to the declaration of any function or class
- Document formatting: save and the file is formatted automatically (4-space indent,
  trailing whitespace stripped, blank lines normalised)
- Debugger (DAP): set breakpoints, step through UniLang programs inside VS Code

### JetBrains Plugin

Install from the JetBrains Marketplace (`UniLang`) and get syntax highlighting and
LSP-powered completion in IntelliJ IDEA, PyCharm, and other JetBrains IDEs.

### Eclipse Plugin

Available from the Eclipse Marketplace for teams standardised on Eclipse.

### Standalone UniLang IDE

For those who want a zero-configuration option, `unilang-ide` is an Electron-based
editor with UniLang support baked in. No VS Code or JetBrains license required.

---

## Getting Started

### Install

```bash
# Linux / macOS (one-liner)
curl -fsSL https://raw.githubusercontent.com/AIWithHitesh/unilang/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/AIWithHitesh/unilang/main/install.ps1 | iex

# Verify
unilang --version
# unilang 1.5.0
```

### Create Your First Project

```bash
unilang new my-first-project
cd my-first-project
unilang run src/main.uniL
```

The `unilang new` wizard creates a project with a sensible `unilang.toml`, a starter
`src/main.uniL`, a `.gitignore`, and a `README.md`.

### Key Commands

| Command | What it does |
|---------|-------------|
| `unilang run <file>` | Compile and run a `.uniL` file |
| `unilang check <file>` | Type-check without running |
| `unilang fmt --write <file>` | Format in-place |
| `unilang lint <file>` | Lint (semantic + style rules) |
| `unilang test` | Discover and run `def test_*()` functions |
| `unilang repl` | Interactive REPL with block continuation |
| `unilang compile <file>` | Dump bytecode disassembly |
| `unilang pack` | Package project as `.uniLpkg` |
| `unilang build --incremental` | Incremental compilation (skip unchanged files) |
| `unilang driver list` | List all registered drivers |
| `unilang driver new <name>` | Scaffold a community driver |

### Example: REPL Session

```
$ unilang repl
UniLang 1.5.0 REPL
Type 'exit' to quit, 'help' for help.

>>> def add(a, b):
...     return a + b
...
>>> add(3, 4)
7
>>> class Point { int x; int y; }
>>> p = Point(x=10, y=20)
>>> print(f"Point: ({p.x}, {p.y})")
Point: (10, 20)
```

---

## What is Next

UniLang 1.5 is a production-ready release for building services and tooling. We are already
working on what comes next.

### v1.x — Near Term (2026)

- **Jupyter kernel** — run UniLang cells in notebooks for data science workflows
- **Package registry** — `unilang.dev` for publishing and installing community packages
- **WebAssembly target** — compile `.uniL` programs to WASM for browser execution
- **`unilang deploy`** — one-command deployment to cloud environments

### v2.0 — Long Term

- **Real JVM backend** — emit `.class` files and call JVM libraries natively (not via JNI
  bridge from the UniLang VM, but direct classfile output)
- **Real CPython bridge** — `import numpy`, `import sklearn`, `import torch` with zero glue
  code
- **GraalVM AOT compilation** — single native binary, no JVM needed for deployment
- **JavaScript/TypeScript interop** — a third language in the unified ecosystem

The long-term vision is a language where `import numpy`, `import java.util.concurrent`, and
your own UniLang modules all coexist in the same file without any friction.

---

## Numbers

A few highlights from the v1.5.0 release:

- **12 Rust crates** in the workspace
- **40+ VM opcodes** in the instruction set
- **113 stdlib functions** across 12 modules
- **13 built-in database/service drivers**
- **136 EBNF grammar rules** in the formal specification
- **500+ test cases** (unit, integration, e2e, stress)
- **15 IDE snippets** in the VS Code extension
- **80+ hover entries** in the LSP server
- **5 complete example projects** in the repository
- **~8.5 ms** full compile time for a 1,000-line program (Apple M2)

---

## Join the Community

UniLang is open source under the Apache License 2.0. We want your contributions.

- **GitHub:** `https://github.com/AIWithHitesh/unilang`
- **Issues:** Bug reports, feature requests, driver requests
- **Contributing:** See `CONTRIBUTING.md` for the development workflow
- **Drivers:** See `CONTRIBUTING_DRIVERS.md` to add a new database driver

Whether you want to improve the compiler, write a new driver, add a cookbook recipe, fix
a documentation typo, or build an example project — all contributions are welcome. The
`unilang driver new` command makes writing a driver genuinely approachable even if you are
new to Rust; the community auto-discovery system means you can contribute a working driver
in a single file.

We are also pursuing Apache Software Foundation incubation to establish neutral, community
governance for the long term. If you are an ASF member and would like to champion or mentor
the project, please reach out.

---

## Thank You

UniLang started as a question: what if you could write Python and Java in the same file?
Four months later, it is a full compiler pipeline, a VM, 13 database drivers, four IDE
plugins, a CLI with 15 commands, and 500 tests.

Thank you for reading this far. We hope UniLang makes your day a little less split between
two ecosystems.

```bash
curl -fsSL https://raw.githubusercontent.com/AIWithHitesh/unilang/main/install.sh | sh
unilang new my-project
unilang run src/main.uniL
```

---

*UniLang is released under the Apache License 2.0.*  
*Source: `https://github.com/AIWithHitesh/unilang`*

---

### Discussion Templates

#### Hacker News

**Title:** UniLang: Write Python and Java in the same file

UniLang is a new language (implemented in Rust) whose grammar is a strict superset of both
Python and Java surface syntax. You can mix `def` functions and `class {} ` declarations
with brace blocks in the same source file. It ships a full compiler pipeline, a stack-based
VM, 13 database drivers, and first-class VS Code/JetBrains/Eclipse IDE support.

The "unified Python+Java" story targets teams maintaining both ecosystems who want to share
code, idioms, and tooling. The JVM/CPython bridge allows calling into real Java libraries
(via JNI) and real Python packages (via pyo3) for cases where ecosystem access matters more
than the unified syntax.

GitHub: https://github.com/AIWithHitesh/unilang

Curious to hear thoughts — especially on the grammar ambiguity resolution and whether the
"same file" pitch resonates with anyone actually doing polyglot work.

#### Reddit r/programming

**Title:** UniLang 1.0 — a hybrid Python+Java language written in Rust

We shipped UniLang 1.0 today. The pitch: write Python and Java in the same `.uniL` file.
The compiler (Rust, 12 crates) handles both indented and brace-delimited blocks, both `def`
and typed method declarations, and compiles to a common bytecode that runs on a Rust VM.

Ships with 13 database drivers (SQLite, Redis, Postgres, MongoDB, Kafka, …), LSP server,
VS Code extension with DAP debugger, JetBrains plugin, and a 15-command CLI.

One-line install: `curl -fsSL .../install.sh | sh`

Repo: https://github.com/AIWithHitesh/unilang
