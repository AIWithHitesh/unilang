# UniLang IDE -- Standalone Editor Setup Guide

UniLang IDE is a standalone code editor built specifically for UniLang development. It comes with everything you need out of the box: syntax highlighting, an integrated terminal, file navigation, and build tools. No plugins or configuration required.

---

## Table of Contents

- [Download](#download)
- [macOS Installation](#macos-installation)
- [Windows Installation](#windows-installation)
- [Linux Installation](#linux-installation)
- [First Launch](#first-launch)
- [Features](#features)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Troubleshooting](#troubleshooting)
- [Uninstalling](#uninstalling)

---

## Download

Download the UniLang IDE for your platform from the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.

| Platform | File | Approximate Size |
|---|---|---|
| macOS | `UniLang-IDE.dmg` | ~80 MB |
| Windows | `UniLang-IDE-Setup.exe` | ~60 MB |
| Linux | `UniLang-IDE.AppImage` | ~70 MB |

---

## macOS Installation

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page and download `UniLang-IDE.dmg`.
2. Open the downloaded `UniLang-IDE.dmg` file by double-clicking it.
3. A Finder window will appear showing the UniLang IDE icon and an Applications folder shortcut.
4. Drag the **"UniLang IDE"** icon onto the **Applications** folder.
5. Wait for the copy to complete (the progress bar will appear briefly).
6. Eject the DMG by right-clicking the "UniLang IDE" volume on your Desktop (or in the Finder sidebar) and selecting **"Eject"**.
7. Open **Applications** in Finder and double-click **"UniLang IDE"** to launch it.
8. **If macOS blocks the app** with a message like "UniLang IDE can't be opened because it is from an unidentified developer":
   - Do **not** click "Move to Trash."
   - Open **System Preferences** (or **System Settings** on macOS Ventura and later).
   - Click **Privacy & Security**.
   - Scroll down. You will see a message about UniLang IDE being blocked.
   - Click **"Open Anyway"**.
   - In the confirmation dialog, click **"Open"**.
   - You only need to do this once. After the first launch, macOS will remember your choice.

---

## Windows Installation

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page and download `UniLang-IDE-Setup.exe`.
2. Open the downloaded `UniLang-IDE-Setup.exe` file by double-clicking it.
3. **If Windows Defender SmartScreen blocks the installer** with a message like "Windows protected your PC":
   - Click **"More info"** (this text is easy to miss -- it is a small link below the warning message).
   - Click **"Run anyway"**.
4. The installation wizard will open. Follow these steps:
   - **Welcome screen:** Click **Next**.
   - **License Agreement:** Read the license, check "I accept the agreement," and click **Next**.
   - **Installation Location:** The default is `C:\Program Files\UniLang IDE\`. You can change this if you want, but the default is fine. Click **Next**.
   - **Start Menu Folder:** Leave as default. Click **Next**.
   - **Additional Tasks:** Check **"Create a desktop shortcut"** if you want one. Click **Next**.
   - **Ready to Install:** Click **Install**.
5. Wait for the installation to complete (this takes about 30 seconds).
6. On the final screen, check **"Launch UniLang IDE"** and click **Finish**.
7. UniLang IDE will open.

---

## Linux Installation

### Option 1: AppImage (Recommended)

AppImage files run on most Linux distributions without installation.

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page and download `UniLang-IDE.AppImage`.
2. Open a terminal and navigate to where you downloaded the file:
   ```bash
   cd ~/Downloads
   ```
3. Make the AppImage file executable:
   ```bash
   chmod +x UniLang-IDE.AppImage
   ```
4. Run the IDE:
   ```bash
   ./UniLang-IDE.AppImage
   ```

### Option 2: System-Wide Installation

If you want UniLang IDE available to all users on the system:

1. Download the AppImage as described above.
2. Move it to a system-wide location:
   ```bash
   sudo mkdir -p /opt/unilang-ide
   sudo mv ~/Downloads/UniLang-IDE.AppImage /opt/unilang-ide/
   sudo chmod +x /opt/unilang-ide/UniLang-IDE.AppImage
   ```
3. Create a symbolic link so you can launch it from anywhere:
   ```bash
   sudo ln -s /opt/unilang-ide/UniLang-IDE.AppImage /usr/local/bin/unilang-ide
   ```
4. Now you can launch the IDE from any terminal:
   ```bash
   unilang-ide
   ```

### Option 3: Create a Desktop Entry

If you want UniLang IDE to appear in your application launcher (GNOME, KDE, etc.):

1. Install the AppImage system-wide as described in Option 2.
2. Create a desktop entry file:
   ```bash
   sudo tee /usr/share/applications/unilang-ide.desktop > /dev/null << 'ENTRY'
   [Desktop Entry]
   Name=UniLang IDE
   Comment=UniLang Development Environment
   Exec=/opt/unilang-ide/UniLang-IDE.AppImage
   Type=Application
   Categories=Development;IDE;
   Terminal=false
   ENTRY
   ```
3. The IDE should now appear in your application launcher. You may need to log out and log back in for it to show up.

---

## First Launch

When you open UniLang IDE for the first time:

1. **Welcome screen:** You will see a welcome screen with options to create a new file, open a file, or open a folder.
2. **Create a new file:** Click **"New File"** (or press `Ctrl + N` / `Cmd + N`).
3. **Save it as a .uniL file:** Press `Ctrl + S` / `Cmd + S`, name the file `hello.uniL`, and choose where to save it.
4. **Type some code:**

   ```unilang
   // Your first program in UniLang IDE
   print("Hello from UniLang IDE!")

   def add(a, b):
       return a + b

   result = add(3, 7)
   print(f"3 + 7 = {result}")
   ```

5. **Run the program:** Click the **Run** button in the toolbar (the green triangle icon), or press `Ctrl + R` / `Cmd + R`.
6. The output will appear in the **integrated terminal** at the bottom of the screen:
   ```
   Hello from UniLang IDE!
   3 + 7 = 10
   ```

---

## Features

### Code Editor

- Full syntax highlighting for `.uniL` files, including both Python-style and Java-style keywords.
- Line numbers displayed along the left margin.
- Current line highlighting to show where your cursor is.
- Automatic indentation for both Python-style (after `:`) and Java-style (inside `{}`) blocks.
- Bracket matching: clicking on any bracket highlights its matching pair.

### File Tree Panel

- The left sidebar displays a file tree for your project.
- Open a folder to see all files and subfolders.
- Click any file to open it in the editor.
- Right-click for options to create, rename, or delete files.

### Integrated Terminal

- The bottom panel contains a terminal for running commands.
- You can run `unilang run yourfile.uniL` directly in this terminal.
- The terminal uses your system's default shell (bash, zsh, PowerShell, etc.).

### Build and Run Buttons

- The toolbar at the top includes:
  - **Build** button (hammer icon): Compiles the current file (`unilang compile`).
  - **Run** button (green triangle): Runs the current file (`unilang run`).
  - **Check** button (checkmark icon): Checks the current file for errors (`unilang check`).

### Tabs

- Open multiple files in tabs.
- Click a tab to switch between files.
- Middle-click a tab (or click the X on the tab) to close it.
- Modified files show a dot or indicator on the tab until saved.

---

## Keyboard Shortcuts

These shortcuts work on all platforms. On macOS, replace `Ctrl` with `Cmd`.

| Shortcut | Action |
|---|---|
| `Ctrl/Cmd + S` | Save the current file |
| `Ctrl/Cmd + B` | Build (compile) the current file |
| `Ctrl/Cmd + R` | Run the current file |
| `Ctrl/Cmd + N` | Create a new file |
| `Ctrl/Cmd + O` | Open a file |
| `Ctrl/Cmd + W` | Close the current tab |
| `Ctrl/Cmd + Z` | Undo |
| `Ctrl/Cmd + Shift + Z` | Redo |
| `Ctrl/Cmd + F` | Find in file |
| `Ctrl/Cmd + H` | Find and replace |
| `Ctrl/Cmd + G` | Go to line number |
| `Ctrl/Cmd + /` | Toggle line comment |
| `Ctrl/Cmd + +` | Increase font size |
| `Ctrl/Cmd + -` | Decrease font size |
| `Ctrl/Cmd + 0` | Reset font size to default |
| `` Ctrl/Cmd + ` `` | Toggle the integrated terminal |

---

## Troubleshooting

### IDE does not open on macOS

If double-clicking the app does nothing:

1. Open **System Preferences** (or **System Settings**) > **Privacy & Security**.
2. Look for a message about UniLang IDE being blocked.
3. Click **"Open Anyway"**.
4. See the detailed instructions in the [macOS Installation](#macos-installation) section above.

### IDE does not open on Linux

1. Make sure the AppImage is executable:
   ```bash
   chmod +x UniLang-IDE.AppImage
   ```
2. Some Linux distributions require FUSE to run AppImages. If you see an error about FUSE:
   - **Ubuntu / Debian:**
     ```bash
     sudo apt install libfuse2
     ```
   - **Fedora:**
     ```bash
     sudo dnf install fuse-libs
     ```
3. Try running the AppImage with the `--appimage-extract-and-run` flag:
   ```bash
   ./UniLang-IDE.AppImage --appimage-extract-and-run
   ```

### Run button does not work

1. Make sure the UniLang CLI (`unilang`) is installed and on your PATH. Open the integrated terminal in the IDE and type:
   ```bash
   unilang --version
   ```
2. If this says "command not found," install the CLI first. See the [Installation Guide](INSTALLATION.md).
3. If `unilang` is installed but the Run button still does not work, check the IDE settings for the path to the UniLang binary.

### File does not have syntax highlighting

1. Make sure the file name ends with `.uniL` (case-sensitive on macOS and Linux).
2. Try closing the file and reopening it.
3. If the problem persists, try restarting the IDE.

### Text is too small or too large

Use the keyboard shortcuts to adjust font size:
- **Increase:** `Ctrl/Cmd + +` (plus key)
- **Decrease:** `Ctrl/Cmd + -` (minus key)
- **Reset to default:** `Ctrl/Cmd + 0`

---

## Uninstalling

### macOS

1. Open **Applications** in Finder.
2. Drag **"UniLang IDE"** to the Trash.
3. Empty the Trash.

### Windows

1. Open **Settings** > **Apps** > **Apps & features** (or **Installed apps** on Windows 11).
2. Search for "UniLang IDE."
3. Click on it and select **Uninstall**.
4. Follow the prompts.

Alternatively, use **Control Panel** > **Programs and Features** > select "UniLang IDE" > click **Uninstall**.

### Linux

If you used the AppImage directly:

1. Delete the AppImage file:
   ```bash
   rm ~/Downloads/UniLang-IDE.AppImage
   ```

If you installed system-wide:

1. Remove the files:
   ```bash
   sudo rm /usr/local/bin/unilang-ide
   sudo rm -rf /opt/unilang-ide
   sudo rm /usr/share/applications/unilang-ide.desktop
   ```
