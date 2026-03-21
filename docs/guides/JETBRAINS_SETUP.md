# IntelliJ IDEA / PyCharm Setup Guide for UniLang

This guide walks you through installing the UniLang plugin for JetBrains IDEs. The plugin works with IntelliJ IDEA, PyCharm, WebStorm, CLion, GoLand, Rider, and all other JetBrains IDEs.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Download the Plugin](#step-1-download-the-plugin)
- [Step 2: Install the Plugin](#step-2-install-the-plugin)
- [Step 3: Restart the IDE](#step-3-restart-the-ide)
- [Step 4: Verify the Installation](#step-4-verify-the-installation)
- [Features](#features)
- [Building the Plugin from Source](#building-the-plugin-from-source)
- [Troubleshooting](#troubleshooting)
- [Uninstalling](#uninstalling)

---

## Prerequisites

Before you begin, make sure you have:

- **A JetBrains IDE** installed (IntelliJ IDEA, PyCharm, WebStorm, or any other JetBrains IDE). Download from [https://www.jetbrains.com](https://www.jetbrains.com).
- The IDE should be version **2023.1 or later**.
- **UniLang CLI** installed (optional, for running `.uniL` files from the IDE). See the [Installation Guide](INSTALLATION.md).

---

## Step 1: Download the Plugin

1. Go to the [UniLang GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Find the latest release.
3. Under **Assets**, download the file named `unilang-jetbrains.zip`.
4. Save it somewhere you can easily find it (for example, your Downloads folder).
5. **Do not unzip the file.** The IDE installs it directly from the `.zip`.

---

## Step 2: Install the Plugin

1. Open your JetBrains IDE (IntelliJ IDEA, PyCharm, etc.).
2. Open the **Settings** window:
   - **Windows / Linux:** Go to **File** in the top menu bar, then click **Settings**.
   - **macOS:** Go to the IDE name in the top menu bar (for example, **IntelliJ IDEA**), then click **Preferences** (or **Settings** on newer versions).
3. In the left sidebar of the Settings window, click **Plugins**.
4. At the top of the Plugins page, you will see a search bar and a gear icon. Click the **gear icon** (it looks like a small cogwheel).
5. From the dropdown menu, select **"Install Plugin from Disk..."**.
6. A file browser will open. Navigate to where you saved `unilang-jetbrains.zip`.
7. Select the file and click **OK** (or **Open** on some systems).
8. The IDE will install the plugin. You will see "UniLang" appear in the list of installed plugins.
9. Click **OK** to close the Settings window.

---

## Step 3: Restart the IDE

After installing the plugin, you need to restart the IDE for it to take effect.

1. A banner may appear at the top or bottom of the IDE saying **"Restart IDE to activate changes"**. If you see it, click **Restart**.
2. If no banner appears, manually restart the IDE:
   - Close the IDE completely.
   - Open it again.

---

## Step 4: Verify the Installation

1. Open a project in the IDE (any project will do, or create a new one).
2. Create a new file:
   - Right-click on the project folder in the left sidebar.
   - Select **New** from the context menu.
   - Select **File**.
   - Name the file `test.uniL` and press Enter.
3. Type the following code into the file:

   ```unilang
   // Python-style function
   def greet(name):
       return f"Hello, {name}!"

   // Java-style class
   public class App {
       public static void main(String[] args) {
           message = greet("World")
           print(message)
       }
   }
   ```

4. Check for syntax highlighting:
   - **Python keywords** (`def`, `return`) should appear in one color (typically blue).
   - **Java keywords** (`public`, `class`, `static`, `void`) should appear in another color (typically purple).
   - **Strings** should be highlighted in a distinct color (typically green).
   - **Comments** starting with `//` should appear dimmed or in gray.
5. You should also see a custom UniLang file icon next to `test.uniL` in the file tree.

If you see colored syntax, the plugin is working correctly.

---

## Features

The UniLang JetBrains plugin provides the following features:

### Syntax Highlighting

- **Python keywords** are displayed in blue: `def`, `class`, `import`, `from`, `return`, `if`, `elif`, `else`, `for`, `while`, `try`, `except`, `finally`, `with`, `as`, `lambda`, `yield`, `pass`, `break`, `continue`, `and`, `or`, `not`, `in`, `is`, `True`, `False`, `None`.
- **Java keywords** are displayed in purple: `public`, `private`, `protected`, `static`, `final`, `abstract`, `void`, `int`, `double`, `float`, `boolean`, `char`, `long`, `byte`, `short`, `String`, `new`, `this`, `super`, `extends`, `implements`, `interface`, `throws`, `throw`, `catch`, `synchronized`.
- **Strings**, **numbers**, and **comments** are each highlighted in their own distinct colors.

### Real-Time Error Annotations

The plugin highlights common issues as you type:

- **Unmatched brackets:** If you have an opening `(`, `[`, or `{` without a matching close, it will be highlighted with a red underline.
- **TODO and FIXME comments:** Comments containing `TODO` or `FIXME` are highlighted so they stand out.

### Code Completion

When you start typing, the IDE offers suggestions for:

- All UniLang keywords (both Python-style and Java-style).
- Common code patterns and structures.

### Custom File Type

- Files with the `.uniL` extension are automatically recognized as UniLang files.
- A custom file icon appears in the project tree next to `.uniL` files.

### Works Across All JetBrains IDEs

The plugin is compatible with:

- IntelliJ IDEA (Community and Ultimate)
- PyCharm (Community and Professional)
- WebStorm
- CLion
- GoLand
- Rider
- PhpStorm
- DataGrip
- RubyMine

---

## Building the Plugin from Source

If you want to build the plugin yourself (for development or to get the latest unreleased version):

### Step 1: Clone the Repository

```bash
git clone https://github.com/hiteshkumar2187/unilang.git
cd unilang
```

### Step 2: Navigate to the Plugin Directory

```bash
cd tools/jetbrains-plugin
```

### Step 3: Build the Plugin

```bash
./gradlew buildPlugin
```

On Windows, use:

```cmd
gradlew.bat buildPlugin
```

### Step 4: Find the Built Plugin

The built plugin ZIP file will be at:

```
build/distributions/unilang-intellij-*.zip
```

### Step 5: Install the Built Plugin

Follow the same [Step 2: Install the Plugin](#step-2-install-the-plugin) instructions above, but select the ZIP file from the `build/distributions/` folder instead of the downloaded one.

---

## Troubleshooting

### No syntax highlighting appears

1. **Check the file extension.** The file must end with `.uniL` (case-sensitive on macOS and Linux). `.uni` or `.unilang` will not work.
2. **Check that the plugin is enabled.** Go to **Settings** (or **Preferences**) > **Plugins** > **Installed** tab. Look for "UniLang" in the list. Make sure it is checked (enabled).
3. **Restart the IDE.** Some changes only take effect after a full restart.

### Plugin fails to install

1. **Check IDE version.** The plugin requires JetBrains IDE version 2023.1 or later. Go to the IDE's **About** screen (under the **Help** menu on Windows/Linux, or under the IDE name menu on macOS) to check your version.
2. **Check the ZIP file.** Make sure you downloaded `unilang-jetbrains.zip` and not a different file. The file should not be unzipped before installation.
3. **Try redownloading.** The file may have been corrupted during download. Delete it and download again.

### Plugin installed but no file icon appears

1. Close and reopen the project.
2. If the issue persists, invalidate caches: go to **File** > **Invalidate Caches...** > select **Invalidate and Restart**.

### Code completion not working

1. Make sure you are in a `.uniL` file.
2. Try pressing `Ctrl + Space` (Windows/Linux) or `Cmd + Space` (macOS) to manually trigger code completion.
3. If you are on macOS and `Cmd + Space` opens Spotlight instead, go to **System Preferences** > **Keyboard** > **Shortcuts** > **Spotlight** and change or disable the Spotlight shortcut.

---

## Uninstalling

1. Open the IDE.
2. Go to **Settings** (or **Preferences**) > **Plugins** > **Installed** tab.
3. Find "UniLang" in the list.
4. Click the **gear icon** next to the plugin name.
5. Select **"Uninstall"**.
6. Click **OK** to close Settings.
7. Restart the IDE when prompted.
