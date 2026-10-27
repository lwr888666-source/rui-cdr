/**
 * rui-ColeSDK - AI Integration & Node.js Tool
 * Allows AI Agents (Antigravity, etc.) to inspect and convert .cdr files.
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function findCliBinary() {
  const candidates = [
    path.resolve(__dirname, '../../bin/rui-cdr-cli.exe'),
    path.resolve(__dirname, '../../bin/rui-cdr-cli'),
    path.resolve(__dirname, '../../target/release/rui-cdr-cli.exe'),
    'rui-cdr-cli.exe',
    'rui-cdr-cli'
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) {
      return c;
    }
  }
  return 'rui-cdr-cli.exe';
}

function getCdrInfo(filePath) {
  const cli = findCliBinary();
  const res = spawnSync(cli, ['info', filePath], { encoding: 'utf8' });
  if (res.status !== 0) {
    return { error: (res.stderr || res.stdout || 'Failed to inspect file').trim() };
  }
  return {
    file: filePath,
    raw_info: res.stdout.trim()
  };
}

function convertCdr(inPath, outPath) {
  const cli = findCliBinary();
  const res = spawnSync(cli, ['convert', inPath, outPath], { encoding: 'utf8' });
  if (res.status !== 0) {
    return { success: false, error: (res.stderr || res.stdout || 'Conversion failed').trim() };
  }
  return {
    success: true,
    input: inPath,
    output: outPath,
    size_bytes: fs.existsSync(outPath) ? fs.statSync(outPath).size : 0
  };
}

const args = process.argv.slice(2);
if (args.length < 1) {
  console.log('Usage:');
  console.log('  node scripts/ai/cdr_ai_tool.js info <file.cdr>');
  console.log('  node scripts/ai/cdr_ai_tool.js convert <input.cdr> <output.png|svg|pdf>');
  process.exit(1);
}

const cmd = args[0].toLowerCase();
if (cmd === 'info' && args[1]) {
  console.log(JSON.stringify(getCdrInfo(args[1]), null, 2));
} else if (cmd === 'convert' && args[1] && args[2]) {
  console.log(JSON.stringify(convertCdr(args[1], args[2]), null, 2));
} else {
  console.log('Invalid arguments.');
}
