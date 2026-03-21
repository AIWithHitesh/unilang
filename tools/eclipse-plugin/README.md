# UniLang Eclipse Plugin

> Part of the [UniLang](../../README.md) project — a unified programming language combining Python and Java syntax.

An Eclipse IDE plugin providing editor support for the UniLang programming language (`.uniL` files). UniLang combines Python and Java syntax into a single unified language.

## Features

- **Syntax Highlighting** for both Python and Java keywords, with distinct colors:
  - Python keywords: blue
  - Java keywords: purple
  - Strings: green
  - Comments: gray (supports `//`, `#`, and `/* */`)
  - Numbers: cyan
  - Types: teal
  - Decorators/Annotations: yellow
- **Content Assist** (auto-completion) for keywords, types, and common patterns
- **Document Partitioning** with support for:
  - Single-line comments (`//` and `#`)
  - Block comments (`/* ... */`)
  - Regular strings (single and double quoted)
  - Triple-quoted strings (`"""..."""` and `'''...'''`)
- **Line Numbers** and overview ruler enabled by default

## Building

### Option 1: Eclipse PDE (recommended for development)

1. Import the project into Eclipse via **File > Import > Existing Projects into Workspace**.
2. Select the `eclipse-plugin` directory.
3. Right-click the project and select **Export > Plug-in Development > Deployable plug-ins and fragments**.
4. Choose an output directory and click **Finish**.

### Option 2: Maven/Tycho

Add a `pom.xml` with the Tycho plug-in for automated builds:

```bash
mvn clean package -Dtycho.version=3.0.0
```

The built JAR will be in `target/`.

## Installation

### Drop-in Install

Copy the built JAR (`org.unilang.eclipse_0.1.0.jar`) into your Eclipse installation's `dropins/` folder:

```
<eclipse-install>/dropins/org.unilang.eclipse_0.1.0.jar
```

Restart Eclipse.

### Update Site

If you build a P2 update site (using Tycho or PDE), install via:

1. **Help > Install New Software...**
2. Click **Add...** and point to the update site directory or URL.
3. Select "UniLang Editor" and follow the wizard.

## Requirements

- Eclipse 2023-03 or later
- Java 17+

## File Association

The plugin automatically associates with `.uniL` files. When you open a `.uniL` file, the UniLang Editor is used by default.

## Project Structure

```
eclipse-plugin/
  META-INF/
    MANIFEST.MF              -- OSGi bundle manifest
  src/
    org/unilang/eclipse/
      Activator.java          -- Plugin lifecycle
      editor/
        UniLangEditor.java                      -- Text editor
        UniLangSourceViewerConfiguration.java   -- Highlighting & assist config
        UniLangDocumentProvider.java            -- Document partitioning setup
        SingleTokenScanner.java                 -- Helper for partition styling
      syntax/
        UniLangPartitionScanner.java   -- Splits document into partitions
        UniLangCodeScanner.java        -- Token rules for code regions
        UniLangColorManager.java       -- Color definitions and caching
      completion/
        UniLangCompletionProcessor.java -- Auto-completion proposals
  plugin.xml                 -- Eclipse extension points
  build.properties           -- PDE build configuration
  icons/                     -- Editor icons
```

## Related Documentation

- [Eclipse Setup Guide](../../docs/guides/ECLIPSE_SETUP.md) — Detailed installation and configuration instructions
- [Main Project README](../../README.md) — UniLang overview, downloads, and full documentation index
