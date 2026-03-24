/**
 * UniLang IDE - Main Renderer Entry
 *
 * Initializes all components, wires up toolbar and keyboard shortcuts,
 * connects the editor, file tree, and terminal together.
 */

(function () {
  'use strict';

  // ----- Initialize Components -----
  const editor = new Editor();
  const fileTree = new FileTree();
  const terminal = new Terminal();

  // DOM references
  const btnNew = document.getElementById('btn-new');
  const btnOpen = document.getElementById('btn-open');
  const btnOpenFolder = document.getElementById('btn-open-folder');
  const btnSave = document.getElementById('btn-save');
  const btnBuild = document.getElementById('btn-build');
  const btnRun = document.getElementById('btn-run');
  const statusFilename = document.getElementById('status-filename');
  const editorTabs = document.getElementById('editor-tabs');
  const tabPlaceholder = document.getElementById('editor-tab-placeholder');
  const sidebarResize = document.getElementById('sidebar-resize');
  const sidebarPanel = document.getElementById('sidebar-panel');
  const terminalResize = document.getElementById('terminal-resize');
  const terminalPanel = document.getElementById('terminal-panel');

  // ----- State -----
  let currentWorkDir = null;

  // ----- File Operations -----

  function newFile() {
    editor.setValue('');
    editor.setFilePath(null);
    editor.isModified = false;
    updateTab(null);
    statusFilename.textContent = 'Untitled';
    editor.focus();
  }

  async function openFile(filePath) {
    if (!filePath) {
      const result = await window.electronAPI.openFileDialog();
      if (!result.success) return;
      filePath = result.filePath;
    }

    const readResult = await window.electronAPI.readFile(filePath);
    if (!readResult.success) {
      terminal.writeStderr(`Failed to open file: ${readResult.error}`);
      return;
    }

    editor.setValue(readResult.content);
    editor.setFilePath(filePath);
    editor.markSaved();
    updateTab(filePath);
    statusFilename.textContent = getFileName(filePath);
    fileTree.setCurrentFile(filePath);

    // Set working directory from file
    if (!currentWorkDir) {
      const dir = getDirectory(filePath);
      currentWorkDir = dir;
    }

    editor.focus();
  }

  async function openFolder(folderPath) {
    if (!folderPath) {
      const result = await window.electronAPI.openFolderDialog();
      if (!result.success) return;
      folderPath = result.folderPath;
    }

    currentWorkDir = folderPath;
    await fileTree.setRoot(folderPath);
    terminal.writeSystem(`Opened folder: ${folderPath}`);
  }

  async function saveFile() {
    let filePath = editor.getFilePath();

    if (!filePath) {
      return saveFileAs();
    }

    const content = editor.getValue();
    const result = await window.electronAPI.saveFile(filePath, content);
    if (!result.success) {
      terminal.writeStderr(`Failed to save: ${result.error}`);
      return;
    }

    editor.markSaved();
    terminal.writeSuccess(`Saved: ${filePath}`);
  }

  async function saveFileAs() {
    const dialogResult = await window.electronAPI.saveFileDialog();
    if (!dialogResult.success) return;

    const filePath = dialogResult.filePath;
    const content = editor.getValue();
    const result = await window.electronAPI.saveFile(filePath, content);
    if (!result.success) {
      terminal.writeStderr(`Failed to save: ${result.error}`);
      return;
    }

    editor.setFilePath(filePath);
    editor.markSaved();
    updateTab(filePath);
    statusFilename.textContent = getFileName(filePath);
    terminal.writeSuccess(`Saved: ${filePath}`);
  }

  // ----- Build & Run -----

  async function buildFile() {
    const filePath = editor.getFilePath();
    if (!filePath) {
      terminal.writeStderr('No file to build. Save the file first.');
      return;
    }

    // Auto-save before build
    if (editor.isModified) {
      await saveFile();
    }

    terminal.writeSystem(`Building: ${filePath}`);
    const cwd = currentWorkDir || getDirectory(filePath);

    const result = await window.electronAPI.runCommand(
      'unilang', ['build', filePath], cwd
    );

    if (result.success) {
      terminal.writeSuccess('Build completed successfully.');
    } else {
      terminal.writeStderr(`Build failed with exit code ${result.code}.`);
    }
  }

  async function runFile() {
    const filePath = editor.getFilePath();
    if (!filePath) {
      terminal.writeStderr('No file to run. Save the file first.');
      return;
    }

    // Auto-save before run
    if (editor.isModified) {
      await saveFile();
    }

    terminal.writeSystem(`Running: ${filePath}`);
    const cwd = currentWorkDir || getDirectory(filePath);

    const result = await window.electronAPI.runCommand(
      'unilang', ['run', filePath], cwd
    );

    if (result.success) {
      terminal.writeSuccess(`Process exited with code ${result.code}.`);
    } else {
      terminal.writeStderr(`Process exited with code ${result.code}.`);
    }
  }

  // ----- Tab Management -----

  function updateTab(filePath) {
    // Remove placeholder
    if (tabPlaceholder) tabPlaceholder.style.display = 'none';

    // Remove existing tabs
    const existing = editorTabs.querySelectorAll('.editor-tab');
    existing.forEach(el => el.remove());

    if (!filePath) {
      // Show "Untitled" tab
      const tab = createTabElement('Untitled', null);
      editorTabs.appendChild(tab);
      return;
    }

    const tab = createTabElement(getFileName(filePath), filePath);
    editorTabs.appendChild(tab);
  }

  function createTabElement(label, filePath) {
    const tab = document.createElement('div');
    tab.className = 'editor-tab active';

    const modDot = document.createElement('span');
    modDot.className = 'tab-modified';
    tab.appendChild(modDot);

    const labelSpan = document.createElement('span');
    labelSpan.textContent = label;
    tab.appendChild(labelSpan);

    const closeBtn = document.createElement('span');
    closeBtn.className = 'tab-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      newFile();
    });
    tab.appendChild(closeBtn);

    return tab;
  }

  // ----- Resize Handles -----

  function initResize() {
    // Sidebar horizontal resize
    let sidebarDragging = false;
    sidebarResize.addEventListener('mousedown', (e) => {
      sidebarDragging = true;
      sidebarResize.classList.add('active');
      e.preventDefault();
    });

    // Terminal vertical resize
    let terminalDragging = false;
    terminalResize.addEventListener('mousedown', (e) => {
      terminalDragging = true;
      terminalResize.classList.add('active');
      e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
      if (sidebarDragging) {
        const newWidth = Math.max(150, Math.min(500, e.clientX));
        sidebarPanel.style.width = newWidth + 'px';
      }
      if (terminalDragging) {
        const containerRect = terminalPanel.parentElement.getBoundingClientRect();
        const newHeight = Math.max(80, containerRect.bottom - e.clientY);
        terminalPanel.style.height = newHeight + 'px';
      }
    });

    document.addEventListener('mouseup', () => {
      if (sidebarDragging) {
        sidebarDragging = false;
        sidebarResize.classList.remove('active');
      }
      if (terminalDragging) {
        terminalDragging = false;
        terminalResize.classList.remove('active');
      }
    });
  }

  // ----- Keyboard Shortcuts -----

  function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      const isMod = e.metaKey || e.ctrlKey;

      if (isMod && e.key === 's' && !e.shiftKey) {
        e.preventDefault();
        saveFile();
      } else if (isMod && e.key === 's' && e.shiftKey) {
        e.preventDefault();
        saveFileAs();
      } else if (isMod && e.key === 'b') {
        e.preventDefault();
        buildFile();
      } else if (isMod && e.key === 'r') {
        e.preventDefault();
        runFile();
      } else if (isMod && e.key === 'n') {
        e.preventDefault();
        newFile();
      } else if (isMod && e.key === 'o' && !e.shiftKey) {
        e.preventDefault();
        openFile();
      } else if (isMod && e.key === 'o' && e.shiftKey) {
        e.preventDefault();
        openFolder();
      }
    });
  }

  // ----- IPC Event Listeners -----

  function initIPCListeners() {
    // Command output streaming
    window.electronAPI.onCommandStdout((data) => terminal.writeStdout(data));
    window.electronAPI.onCommandStderr((data) => terminal.writeStderr(data));

    if (window.electronAPI.onUnilangCliStatus) {
      window.electronAPI.onUnilangCliStatus((info) => {
        const label = info.usesCustomPath ? '(saved path)' : '(auto-detected)';
        terminal.writeSystem(`UniLang CLI ${label}: ${info.resolvedPath}`);
      });
    }

    // Menu events
    window.electronAPI.onMenuNewFile(() => newFile());
    window.electronAPI.onMenuSave(() => saveFile());
    window.electronAPI.onMenuSaveAs(() => saveFileAs());
    window.electronAPI.onMenuBuild(() => buildFile());
    window.electronAPI.onMenuRun(() => runFile());
    window.electronAPI.onFileOpened((filePath) => openFile(filePath));
    window.electronAPI.onFolderOpened((folderPath) => openFolder(folderPath));
  }

  // ----- Toolbar Buttons -----

  function initToolbar() {
    btnNew.addEventListener('click', () => newFile());
    btnOpen.addEventListener('click', () => openFile());
    btnOpenFolder.addEventListener('click', () => openFolder());
    btnSave.addEventListener('click', () => saveFile());
    btnBuild.addEventListener('click', () => buildFile());
    btnRun.addEventListener('click', () => runFile());
  }

  // ----- File Tree Events -----

  function initFileTree() {
    fileTree.onFileSelect(async (filePath) => {
      await openFile(filePath);
    });
  }

  // ----- Utility -----

  function getFileName(filePath) {
    if (!filePath) return 'Untitled';
    return filePath.split(/[\\/]/).pop();
  }

  function getDirectory(filePath) {
    if (!filePath) return null;
    const parts = filePath.split(/[\\/]/);
    parts.pop();
    return parts.join('/');
  }

  // ----- Initialize Everything -----

  async function logUnilangCliOnStartup() {
    try {
      if (window.electronAPI.getUnilangCliConfig) {
        const info = await window.electronAPI.getUnilangCliConfig();
        const label = info.usesCustomPath ? '(saved path)' : '(auto-detected)';
        terminal.writeSystem(`UniLang CLI ${label}: ${info.resolvedPath}`);
      }
    } catch (_e) {
      /* ignore */
    }
  }

  function init() {
    initToolbar();
    initFileTree();
    initResize();
    initKeyboardShortcuts();
    initIPCListeners();
    terminal.writeSystem('UniLang IDE v0.1.0 ready.');
    logUnilangCliOnStartup();
    editor.focus();
  }

  // Wait for DOM
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
