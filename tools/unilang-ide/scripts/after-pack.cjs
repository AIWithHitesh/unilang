/**
 * Ad-hoc sign the .app after pack (macOS). Reduces some Gatekeeper friction;
 * does not replace Apple Developer ID + notarization for a fully trusted build.
 */
'use strict';

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

module.exports = async (context) => {
  if (process.platform !== 'darwin') return;

  const productName = context.packager.appInfo.productFilename;
  const appBundle = path.join(context.appOutDir, `${productName}.app`);
  if (!fs.existsSync(appBundle)) {
    console.warn('after-pack: .app not found at', appBundle);
    return;
  }

  try {
    execSync(`codesign --force --deep --sign - "${appBundle}"`, { stdio: 'inherit' });
    console.log('after-pack: ad-hoc signed', appBundle);
  } catch (e) {
    console.warn('after-pack: codesign failed (non-fatal):', e.message);
  }
};
