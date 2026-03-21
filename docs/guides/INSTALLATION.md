# UniLang Installation Guide

This guide walks you through installing UniLang on macOS, Windows, and Linux. By the end, you will have the `unilang` command-line tool ready to use.

---

## Table of Contents

- [macOS Installation](#macos-installation)
- [Windows Installation](#windows-installation)
- [Linux Installation](#linux-installation)
- [Build from Source (All Platforms)](#build-from-source-all-platforms)
- [Verify Installation](#verify-installation)
- [Troubleshooting](#troubleshooting)

---

## macOS Installation

Choose one of the three options below.

### Option 1: Download DMG (Recommended)

This is the easiest way to install UniLang on macOS.

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Under the latest release, find the **Assets** section.
3. Download the correct file for your Mac:
   - **Apple Silicon** (M1, M2, M3, M4): `unilang-cli-macos-arm64.dmg`
   - **Intel**: `unilang-cli-macos-x86_64.dmg`
   - Not sure which Mac you have? Click the Apple menu in the top-left corner, then "About This Mac." If it says "Apple M1" (or M2/M3/M4), choose Apple Silicon. If it says "Intel," choose Intel.
4. Open the downloaded `.dmg` file by double-clicking it.
5. A Finder window will appear. Drag the `unilang` binary to your **Applications** folder, or copy it to `/usr/local/bin/` for command-line access:
   ```bash
   sudo cp /Volumes/UniLang/unilang /usr/local/bin/
   sudo cp /Volumes/UniLang/unilang-lsp /usr/local/bin/
   ```
6. Eject the DMG by right-clicking the mounted volume in Finder and selecting "Eject."
7. If macOS blocks the app with a message like "unilang can't be opened because it is from an unidentified developer":
   - Go to **System Preferences** (or **System Settings** on macOS Ventura+).
   - Click **Privacy & Security**.
   - Scroll down and click **"Open Anyway"** next to the UniLang message.
8. Open a terminal and verify the installation:
   ```bash
   unilang --version
   ```

### Option 2: Download tar.gz

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Download the correct archive for your Mac:
   - **Apple Silicon**: `unilang-cli-macos-arm64.tar.gz`
   - **Intel**: `unilang-cli-macos-x86_64.tar.gz`
3. Open a terminal and navigate to your Downloads folder:
   ```bash
   cd ~/Downloads
   ```
4. Extract the archive:
   ```bash
   tar xzf unilang-cli-macos-*.tar.gz
   ```
5. Copy the binaries to a location on your PATH:
   ```bash
   sudo cp unilang-cli-*/bin/* /usr/local/bin/
   ```
6. Verify the installation:
   ```bash
   unilang --version
   ```

### Option 3: Homebrew (Coming Soon)

Homebrew support is planned for a future release. In the meantime, use Option 1 or Option 2.

---

## Windows Installation

### Option 1: Download ZIP Archive (Recommended)

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Download `unilang-cli-windows-x86_64.zip`.
3. Right-click the downloaded ZIP file and select **"Extract All..."**.
4. Choose a permanent location for the extracted files. We recommend:
   ```
   C:\Program Files\UniLang\
   ```
5. Add UniLang to your system PATH so you can run it from any Command Prompt or PowerShell window:
   - Press the **Windows key**, type **"Environment Variables"**, and click **"Edit the system environment variables"**.
   - In the System Properties window, click **"Environment Variables..."** at the bottom.
   - Under **"System variables"** (bottom section), find the variable named **Path** and select it.
   - Click **"Edit..."**.
   - Click **"New"** and add:
     ```
     C:\Program Files\UniLang\bin
     ```
   - Click **OK** on all three open windows to save.
6. **Important:** Close any open Command Prompt or PowerShell windows. The PATH change only takes effect in newly opened windows.
7. Open a **new** Command Prompt or PowerShell window.
8. Verify the installation:
   ```cmd
   unilang --version
   ```

### Option 2: Build from Source

See the [Build from Source](#build-from-source-all-platforms) section below.

---

## Linux Installation

### Option 1: Download tar.gz (Recommended)

These steps work on Ubuntu, Debian, Fedora, Arch, and most other Linux distributions.

1. Go to the [GitHub Releases](https://github.com/hiteshkumar2187/unilang/releases) page.
2. Download `unilang-cli-linux-x86_64.tar.gz`.
   - For ARM-based systems (e.g., Raspberry Pi 4 64-bit): download `unilang-cli-linux-aarch64.tar.gz` if available.
3. Open a terminal and navigate to where you downloaded the file:
   ```bash
   cd ~/Downloads
   ```
4. Extract the archive:
   ```bash
   tar xzf unilang-cli-linux-x86_64.tar.gz
   ```
5. Copy the binaries to `/usr/local/bin/`:
   ```bash
   sudo cp unilang-cli-*/bin/* /usr/local/bin/
   ```
6. Verify the installation:
   ```bash
   unilang --version
   ```

### Option 2: Build from Source

See the [Build from Source](#build-from-source-all-platforms) section below.

---

## Build from Source (All Platforms)

Building from source works on macOS, Windows, and Linux. You will need **Rust 1.75 or later** installed.

### Step 1: Install Rust (if you don't have it)

If you already have Rust installed, skip to Step 2.

1. Go to [https://rustup.rs](https://rustup.rs).
2. Follow the instructions for your platform:
   - **macOS / Linux:** Run this in a terminal:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
   - **Windows:** Download and run `rustup-init.exe` from the website.
3. Restart your terminal (or run `source $HOME/.cargo/env` on macOS/Linux).
4. Verify Rust is installed:
   ```bash
   rustc --version
   ```
   You should see version 1.75.0 or later.

### Step 2: Clone the UniLang repository

```bash
git clone https://github.com/hiteshkumar2187/unilang.git
cd unilang
```

### Step 3: Build in release mode

```bash
cargo build --release
```

This will take a few minutes the first time. The compiled binaries will be in the `target/release/` directory.

### Step 4: Install the binaries

**macOS / Linux:**
```bash
sudo cp target/release/unilang-cli /usr/local/bin/unilang
sudo cp target/release/unilang-lsp /usr/local/bin/unilang-lsp
```

**Windows (run Command Prompt as Administrator):**
```cmd
copy target\release\unilang-cli.exe "C:\Program Files\UniLang\bin\unilang.exe"
copy target\release\unilang-lsp.exe "C:\Program Files\UniLang\bin\unilang-lsp.exe"
```

### Step 5: Verify

```bash
unilang --version
```

---

## Verify Installation

After installing UniLang using any method above, run these commands to confirm everything is working.

### Check the version

```bash
unilang --version
```

Expected output (version number may vary):

```
unilang 0.1.0
```

### Check the help menu

```bash
unilang --help
```

This should display a list of available commands and options.

### Run a quick test

Create a file called `test.uniL` with the following content:

```unilang
print("UniLang is installed!")
```

Then run it:

```bash
unilang run test.uniL
```

Expected output:

```
UniLang is installed!
```

If you see that output, UniLang is installed correctly. You can delete `test.uniL` or keep it for reference.

---

## Troubleshooting

### "command not found: unilang"

This means the `unilang` binary is not on your system PATH.

- **macOS / Linux:** Make sure you copied the binary to `/usr/local/bin/`. Run `ls /usr/local/bin/unilang` to check if it exists.
- **Windows:** Make sure you added `C:\Program Files\UniLang\bin` to your PATH environment variable and opened a **new** terminal window.

### "Permission denied" on macOS or Linux

You need to use `sudo` when copying to `/usr/local/bin/`:

```bash
sudo cp unilang /usr/local/bin/
```

### macOS Gatekeeper blocks the app

Go to **System Preferences** (or **System Settings**) > **Privacy & Security** and click **"Open Anyway"**.

### Rust build fails with "edition 2021 is not supported"

Your Rust version is too old. Update Rust:

```bash
rustup update
```

Then try `cargo build --release` again.

### Still stuck?

Open an issue at [https://github.com/hiteshkumar2187/unilang/issues](https://github.com/hiteshkumar2187/unilang/issues) with:
- Your operating system and version
- The command you ran
- The full error message
