# UniLang Compiler Pipeline

Complete documentation of the UniLang compilation pipeline from source code to execution.

## Pipeline Overview

```
.uniL Source Code
       │
       ▼
┌──────────────┐
│    Lexer      │  crates/unilang-lexer/
│  (Tokenizer)  │  Produces: Vec<Token>
└──────┬───────┘
       │
       ▼
┌──────────────┐
│    Parser     │  crates/unilang-parser/
│  (AST Gen)    │  Produces: Module (AST)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Semantic    │  crates/unilang-semantic/
│   Analyzer    │  Validates: types, scopes, names
└──────┬───────┘
       │
       ▼
┌──────────────┐
│    Code       │  crates/unilang-codegen/
│   Generator   │  Produces: Bytecode
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Runtime     │  crates/unilang-runtime/
│     VM        │  + crates/unilang-stdlib/
│  (Executor)   │  Executes bytecode
└──────────────┘
```

## Stage 1: Lexer (`crates/unilang-lexer/`)

Converts raw source text into a stream of tokens.

**Key features:**
- Handles the union of Python and Java token sets (75+ keywords)
- Indentation tracking: emits `Indent`/`Dedent` tokens for Python-style blocks
- Automatic Semicolon Insertion (ASI): emits `Newline` tokens at statement boundaries
- All string types: regular, triple-quoted, f-strings, raw strings
- All numeric formats: decimal, hex (0x), octal (0o), binary (0b), float, scientific
- Brace depth tracking: suppresses indentation inside `{}`
- Paren depth tracking: suppresses newlines inside `()` and `[]`

**Input:** Source text (`&str`)
**Output:** `Vec<Token>` where each `Token` has a `TokenKind` and `Span`

## Stage 2: Parser (`crates/unilang-parser/`)

Converts token stream into an Abstract Syntax Tree (AST).

**Key features:**
- **Pratt expression parser** with 15 precedence levels
- **Recursive descent** for statements
- Handles both Python-style (indentation + colon) and Java-style (braces) blocks
- Error recovery: skips to next statement boundary on parse error
- Produces `Stmt::Error` and `Expr::Error` nodes for unrecoverable errors

**Expression precedence (lowest to highest):**
1. Assignment (`=`, `+=`, `-=`, etc.)
2. Ternary (`? :`, `x if cond else y`)
3. Logical Or (`or`, `||`)
4. Logical And (`and`, `&&`)
5. Logical Not (`not`, `!`)
6. Comparison (`==`, `!=`, `<`, `>`, `in`, `is`, `instanceof`)
7. Bitwise Or (`|`)
8. Bitwise Xor (`^`)
9. Bitwise And (`&`)
10. Shift (`<<`, `>>`)
11. Addition/Subtraction (`+`, `-`)
12. Multiplication/Division (`*`, `/`, `//`, `%`)
13. Power (`**`)
14. Unary (`-`, `+`, `~`)
15. Postfix (call, index, attribute, `new`)
16. Primary (literals, identifiers, parenthesized)

**Input:** `Vec<Token>` + source text
**Output:** `Module` (AST root containing `Vec<Spanned<Stmt>>`)

## Stage 3: Semantic Analyzer (`crates/unilang-semantic/`)

Validates the AST for correctness.

**Checks performed:**
- **Name resolution:** Every identifier resolves to a declaration
- **Scope management:** Lexical scoping with nested function/class/block scopes
- **Type checking:** Gradual type system — `Dynamic` type is compatible with everything
- **Declaration validation:** Duplicate names, missing initializers
- **Context validation:** `return` inside functions, `break`/`continue` inside loops
- **Mutability:** Prevents reassignment to `val`, `const`, `final` variables
- **Call arity:** Function call argument count matches parameter count

**Type system:**
- Primitive types: `Int`, `Float`, `Double`, `Bool`, `String`, `Char`, `Void`
- Compound types: `Array(T)`, `Generic(name, params)`, `Optional(T)`, `Union(types)`
- Special: `Dynamic` (Python-style untyped — assignable to/from anything)
- `Unknown` for unresolved types, `Error` for error recovery

