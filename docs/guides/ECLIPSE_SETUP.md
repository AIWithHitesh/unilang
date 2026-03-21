# Eclipse Setup Guide for UniLang

This guide walks you through installing the UniLang plugin for Eclipse IDE. By the end, you will have syntax highlighting and `.uniL` file support in Eclipse.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Download the Plugin](#step-1-download-the-plugin)
- [Step 2: Install the Plugin](#step-2-install-the-plugin)
- [Step 3: Restart Eclipse](#step-3-restart-eclipse)
- [Step 4: Verify the Installation](#step-4-verify-the-installation)
- [Features](#features)
- [Troubleshooting](#troubleshooting)
- [Uninstalling](#uninstalling)

---

## Prerequisites

Before you begin, make sure you have:

- **Eclipse IDE** installed (version 2023-03 or later recommended). Download from [https://www.eclipse.org/downloads/](https://www.eclipse.org/downloads/).
- **UniLang CLI** installed (optional, for running `.uniL` files). See the [Installation Guide](INSTALLATION.md).

---

## Step 1: Download the Plugin

1. Go to the [UniLang GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Find the latest release.
3. Under **Assets**, download the file named `unilang-eclipse.jar`.
4. Save it somewhere you can easily find it (for example, your Downloads folder).

---

## Step 2: Install the Plugin

You can install the plugin using either the dropins folder (simpler) or the Eclipse update site (alternative method).

### Option A: Dropins Folder (Easiest)

The dropins folder is a special Eclipse directory. Any plugin JAR file placed in this folder is automatically loaded when Eclipse starts.

1. **Find your Eclipse installation directory.** This is the folder where `eclipse.exe` (Windows), `Eclipse.app` (macOS), or `eclipse` (Linux) is located.
   - **macOS:** Right-click `Eclipse.app` in your Applications folder, then select **"Show Package Contents"**. Navigate to `Contents/Eclipse/`.
   - **Windows:** Typically `C:\eclipse\` or wherever you extracted Eclipse.
   - **Linux:** Typically `~/eclipse/` or `/opt/eclipse/`.
2. Inside the Eclipse installation directory, find the folder named **`dropins`**. If it does not exist, create it.
3. Copy the downloaded `unilang-eclipse.jar` file into the `dropins` folder.
   - **macOS example:**
     ```bash
     cp ~/Downloads/unilang-eclipse.jar /Applications/Eclipse.app/Contents/Eclipse/dropins/
     ```
   - **Windows example:**
     ```cmd
     copy C:\Users\YourName\Downloads\unilang-eclipse.jar C:\eclipse\dropins\
     ```
   - **Linux example:**
     ```bash
     cp ~/Downloads/unilang-eclipse.jar ~/eclipse/dropins/
     ```

### Option B: Install via Eclipse UI

1. Open Eclipse.
2. Go to **Help** in the top menu bar.
3. Click **"Install New Software..."**.
4. In the "Install" dialog, click the **"Add..."** button.
5. Click **"Archive..."** and browse to the downloaded `unilang-eclipse.jar` file.
6. Select the file and click **Open**.
7. Give the repository a name (for example, `UniLang Plugin`) and click **Add**.
8. Eclipse will load the available items. Check the box next to **"UniLang Support"**.
9. Click **Next**, then **Next** again to review.
10. Accept the license agreement and click **Finish**.
11. If Eclipse warns about unsigned content, click **"Install Anyway"** to proceed.

---

## Step 3: Restart Eclipse

After installing the plugin, you must restart Eclipse.

1. If Eclipse shows a dialog asking to restart, click **"Restart Now"**.
2. If no dialog appears, manually restart Eclipse:
   - Close Eclipse completely (File > Exit on Windows/Linux, Eclipse > Quit on macOS).
   - Open Eclipse again.

---

## Step 4: Verify the Installation

1. Open a project in Eclipse (any project will do, or create a new one).
2. Create a new file:
   - Right-click on the project in the **Package Explorer** (left sidebar).
   - Select **New** > **File**.
   - Name the file `test.uniL` and click **Finish**.
3. The file will open in the editor. Type the following code:

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
   - Keywords like `def`, `class`, `public`, `return` should be colored.
   - Strings should appear in a different color than regular code.
   - Comments starting with `//` should appear in a distinct color (usually gray or green).

5. If you see colored syntax, the plugin is working correctly.

---

## Features

The UniLang Eclipse plugin provides the following features:

### Syntax Highlighting

All Python and Java keywords are highlighted with distinct colors:

- **Python keywords:** `def`, `class`, `import`, `from`, `return`, `if`, `elif`, `else`, `for`, `while`, `try`, `except`, `finally`, `with`, `as`, `lambda`, `pass`, `break`, `continue`, `True`, `False`, `None`
- **Java keywords:** `public`, `private`, `protected`, `static`, `final`, `void`, `int`, `double`, `float`, `boolean`, `String`, `new`, `this`, `super`, `extends`, `implements`, `interface`, `throw`, `throws`, `catch`
- **Strings:** Single-quoted, double-quoted, and f-strings
- **Comments:** Both `//` and `#` styles
- **Numbers:** Integer and floating-point literals

### File Type Recognition

Files ending in `.uniL` are automatically recognized as UniLang source files and opened with the UniLang editor.

### Bracket Matching

When you click on a bracket, parenthesis, or brace, Eclipse highlights the matching pair.

### Configurable Colors

You can customize the syntax highlighting colors:

1. Go to **Window** > **Preferences** (Windows/Linux) or **Eclipse** > **Preferences** (macOS).
2. Navigate to **UniLang** > **Syntax Coloring**.
3. Adjust colors for keywords, strings, comments, and other elements.
4. Click **Apply and Close**.

---

## Troubleshooting

### No syntax highlighting appears

1. **Check the file extension.** The file must end with `.uniL` (case-sensitive on macOS and Linux).
2. **Check that the plugin is loaded.** Go to **Help** > **About Eclipse IDE** > **Installation Details** > **Plug-ins** tab. Look for entries that mention "UniLang." If none appear, the plugin is not installed.
3. **Try Option B installation.** If you used the dropins folder method and it did not work, try the Eclipse UI method instead.
4. **Restart Eclipse.** Close Eclipse completely and reopen it.

### Eclipse does not recognize the .uniL file type

1. Go to **Window** > **Preferences** (Windows/Linux) or **Eclipse** > **Preferences** (macOS).
2. Navigate to **General** > **Editors** > **File Associations**.
3. Click **Add...** next to the "File types" list.
4. Enter `*.uniL` and click **OK**.
5. With `*.uniL` selected, click **Add...** next to the "Associated editors" list.
6. Select **"UniLang Editor"** and click **OK**.
7. Click **Apply and Close**.

### Plugin JAR fails to load from dropins

1. Make sure you placed the JAR in the correct `dropins` directory. On macOS, this is inside the `.app` package contents, not next to the `.app` file.
2. Make sure the JAR file is not corrupted. Try redownloading it.
3. Check Eclipse's error log: go to **Window** > **Show View** > **Error Log** and look for messages related to UniLang.

### Colors are hard to read with my theme

If you use a dark theme and the default colors are not readable:

1. Go to **Window** > **Preferences** > **UniLang** > **Syntax Coloring**.
2. Adjust the colors to work with your theme.
3. Click **Apply and Close**.

---

## Uninstalling

### If you used the dropins folder method:

1. Close Eclipse.
2. Navigate to the `dropins` folder inside your Eclipse installation directory.
3. Delete `unilang-eclipse.jar`.
4. Reopen Eclipse.

### If you used the Eclipse UI method:

1. Go to **Help** > **About Eclipse IDE** > **Installation Details**.
2. Select the **Installed Software** tab.
3. Find "UniLang Support" in the list.
4. Select it and click **Uninstall...**.
5. Follow the prompts and restart Eclipse.
