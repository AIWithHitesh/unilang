# UniLang IDE

> Part of the [UniLang](../../README.md) project — a unified programming language combining Python and Java syntax.

A lightweight, standalone Electron-based IDE for the UniLang programming language. UniLang combines Python and Java syntax in `.uniL` files.

## Features

- **Syntax Highlighting** for UniLang (Python + Java keywords, strings, comments, numbers, decorators)
- **File Explorer** with recursive directory tree, expand/collapse, and `.uniL` file icons
- **Code Editor** with line numbers, auto-indent, bracket matching, tab handling, and current-line highlight
- **Build & Run** integration with terminal output streaming
- **Output Terminal** with timestamps, color-coded output (stdout/stderr), and auto-scroll
- **Dark Theme** inspired by VS Code
- **Resizable Panels** (sidebar and terminal)
- **Keyboard Shortcuts** for common operations
- **Cross-Platform** builds for macOS, Windows, and Linux

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- npm >= 9

## Getting Started

Install dependencies:

```bash
cd tools/unilang-ide
npm install
```

Launch the IDE in development mode:

```bash
npm start
```

## Keyboard Shortcuts

| Action         | Shortcut              |
|----------------|-----------------------|
| New File       | Ctrl/Cmd + N          |
| Open File      | Ctrl/Cmd + O          |
| Open Folder    | Ctrl/Cmd + Shift + O  |
| Save           | Ctrl/Cmd + S          |
| Save As        | Ctrl/Cmd + Shift + S  |
| Build          | Ctrl/Cmd + B          |
| Run            | Ctrl/Cmd + R          |

## Building Distributables

Build for your current platform:

```bash
# macOS
npm run build:mac

# Windows
npm run build:win

# Linux
npm run build:linux

# All platforms
npm run build:all
```

Build output is written to the `dist/` directory.

## Project Structure

```
unilang-ide/
  package.json            # Electron app manifest
  electron-builder.yml    # Cross-platform build configuration
  src/
    main/
      main.js             # Electron main process
      preload.js          # Context bridge for renderer
    renderer/
      index.html          # IDE layout
      app.js              # Main renderer entry point
      editor.js           # Code editor component
      file-tree.js        # File explorer component
      terminal.js         # Output terminal component
      styles/
        main.css          # Dark theme stylesheet
```

## Architecture

The IDE uses Electron's main/renderer process model with `contextIsolation` enabled for security. The renderer communicates with the main process through a preload script that exposes a limited `electronAPI` surface:

- **File I/O**: `readFile`, `saveFile`, `readDirectory`
- **Dialogs**: `openFileDialog`, `openFolderDialog`, `saveFileDialog`
- **Commands**: `runCommand` (spawns child processes, streams stdout/stderr back)
- **Menu Events**: menu actions are forwarded from main to renderer via IPC

The code editor uses a textarea overlaid with a syntax-highlighted `<div>` -- no external editor library required.

## Related Documentation

- [IDE Setup Guide](../../docs/guides/IDE_SETUP.md) — Detailed installation and configuration instructions
- [Compiler Pipeline](../../docs/architecture/COMPILER_PIPELINE.md) — 6-stage compilation from source to execution
- [Main Project README](../../README.md) — UniLang overview, downloads, and full documentation index

## License

MIT
