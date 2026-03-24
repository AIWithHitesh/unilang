const { app, BrowserWindow, Menu, dialog, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawn } = require('child_process');

const settingsStore = require('./settings-store');

let mainWindow = null;

/** @type {string | null} */
let cachedUnilangCli = null;

function invalidateUnilangCliCache() {
  cachedUnilangCli = null;
}

function unilangBinaryName() {
  return process.platform === 'win32' ? 'unilang.exe' : 'unilang';
}

/**
 * PATH used by GUI apps on macOS/Linux is often minimal (no ~/.cargo/bin, no Homebrew).
 * Prepend typical install locations so `unilang` works when installed but not on default PATH.
 */
function augmentedPathEnv() {
  const sep = path.delimiter;
  const home = os.homedir();
  const prefix = [
    path.join(home, '.cargo', 'bin'),
    '/opt/homebrew/bin',
    '/usr/local/bin',
  ].join(sep);
  const base = process.env.PATH || '';
  return `${prefix}${sep}${base}`;
}

/**
 * Resolve the UniLang CLI executable. Caches the result for the app session.
 *
 * Order: saved IDE path → UNILANG_CLI → bundled resource → monorepo target/{release,debug}
 * → ~/.cargo/bin → Homebrew /usr/local → literal `unilang` (uses augmented PATH + shell).
 */
function resolveUnilangCli() {
  if (cachedUnilangCli !== null) {
    return cachedUnilangCli;
  }

  const name = unilangBinaryName();

  const configured = settingsStore.getUnilangCliPath();
  if (configured && fs.existsSync(configured)) {
    cachedUnilangCli = configured;
    return cachedUnilangCli;
  }

  const fromEnv = process.env.UNILANG_CLI;
  if (fromEnv && fs.existsSync(fromEnv)) {
    cachedUnilangCli = fromEnv;
    return cachedUnilangCli;
  }

  if (app.isPackaged && process.resourcesPath) {
    const bundled = path.join(process.resourcesPath, 'bin', name);
    if (fs.existsSync(bundled)) {
      cachedUnilangCli = bundled;
      return cachedUnilangCli;
    }
  }

  let dir = __dirname;
  for (let i = 0; i < 10; i++) {
    const release = path.join(dir, 'target', 'release', name);
    const debug = path.join(dir, 'target', 'debug', name);
    if (fs.existsSync(release)) {
      cachedUnilangCli = release;
      return cachedUnilangCli;
    }
    if (fs.existsSync(debug)) {
      cachedUnilangCli = debug;
      return cachedUnilangCli;
    }
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }

  const home = os.homedir();
  const fixedCandidates = [
    path.join(home, '.cargo', 'bin', name),
    process.platform === 'darwin' ? '/opt/homebrew/bin/unilang' : null,
    process.platform !== 'win32' ? '/usr/local/bin/unilang' : null,
  ].filter(Boolean);

  for (const p of fixedCandidates) {
    if (fs.existsSync(p)) {
      cachedUnilangCli = p;
      return cachedUnilangCli;
    }
  }

  // Last resort: rely on shell + augmented PATH (finds unilang.exe on Windows too).
  cachedUnilangCli = 'unilang';
  return cachedUnilangCli;
}

