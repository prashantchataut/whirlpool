// Reads docs/capture-matrix.json. Two modes:
//   node scripts/check-matrix.mjs                     static: the matrix and scripts/drive.ps1 agree
//   node scripts/check-matrix.mjs --verify DIR [--scale N]   after a run: every expected capture exists
import { readFileSync, existsSync } from 'node:fs';

const matrix = JSON.parse(readFileSync('docs/capture-matrix.json', 'utf8'));
const drive = readFileSync('scripts/drive.ps1', 'utf8');
const errors = [];
const args = process.argv.slice(2);
const verifyAt = args.indexOf('--verify');

// A driver name such as "settings $page" or "20-start-$d" becomes a pattern; matrix names match it.
const pattern = (name) => new RegExp('^' + name.replace(/[.*+?^{}()|[\]\\]/g, '\\$&').replace(/\$\\\(.*?\\\)|\$\w+/g, '.+') + '$');
const need = (list, want, what, id) => {
  for (const w of want) if (!list.includes(w)) errors.push(`${id}: missing ${what} ${w}`);
};

if (verifyAt < 0) {
  const scenarios = [...drive.matchAll(/Scenario\s+["']([^"']+)["']/g)].map((m) => m[1]);
  const shotNames = [...drive.matchAll(/Shot(?:-Panel)?\s+["']([^"']+)["']/g)].map((m) => m[1]);
  const known = new Set();
  for (const s of matrix.screens) {
    need(matrix.runners, ['windows-latest', 'windows-11-arm'], 'runner', 'matrix');
    need(matrix.scales, [100, 150, 175], 'scale', 'matrix');
    if (!scenarios.some((n) => n.includes(s.scenario) || pattern(n).test(s.scenario) || new RegExp(n.replace(/\$\w+/g, '.+')).test(s.scenario))) errors.push(`${s.id}: no Scenario in drive.ps1 contains "${s.scenario}"`);
    for (const shot of s.shots) {
      known.add(shot);
      if (!shotNames.some((n) => n === shot || pattern(n).test(shot))) errors.push(`${s.id}: drive.ps1 never writes ${shot}`);
    }
  }
  for (const n of matrix.panelShotsOutsideTheMatrix.names) known.add(n);
  const used = [...drive.matchAll(/Shot-Panel\s+["']([^"']+)["']/g)].map((m) => m[1].split('$')[0]);
  for (const u of used) {
    if (![...known].some((k) => k === u || k.startsWith(u) || u.startsWith(k))) {
      errors.push(`drive.ps1 captures a panel as "${u}" that is not in docs/capture-matrix.json`);
    }
  }
  console.log(`matrix: ${matrix.screens.length} screens x ${matrix.scales.length} scales x ${matrix.runners.length} runners = ${matrix.screens.length * matrix.scales.length * matrix.runners.length} captures required`);
} else {
  const dir = args[verifyAt + 1];
  const scale = args.includes('--scale') ? args[args.indexOf('--scale') + 1] : '100';
  for (const s of matrix.screens) {
    for (const shot of s.shots) {
      const f = `${dir}/${shot}.png`;
      if (!existsSync(f)) errors.push(`missing capture for ${s.id} at ${scale}%: ${f}`);
    }
  }
}
if (errors.length) {
  console.error(errors.map((e) => `  - ${e}`).join('\n'));
  process.exit(1);
}
console.log('matrix OK');
