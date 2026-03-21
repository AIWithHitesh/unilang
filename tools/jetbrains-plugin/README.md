# UniLang JetBrains Plugin

> Part of the [UniLang](../../README.md) project — a unified programming language combining Python and Java syntax.

Language support for UniLang (`.uniL` files) in JetBrains IDEs including IntelliJ IDEA, PyCharm, WebStorm, and others.

UniLang combines Python and Java syntax into a single language.

## Features

- **Syntax Highlighting** — Keywords (Python + Java), strings (regular, triple-quoted, f-strings, raw), comments (`//`, `#`, `/* */`), numbers, operators, types, and decorators
- **Code Completion** — Auto-complete for Python keywords, Java keywords, built-in types, and common patterns
- **Error Annotations** — Real-time detection of unmatched brackets/braces/parens, unknown escape sequences, and TODO/FIXME highlighting

## Building

Requires Java 17+.

```bash
./gradlew buildPlugin
```

The installable plugin `.zip` is produced at `build/distributions/`.

## Installation

1. Build the plugin with `./gradlew buildPlugin`
2. In your JetBrains IDE, go to **Settings > Plugins > Gear icon > Install Plugin from Disk...**
3. Select the `.zip` file from `build/distributions/`
4. Restart the IDE

## Development

Run the plugin in a sandboxed IDE instance:

```bash
./gradlew runIde
```

## Related Documentation

- [JetBrains Setup Guide](../../docs/guides/JETBRAINS_SETUP.md) — Detailed installation and configuration instructions
- [Language Specification](../../docs/specifications/LANGUAGE_SPEC.md) — Formal grammar and semantics
