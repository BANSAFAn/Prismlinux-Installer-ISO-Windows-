const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

const args = process.argv.slice(2);
const help = args.includes('--help') || args.includes('-h');
const isDebug = args.includes('--debug') || args.includes('-d');
let targetPlatform = 'current';

for (let i = 0; i < args.length; i++) {
  if (args[i] === '--platform' || args[i] === '-p') {
    targetPlatform = args[i + 1];
  }
}

if (help) {
  console.log('Prism Linux Installer CLI Build Tool');
  console.log('Usage: node build.js [options]');
  console.log('');
  console.log('Options:');
  console.log('  -p, --platform <platform>  Target platform: windows, linux, all (default: current)');
  console.log('  -d, --debug                Build in debug mode instead of release');
  console.log('  -h, --help                 Show help options');
  process.exit(0);
}

const currentOS = os.platform();
let buildOS = '';
if (currentOS === 'win32') {
  buildOS = 'windows';
} else if (currentOS === 'linux') {
  buildOS = 'linux';
} else {
  buildOS = 'unknown';
}

if (targetPlatform !== 'current' && targetPlatform !== buildOS && targetPlatform !== 'all') {
  console.error(`Error: Cannot build for ${targetPlatform} on ${buildOS}. Cross-compilation is not natively supported by Tauri without virtual machines.`);
  process.exit(1);
}

console.log(`Starting Prism Linux Installer build...`);
console.log(`Current OS: ${buildOS}`);
console.log(`Mode: ${isDebug ? 'Debug' : 'Release'}`);

try {
  console.log('Installing project dependencies...');
  execSync('npm install', { stdio: 'inherit' });

  const buildCmd = isDebug 
    ? 'npx tauri build --debug'
    : 'npx tauri build';

  console.log(`Running build command: ${buildCmd}`);
  execSync(buildCmd, { stdio: 'inherit' });

  console.log('Build completed successfully!');

  const distFolder = path.join(__dirname, 'dist-installers');
  if (!fs.existsSync(distFolder)) {
    fs.mkdirSync(distFolder);
  }

  let srcDir = '';
  if (buildOS === 'windows') {
    srcDir = path.join(__dirname, 'src-tauri', 'target', isDebug ? 'debug' : 'release', 'bundle', 'msi');
    if (!fs.existsSync(srcDir)) {
      srcDir = path.join(__dirname, 'src-tauri', 'target', isDebug ? 'debug' : 'release', 'bundle', 'nsis');
    }
  } else if (buildOS === 'linux') {
    srcDir = path.join(__dirname, 'src-tauri', 'target', isDebug ? 'debug' : 'release', 'bundle', 'deb');
    if (!fs.existsSync(srcDir)) {
      srcDir = path.join(__dirname, 'src-tauri', 'target', isDebug ? 'debug' : 'release', 'bundle', 'appimage');
    }
  }

  if (fs.existsSync(srcDir)) {
    const files = fs.readdirSync(srcDir);
    for (const file of files) {
      const srcPath = path.join(srcDir, file);
      const destPath = path.join(distFolder, file);
      fs.copyFileSync(srcPath, destPath);
      console.log(`Copied build artifact to: ${destPath}`);
    }
  } else {
    console.log('Could not locate the build bundle directory. Please check src-tauri/target/ for output files.');
  }
} catch (error) {
  console.error('Build process encountered an error:', error.message);
  process.exit(1);
}
