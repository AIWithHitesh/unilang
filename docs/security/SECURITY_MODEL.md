# UniLang Security Model

## Overview

UniLang programs run inside the UniLang VM — a Rust-based stack VM defined in
`crates/unilang-runtime/`. The VM provides several layers of protection for both
trusted and untrusted code. This document describes what those protections are,
where their boundaries lie, and how to configure the VM for security-sensitive
deployments.

---

## Memory Safety

UniLang's runtime is implemented entirely in safe Rust. This provides the
following guarantees by construction:

- **No buffer overflows.** All slice accesses are bounds-checked at runtime.
  The VM raises an `IndexOutOfBounds` error rather than reading or writing past
  the end of an allocation.
- **No use-after-free.** Rust's ownership system ensures that every allocation
  is freed exactly once. The VM's stack, heap values, and call frames are owned
  by the `VM` struct and dropped when the VM goes out of scope.
- **No dangling pointers.** `RuntimeValue` clones are used freely; references
  never outlive the data they point to.
- **No data races.** The `VM` type is `!Send + !Sync`. Each concurrent execution
  must use its own `VM` instance. There is no shared mutable global state.
- **No `unsafe` blocks** in `unilang-runtime`. Any future `unsafe` code must be
  reviewed and explicitly justified.

The Rust compiler enforces these properties at compile time. The result is that
a well-tested UniLang program cannot corrupt the host process's memory, even if
the UniLang program itself contains bugs.

---

## Execution Limits

Without limits a runaway UniLang program (infinite loop, unbounded recursion,
enormous string concatenation) can consume all available CPU or memory. The
`ExecutionLimits` struct in `crates/unilang-runtime/src/limits.rs` addresses
this.

### Available Limits

| Field                 | Type     | Default          | Sandboxed     | Development |
|-----------------------|----------|------------------|---------------|-------------|
| `max_instructions`    | `u64`    | 50,000,000       | 1,000,000     | `u64::MAX`  |
| `max_call_depth`      | `usize`  | 500              | 100           | 10,000      |
| `max_collection_size` | `usize`  | 1,000,000        | 100,000       | `usize::MAX`|
| `max_string_bytes`    | `usize`  | 10 MB            | 1 MB          | `usize::MAX`|

### Profiles

Three built-in profiles are provided:

```rust
// Default — balanced for production use with trusted code.
ExecutionLimits::default()

// Sandboxed — suitable for executing untrusted user-supplied scripts.
ExecutionLimits::sandboxed()

// Development — generous limits for REPL and local development.
ExecutionLimits::development()
```

### How Limits Are Enforced

- **Instruction limit.** `VM::step()` increments `instruction_count` on every
  opcode dispatch. A check fires every 1,000 instructions to avoid the overhead
  of checking on every single instruction. When `instruction_count` exceeds
  `max_instructions`, the VM returns a `RuntimeError` with `ErrorKind::Exception`.
- **Call depth limit.** Both the `Opcode::Call` handler and the internal
  `call_unilang_function` helper check `self.frames.len()` against
  `limits.max_call_depth` before pushing a new call frame.

### Configuring Limits

```rust
use unilang_runtime::vm::VM;
use unilang_runtime::limits::ExecutionLimits;

// Run with sandboxed limits:
let mut vm = VM::new().with_limits(ExecutionLimits::sandboxed());
vm.run(&bytecode)?;

// Custom limits:
let mut vm = VM::new().with_limits(ExecutionLimits {
    max_instructions: 500_000,
    max_call_depth: 50,
    max_collection_size: 10_000,
    max_string_bytes: 256 * 1024,
});
vm.run(&bytecode)?;
```

---

## VM Isolation

Each call to `VM::new()` produces a completely independent VM instance:

- Independent operand stack, globals map, call-frame stack, and builtin table.
- No inter-VM communication channel exists in the runtime.
- Builtin functions registered on one VM are not visible to another.
- There is no shared allocator state between VM instances beyond the normal
  Rust global allocator.

When running untrusted code in a multi-tenant context (e.g., a web service that
executes user-supplied UniLang scripts), create one `VM` per request rather than
reusing a long-lived instance. This prevents one request from leaking globals
or builtins into the next.

---

## Unsafe Operations

UniLang programs have access to certain I/O operations through registered builtin
functions and drivers. Each builtin is opt-in: it must be explicitly registered
on the `VM` before a UniLang program can call it.

### File I/O

Accessible via the SQLite driver (`db_connect`, `db_query`, `db_exec`). The path
passed to `db_connect` is an arbitrary filesystem path. Running untrusted code
with the SQLite driver registered allows reads and writes to any file the host
process can access. Do not register the SQLite driver when running untrusted
UniLang programs.

### Network

The `serve(port, handler)` builtin binds a TCP socket on the given port.
Untrusted code should not have access to `serve`. Limit network access at the
OS level (e.g., using a network namespace or firewall rule) if additional
isolation is required.

### Environment Variables

There is no built-in `getenv` or `setenv` function in the standard UniLang
runtime. Environment variable access can only occur if a host application
explicitly registers a builtin that exposes it.

### Process Exit

There is no built-in `exit()` function. A program can only terminate normally
(reaching end of bytecode), via `Halt` opcode, or by raising an unhandled
exception.

---

## Running Untrusted Code

The recommended pattern for executing untrusted UniLang scripts:

1. Do **not** register I/O builtins (SQLite driver, `serve`, etc.).
2. Apply `ExecutionLimits::sandboxed()`.
3. Use one `VM` instance per script execution.
4. Run the VM in a separate thread with a wall-clock timeout as a second line
   of defence.

```rust
use unilang_runtime::vm::VM;
use unilang_runtime::limits::ExecutionLimits;
use std::time::Duration;

fn run_untrusted(bytecode: &Bytecode) -> Result<RuntimeValue, RuntimeError> {
    // No builtins registered — untrusted code cannot do I/O.
    let mut vm = VM::new().with_limits(ExecutionLimits::sandboxed());
    vm.run(bytecode)
}
```

For stronger isolation (OS-level sandbox), consider wrapping the VM in a
process with restricted capabilities using `seccomp` (Linux) or App Sandbox
(macOS).

---

## Operand Stack Overflow

In addition to the configurable call depth limit, the VM enforces a hard-coded
operand stack depth limit of `100,000` entries (`MAX_STACK_DEPTH` constant in
`vm.rs`). Exceeding this limit prints a diagnostic message and calls
`std::process::exit(1)`. This is a last-resort guard; in normal operation the
call depth limit is hit first for recursive programs, and instruction limits
catch infinite loops before the stack overflows.

Future work: convert the stack overflow from `process::exit` to a proper
`RuntimeError` return so the caller can handle it gracefully.

---

## Dependency Supply-Chain Security

UniLang's runtime has zero runtime dependencies outside the Rust standard library
and `unilang-codegen` (also first-party). The minimal dependency surface reduces
supply-chain risk. Driver crates (`unilang-driver-sqlite`, etc.) introduce
third-party crates — review their dependencies before deploying to sensitive
environments.

---

## Reporting Vulnerabilities

Please report security vulnerabilities via the GitHub Security Advisories page:

  https://github.com/AIWithHitesh/unilang/security/advisories/new

Do **not** open a public issue for security vulnerabilities. See `SECURITY.md`
at the repository root for the full vulnerability disclosure policy.
