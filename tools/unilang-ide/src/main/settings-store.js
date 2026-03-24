/**
 * Persisted IDE settings (userData directory).
 * Must call warmup() from app.whenReady() before relying on disk-backed values.
 */

const fs = require('fs');
const path = require('path');
const { app } = require('electron');

/** @type {{ unilangCliPath: string | null }} */
let data = { unilangCliPath: null };

let warmedUp = false;

function settingsFilePath() {
  return path.join(app.getPath('userData'), 'ide-settings.json');
}

function warmup() {
  if (warmedUp) return;
  warmedUp = true;
  try {
    const fp = settingsFilePath();
    if (fs.existsSync(fp)) {
      const parsed = JSON.parse(fs.readFileSync(fp, 'utf8'));
      if (typeof parsed.unilangCliPath === 'string' || parsed.unilangCliPath === null) {
        data.unilangCliPath = parsed.unilangCliPath;
      }
    }
  } catch (e) {
    console.warn('UniLang IDE: failed to load settings:', e.message);
  }
}

/** @returns {string | null} */
function getUnilangCliPath() {
  if (!app.isReady()) return null;
  warmup();
  return data.unilangCliPath;
}

/** @param {string | null} p absolute path to binary, or null to clear */
function setUnilangCliPath(p) {
  warmup();
  data.unilangCliPath = p && p.length > 0 ? p : null;
  const fp = settingsFilePath();
  fs.mkdirSync(path.dirname(fp), { recursive: true });
  fs.writeFileSync(fp, JSON.stringify(data, null, 2), 'utf8');
}

module.exports = {
  warmup,
  getUnilangCliPath,
  setUnilangCliPath
};
