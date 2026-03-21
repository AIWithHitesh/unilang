# VS Code / Cursor Setup Guide for UniLang

This guide walks you through installing and configuring the UniLang extension for Visual Studio Code and Cursor IDE. The extension provides syntax highlighting, code snippets, bracket matching, and optional language server support.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Download the Extension](#step-1-download-the-extension)
- [Step 2: Install the Extension](#step-2-install-the-extension)
- [Step 3: Verify Installation](#step-3-verify-installation)
- [Step 4: Configure the Language Server (Optional)](#step-4-configure-the-language-server-optional)
- [Features](#features)
- [Snippets Quick Reference](#snippets-quick-reference)
- [Cursor IDE](#cursor-ide)
- [Troubleshooting](#troubleshooting)
- [Uninstalling](#uninstalling)

---

## Prerequisites

Before you begin, make sure you have:

- **Visual Studio Code** installed. Download it from [https://code.visualstudio.com](https://code.visualstudio.com) if you don't have it yet.
- **UniLang CLI** installed (optional, but needed for the language server). See the [Installation Guide](INSTALLATION.md).

---

## Step 1: Download the Extension

1. Go to the [UniLang GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Find the latest release.
3. Under **Assets**, download the file named `unilang-vscode.vsix`.
4. Save it somewhere you can easily find it (for example, your Downloads folder).

---

## Step 2: Install the Extension

You can install the extension using either the command line or the VS Code user interface.

### Option A: Command Line (Fastest)

1. Open a terminal (Terminal app on macOS, Command Prompt or PowerShell on Windows, or your preferred terminal on Linux).
2. Run the following command, replacing the path with the actual location of the downloaded file:

   ```bash
   code --install-extension ~/Downloads/unilang-vscode.vsix
   ```

   On Windows, the command might look like:

   ```cmd
   code --install-extension C:\Users\YourName\Downloads\unilang-vscode.vsix
   ```

3. You should see a message like: `Extension 'unilang-vscode.vsix' was successfully installed.`
4. If VS Code is already open, restart it to activate the extension.

### Option B: VS Code User Interface

1. Open VS Code.
2. Open the **Command Palette**:
   - **macOS:** Press `Cmd + Shift + P`
   - **Windows / Linux:** Press `Ctrl + Shift + P`
3. Type `Install from VSIX` in the search box.
4. Select **"Extensions: Install from VSIX..."** from the dropdown.
5. A file browser window will appear. Navigate to where you saved `unilang-vscode.vsix`.
6. Select the file and click **Open** (or **Select** on some systems).
7. VS Code will install the extension. You will see a notification in the bottom-right corner confirming the installation.
8. Click **"Reload"** or **"Restart"** when prompted to activate the extension.

---

## Step 3: Verify Installation

1. In VS Code, create a new file by pressing `Cmd + N` (macOS) or `Ctrl + N` (Windows/Linux).
2. Save the file as `test.uniL` (the `.uniL` extension is important).
3. Type the following code into the file:

   ```unilang
   // Python-style function
   def greet(name):
       return f"Hello, {name}!"

   // Java-style class
   public class App {
       public static void main(String[] args) {
           print("UniLang is working!")
       }
   }
   ```

4. Check for syntax highlighting:
   - Keywords like `def`, `class`, `public`, `import`, `return` should be **colored**.
   - Strings like `"Hello, {name}!"` should appear in a different color.
   - Comments starting with `//` or `#` should appear dimmed or in a distinct color.

5. If you see colored syntax highlighting, the extension is installed correctly.

---

## Step 4: Configure the Language Server (Optional)

The UniLang Language Server provides real-time error highlighting, diagnostics, and richer editor features. It requires the UniLang CLI to be installed.

### 4.1: Verify the Language Server Binary Exists

Run this command in your terminal:

```bash
which unilang-lsp
```

On Windows:

```cmd
where unilang-lsp
```

If this prints a path (for example, `/usr/local/bin/unilang-lsp`), the language server is installed. If it says "not found," install UniLang CLI first (see [Installation Guide](INSTALLATION.md)).

### 4.2: Configure VS Code to Use the Language Server

1. Open VS Code.
2. Open **Settings**:
   - **macOS:** Press `Cmd + ,`
   - **Windows / Linux:** Press `Ctrl + ,`
3. In the search bar at the top of the Settings page, type `unilang`.
4. Find the setting **"UniLang: LSP Path"**.
5. Enter the path to the `unilang-lsp` binary. For example:
   - **macOS / Linux:** `/usr/local/bin/unilang-lsp`
   - **Windows:** `C:\Program Files\UniLang\bin\unilang-lsp.exe`

Alternatively, you can edit `settings.json` directly. Open the Command Palette (`Cmd + Shift + P` or `Ctrl + Shift + P`), type `Open User Settings (JSON)`, and add:

```json
{
    "unilang.lsp.path": "/usr/local/bin/unilang-lsp"
}
```

### 4.3: Restart VS Code

Close and reopen VS Code for the language server to start.

### 4.4: Verify the Language Server is Running

1. Open a `.uniL` file.
2. Check the bottom status bar in VS Code. You should see a UniLang indicator (it may show a spinning icon briefly as the server starts).
3. Try introducing a deliberate error (for example, delete a closing bracket). If the language server is running, you should see a red underline on the error.

---

## Features

Once installed, the UniLang extension provides the following features:

### Syntax Highlighting
All Python and Java keywords are highlighted, including `def`, `class`, `public`, `private`, `import`, `return`, `if`, `else`, `for`, `while`, `try`, `except`, `catch`, and many more.

### Code Snippets
Type a snippet prefix and press `Tab` to expand it into a code template. See the [Snippets Quick Reference](#snippets-quick-reference) below.

### Bracket Matching
When you click on a bracket `(`, `[`, or `{`, VS Code highlights the matching closing bracket. This works for both Python-style and Java-style blocks.

### Auto-Closing Pairs
When you type an opening bracket, quote, or parenthesis, VS Code automatically inserts the closing one:
- `(` inserts `)`
- `[` inserts `]`
- `{` inserts `}`
- `"` inserts `"`
- `'` inserts `'`

### Indentation Support
The extension supports both Python-style (indent after `:`) and Java-style (indent inside `{}`) indentation patterns.

### Comment Toggling
- Press `Cmd + /` (macOS) or `Ctrl + /` (Windows/Linux) to toggle line comments.
- Both `//` and `#` comment styles are supported.

---

## Snippets Quick Reference

Type any of these prefixes in a `.uniL` file and press `Tab` to insert the corresponding code template.

| Prefix | What It Generates | Description |
|---|---|---|
| `defpy` | Python-style function | `def function_name(params):` with body |
| `defjava` | Java-style method | `public ReturnType methodName(params) { }` |
| `classpy` | Python class | `class ClassName:` with `__init__` method |
| `classjava` | Java class | `public class ClassName { }` with constructor |
| `ifpy` | Python if block | `if condition:` with body |
| `ifjava` | Java if block | `if (condition) { }` with body |
| `for` | Python for loop | `for item in collection:` with body |
| `forj` | Java for loop | `for (int i = 0; i < n; i++) { }` with body |
| `while` | Python while loop | `while condition:` with body |
| `whilej` | Java while loop | `while (condition) { }` with body |
| `try` | Python try/except | `try:` / `except Exception:` block |
| `trycatch` | Java try/catch | `try { } catch (Exception e) { }` block |
| `main` | Java main method | `public static void main(String[] args) { }` |
| `print` | Print statement | `print(...)` |
| `import` | Import statement | `import module` |
| `mlmodel` | ML model skeleton | Full class structure for an ML model |

---

## Cursor IDE

Cursor is built on top of VS Code, so the UniLang extension works in Cursor without any modifications.

### Installing in Cursor

1. Follow the same steps as for VS Code above.
2. If using the command-line method, replace `code` with `cursor`:

   ```bash
   cursor --install-extension ~/Downloads/unilang-vscode.vsix
   ```

3. Restart Cursor to activate the extension.
4. All features (syntax highlighting, snippets, language server) work identically in Cursor.

---

## Troubleshooting

### No syntax highlighting appears

1. Make sure the file has the `.uniL` extension (not `.uni` or `.unilang`).
2. Check that the extension is installed: open the Extensions panel (`Cmd + Shift + X` or `Ctrl + Shift + X`) and search for "UniLang." It should appear in the "Installed" section.
3. Try reloading VS Code: open the Command Palette and run **"Developer: Reload Window"**.

### Snippets don't expand when pressing Tab

1. Open Settings (`Cmd + ,` or `Ctrl + ,`).
2. Search for `Tab Completion`.
3. Set **"Editor: Tab Completion"** to `on` or `onlySnippets`.

### Language server doesn't start

1. Verify `unilang-lsp` is installed by running `unilang-lsp --version` in your terminal.
2. Check the path in your VS Code settings matches the actual binary location.
3. Open the VS Code Output panel (`Cmd + Shift + U` or `Ctrl + Shift + U`) and select "UniLang" from the dropdown to see any error messages.

### Extension conflicts

If another extension is interfering with `.uniL` files, you can set UniLang as the default for this file type:

1. Open a `.uniL` file.
2. Click the language indicator in the bottom-right status bar (it might say "Plain Text").
3. Select **"Configure File Association for '.uniL'..."**.
4. Choose **"UniLang"** from the list.

---

## Uninstalling

### From the command line:

```bash
code --uninstall-extension unilang.unilang-vscode
```

### From the VS Code UI:

1. Open the Extensions panel (`Cmd + Shift + X` or `Ctrl + Shift + X`).
2. Search for "UniLang."
3. Click the gear icon next to the extension.
4. Select **"Uninstall"**.
5. Reload VS Code when prompted.