function isAbsoluteExecutable(cmd) {
  return path.isAbsolute(cmd) || /^[A-Za-z]:[\\/]/.test(cmd);
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    backgroundColor: '#1e1e1e',
    title: 'UniLang IDE',
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  mainWindow.loadFile(path.join(__dirname, '..', 'renderer', 'index.html'));

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

function buildMenu() {
  const isMac = process.platform === 'darwin';

  const template = [
    ...(isMac ? [{
      label: app.name,
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'quit' }
      ]
    }] : []),
    {
      label: 'File',
      submenu: [
        {
          label: 'New File',
          accelerator: 'CmdOrCtrl+N',
          click: () => mainWindow?.webContents.send('menu-new-file')
        },
        {
          label: 'Open File...',
          accelerator: 'CmdOrCtrl+O',
          click: () => handleOpenFile()
        },
        {
          label: 'Open Folder...',
          accelerator: 'CmdOrCtrl+Shift+O',
          click: () => handleOpenFolder()
        },
        { type: 'separator' },
        {
          label: 'Save',
          accelerator: 'CmdOrCtrl+S',
          click: () => mainWindow?.webContents.send('menu-save')
        },
        {
          label: 'Save As...',
          accelerator: 'CmdOrCtrl+Shift+S',
          click: () => mainWindow?.webContents.send('menu-save-as')
        },
        { type: 'separator' },
        ...(isMac ? [] : [{ role: 'quit' }])
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' }
      ]
    },
    {
      label: 'Build',
      submenu: [
        {
          label: 'Build File',
          accelerator: 'CmdOrCtrl+B',
          click: () => mainWindow?.webContents.send('menu-build')
        },
        {
          label: 'Run File',
          accelerator: 'CmdOrCtrl+R',
          click: () => mainWindow?.webContents.send('menu-run')
        },
        { type: 'separator' },
        {
          label: 'Choose UniLang CLI…',
          click: () => handleChooseUnilangCli()
        },
        {
          label: 'Reset UniLang CLI to Auto-Detect',
          click: () => handleResetUnilangCliPath()
        }
      ]
    },
    {
      label: 'Help',
      submenu: [
        {
          label: 'About UniLang IDE',
          click: () => {
            dialog.showMessageBox(mainWindow, {
              type: 'info',
              title: 'About UniLang IDE',
              message: 'UniLang IDE v0.1.0',
              detail: 'A lightweight IDE for the UniLang programming language.\n\nUniLang combines Python + Java syntax in .uniL files.'
            });
          }
        },
        ...(isMac
          ? [
              {
                label: 'macOS: Gatekeeper (unsigned app & CLI)…',
                click: () => {
                  dialog.showMessageBox(mainWindow, {
                    type: 'info',
                    title: 'macOS Gatekeeper',
                    message:
                      'Downloads may be blocked because builds are not Apple-notarized (no Developer ID signing).',
                    detail:
                      'This is expected for open-source CI builds. The software is still safe if you trust the source.\n\n' +
                      'UniLang IDE — remove quarantine, then open:\n' +
                      'xattr -dr com.apple.quarantine /Applications/UniLang\\ IDE.app\n\n' +
                      'unilang CLI — same for the binary you copied (adjust path):\n' +
                      'xattr -dr com.apple.quarantine /usr/local/bin/unilang\n\n' +
                      'Or use System Settings → Privacy & Security → Open Anyway.\n\n' +
                      'Fully verified developer installs require Apple notarization (paid Developer Program).'
                  });
                }
              },
              { type: 'separator' }
            ]
          : []),
        {
          label: 'Toggle Developer Tools',
          accelerator: isMac ? 'Alt+Cmd+I' : 'Ctrl+Shift+I',
          click: () => mainWindow?.webContents.toggleDevTools()
        }
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

async function handleOpenFile() {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile'],
    filters: [
      { name: 'UniLang Files', extensions: ['uniL'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  });

  if (!result.canceled && result.filePaths.length > 0) {
    mainWindow?.webContents.send('file-opened', result.filePaths[0]);
  }
}

async function handleOpenFolder() {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory']
  });

  if (!result.canceled && result.filePaths.length > 0) {
    mainWindow?.webContents.send('folder-opened', result.filePaths[0]);
  }
}

function sendUnilangCliStatus() {
  if (!mainWindow?.webContents) return;
  invalidateUnilangCliCache();
  const resolved = resolveUnilangCli();
  const saved = settingsStore.getUnilangCliPath();
  const usesCustomPath = !!(
    saved &&
    fs.existsSync(saved) &&
    isAbsoluteExecutable(resolved) &&
    path.resolve(saved) === path.resolve(resolved)
  );
  mainWindow.webContents.send('unilang-cli-status', {
    resolvedPath: resolved,
    savedPath: saved,
    usesCustomPath
  });
}

async function handleChooseUnilangCli() {
  const result = await dialog.showOpenDialog(mainWindow, {
    title: 'Select UniLang CLI executable',
    properties: ['openFile'],
    filters:
      process.platform === 'win32'
        ? [
            { name: 'Executable', extensions: ['exe'] },
            { name: 'All Files', extensions: ['*'] }
          ]
        : [{ name: 'All Files', extensions: ['*'] }]
  });
  if (result.canceled || result.filePaths.length === 0) return;
  const chosen = result.filePaths[0];
  if (!fs.existsSync(chosen)) {
    await dialog.showMessageBox(mainWindow, {
      type: 'error',
      title: 'UniLang CLI',
      message: 'That file does not exist.'
    });
    return;
  }
  settingsStore.setUnilangCliPath(chosen);
  invalidateUnilangCliCache();
  await dialog.showMessageBox(mainWindow, {
    type: 'info',
    title: 'UniLang CLI',
    message: 'Saved UniLang CLI path.',
    detail: chosen
  });
  sendUnilangCliStatus();
}

async function handleResetUnilangCliPath() {
  settingsStore.setUnilangCliPath(null);
  invalidateUnilangCliCache();
  await dialog.showMessageBox(mainWindow, {
    type: 'info',
    title: 'UniLang CLI',
    message: 'Cleared saved path. The IDE will auto-detect the CLI again.'
  });
  sendUnilangCliStatus();
}

// --- IPC Handlers ---

ipcMain.handle('read-file', async (_event, filePath) => {
  try {
    const content = await fs.promises.readFile(filePath, 'utf-8');
    return { success: true, content, filePath };
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('save-file', async (_event, filePath, content) => {
  try {
    await fs.promises.writeFile(filePath, content, 'utf-8');
    return { success: true, filePath };
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('save-file-dialog', async () => {
  const result = await dialog.showSaveDialog(mainWindow, {
    filters: [
      { name: 'UniLang Files', extensions: ['uniL'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  });
  if (result.canceled) return { success: false, canceled: true };
  return { success: true, filePath: result.filePath };
});

ipcMain.handle('read-directory', async (_event, dirPath) => {
  try {
    const entries = await fs.promises.readdir(dirPath, { withFileTypes: true });
    const items = entries
      .filter(e => !e.name.startsWith('.'))
      .map(e => ({
        name: e.name,
        path: path.join(dirPath, e.name),
        isDirectory: e.isDirectory()
      }))
      .sort((a, b) => {
        if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
    return { success: true, items, dirPath };
  } catch (err) {
    return { success: false, error: err.message };
  }
});

ipcMain.handle('run-command', async (_event, command, args, cwd) => {
  return new Promise((resolve) => {
    let stdout = '';
    let stderr = '';

    let cmd = command;
    if (command === 'unilang') {
      cmd = resolveUnilangCli();
    }

    const useAbsolute = isAbsoluteExecutable(cmd);
    const env = {
      ...process.env,
      PATH: augmentedPathEnv()
    };

    const proc = spawn(cmd, args, {
      cwd: cwd || process.cwd(),
      shell: !useAbsolute,
      env
    });

    proc.stdout.on('data', (data) => {
      const text = data.toString();
      stdout += text;
      mainWindow?.webContents.send('command-stdout', text);
    });

    proc.stderr.on('data', (data) => {
      const text = data.toString();
      stderr += text;
      mainWindow?.webContents.send('command-stderr', text);
    });

    proc.on('close', (code) => {
      resolve({ success: code === 0, code, stdout, stderr });
    });

    proc.on('error', (err) => {
      let hint = err.message;
      if (command === 'unilang') {
        hint +=
          '\nInstall the UniLang CLI, use Build → Choose UniLang CLI…, or set UNILANG_CLI ' +
          'to the full path of the unilang binary.';
      }
      resolve({ success: false, code: -1, stdout, stderr: hint });
    });
  });
});

ipcMain.handle('open-file-dialog', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile'],
    filters: [
      { name: 'UniLang Files', extensions: ['uniL'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  });
  if (result.canceled) return { success: false, canceled: true };
  return { success: true, filePath: result.filePaths[0] };
});

ipcMain.handle('open-folder-dialog', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory']
  });
  if (result.canceled) return { success: false, canceled: true };
  return { success: true, folderPath: result.filePaths[0] };
});

ipcMain.handle('get-unilang-cli-config', async () => {
  invalidateUnilangCliCache();
  const resolved = resolveUnilangCli();
  const saved = settingsStore.getUnilangCliPath();
  const usesCustomPath = !!(
    saved &&
    fs.existsSync(saved) &&
    isAbsoluteExecutable(resolved) &&
    path.resolve(saved) === path.resolve(resolved)
  );
  return {
    savedPath: saved,
    resolvedPath: resolved,
    usesCustomPath
  };
});

ipcMain.handle('set-unilang-cli-path', async (_event, absolutePath) => {
  if (absolutePath == null || absolutePath === '') {
    settingsStore.setUnilangCliPath(null);
  } else if (typeof absolutePath === 'string' && fs.existsSync(absolutePath)) {
    settingsStore.setUnilangCliPath(absolutePath);
  } else {
    return { success: false, error: 'Path does not exist' };
  }
  invalidateUnilangCliCache();
  sendUnilangCliStatus();
  return { success: true };
});

ipcMain.handle('pick-unilang-cli-executable', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    title: 'Select UniLang CLI executable',
    properties: ['openFile'],
    filters:
      process.platform === 'win32'
        ? [
            { name: 'Executable', extensions: ['exe'] },
            { name: 'All Files', extensions: ['*'] }
          ]
        : [{ name: 'All Files', extensions: ['*'] }]
  });
  if (result.canceled || result.filePaths.length === 0) {
    return { success: false, canceled: true };
  }
  return { success: true, filePath: result.filePaths[0] };
});

// --- App Lifecycle ---

app.whenReady().then(() => {
  settingsStore.warmup();
  buildMenu();
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
