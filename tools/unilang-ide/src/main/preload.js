const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  // File operations
  readFile: (filePath) => ipcRenderer.invoke('read-file', filePath),
  saveFile: (filePath, content) => ipcRenderer.invoke('save-file', filePath, content),
  saveFileDialog: () => ipcRenderer.invoke('save-file-dialog'),
  readDirectory: (dirPath) => ipcRenderer.invoke('read-directory', dirPath),
  openFileDialog: () => ipcRenderer.invoke('open-file-dialog'),
  openFolderDialog: () => ipcRenderer.invoke('open-folder-dialog'),

  // Command execution
  runCommand: (command, args, cwd) => ipcRenderer.invoke('run-command', command, args, cwd),

  // UniLang CLI path (persisted in userData)
  getUnilangCliConfig: () => ipcRenderer.invoke('get-unilang-cli-config'),
  setUnilangCliPath: (absolutePath) => ipcRenderer.invoke('set-unilang-cli-path', absolutePath),
  pickUnilangCliExecutable: () => ipcRenderer.invoke('pick-unilang-cli-executable'),

  // Menu events from main process
  onMenuNewFile: (callback) => ipcRenderer.on('menu-new-file', callback),
  onMenuSave: (callback) => ipcRenderer.on('menu-save', callback),
  onMenuSaveAs: (callback) => ipcRenderer.on('menu-save-as', callback),
  onMenuBuild: (callback) => ipcRenderer.on('menu-build', callback),
  onMenuRun: (callback) => ipcRenderer.on('menu-run', callback),
  onFileOpened: (callback) => ipcRenderer.on('file-opened', (_event, filePath) => callback(filePath)),
  onFolderOpened: (callback) => ipcRenderer.on('folder-opened', (_event, folderPath) => callback(folderPath)),

  // Command output streams
  onCommandStdout: (callback) => ipcRenderer.on('command-stdout', (_event, data) => callback(data)),
  onCommandStderr: (callback) => ipcRenderer.on('command-stderr', (_event, data) => callback(data)),

  onUnilangCliStatus: (callback) =>
    ipcRenderer.on('unilang-cli-status', (_event, payload) => callback(payload))
});