**Input:** `&Module` (AST)
**Output:** `(AnalysisResult, DiagnosticBag)`

## Stage 4: Code Generator (`crates/unilang-codegen/`)

Compiles the AST into stack-based bytecode.

**Instruction set (40+ opcodes):**

| Category | Opcodes |
|----------|---------|
| Stack | `LoadConst`, `LoadLocal`, `StoreLocal`, `LoadGlobal`, `StoreGlobal`, `Pop`, `Dup` |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Mod`, `Pow`, `Neg` |
| Comparison | `Eq`, `NotEq`, `Lt`, `Gt`, `LtEq`, `GtEq` |
| Logical | `And`, `Or`, `Not` |
| Bitwise | `BitAnd`, `BitOr`, `BitXor`, `BitNot`, `LShift`, `RShift` |
| Control flow | `Jump`, `JumpIfFalse`, `JumpIfTrue` |
| Functions | `Call`, `Return`, `MakeFunction` |
| Objects | `GetAttr`, `SetAttr`, `MakeClass`, `NewInstance` |
| Collections | `MakeList`, `MakeDict`, `GetIndex`, `SetIndex` |
| I/O | `Print` |
| Control | `Halt`, `Concat` |

**Jump patching:** Forward jumps (if/else, loops) use placeholder targets that are patched when the jump destination is known.

**Input:** `&Module` (AST)
**Output:** `Bytecode` (instructions + function table + class definitions)

## Stage 5: Runtime VM (`crates/unilang-runtime/`)

Stack-based virtual machine that interprets bytecodes.

**Architecture:**
- **Operand stack:** Values pushed/popped during computation
- **Call frames:** Each function call pushes a frame with its own locals and instruction pointer
- **Globals:** Module-level variable store
- **Function table:** Compiled function bodies indexed by ID
- **Output buffer:** Captured print output (for testing)

**Value types at runtime:**
- `Int(i64)`, `Float(f64)`, `String`, `Bool`, `Null`
- `List(Vec<RuntimeValue>)`, `Dict(Vec<(key, value)>)`
- `Function(usize)` — index into function table
- `NativeFunction(String)` — built-in function by name
- `Instance { class_name, fields }` — object instance

**Arithmetic promotion:** `Int + Float → Float` (automatic widening)

## Stage 6: Standard Library (`crates/unilang-stdlib/`)

35+ built-in functions registered in the VM.

| Category | Functions |
|----------|-----------|
| I/O | `print`, `input` |
| Type conversion | `int`, `float`, `str`, `bool` |
| Type checking | `type`, `isinstance` |
| Math | `abs`, `min`, `max`, `pow`, `sqrt`, `floor`, `ceil`, `round` |
| Collections | `len`, `range`, `sorted`, `reversed`, `enumerate`, `zip` |
| Strings | `upper`, `lower`, `split`, `join`, `strip`, `replace`, `contains`, `startswith`, `endswith` |
| Utility | `hash` |

## CLI Commands

```bash
# Run a UniLang program (full pipeline)
unilang run hello.uniL

# Check for errors without running
unilang check hello.uniL

# Compile and show bytecode disassembly
unilang compile hello.uniL

# Tokenize and show token stream
unilang lex hello.uniL
```

## Test Coverage

| Crate | Tests | What's tested |
|-------|-------|---------------|
| unilang-common | 10 | Span, SourceFile, DiagnosticBag |
| unilang-lexer | 35 | All token types, indentation, ASI, strings, numbers |
| unilang-parser | 15 | Expressions, statements, blocks, classes, imports |
| unilang-semantic | 18 | Scoping, types, name resolution, context validation |
| unilang-codegen | 16 | All compilation targets, jump patching |
| unilang-runtime | 24 | All opcodes, function calls, collections, full pipeline |
| unilang-stdlib | 24 | All built-in functions |
| **Total** | **142** | |
