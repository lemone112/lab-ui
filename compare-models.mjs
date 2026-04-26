import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toOklab = converter('oklab');
const diffOklab = differenceEuclidean('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const anchors = { light: '#FFFFFF', base: '#787880', dark: '#101012' };
const a0 = toOklab(parse(anchors.light));
const a6 = toOklab(parse(anchors.base));
const a12 = toOklab(parse(anchors.dark));

function baseline() {
  const p = 1.53;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      const e = Math.pow(t, p);
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*e, a: a0.a + (a6.a-a0.a)*t, b: a0.b + (a6.b-a0.b)*t, alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const t = (i - 6) / 6;
      const e = 1 - Math.pow(1 - t, p);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*e, a: a6.a + (a12.a-a6.a)*t, b: a6.b + (a12.b-a6.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

function bestModel() {
  const p = 1.53;
  const A = 2.2, t0 = 0.40, s = 0.10, pa = 0.6;
  const norm = (1 + A * Math.exp(-Math.pow((1 - t0)/s, 2)));
  const bump = (t) => Math.pow(t, pa) * (1 + A * Math.exp(-Math.pow((t - t0)/s, 2))) / norm;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      const eL = Math.pow(t, p);
      const eAB = bump(t);
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*eL, a: a0.a + (a6.a-a0.a)*eAB, b: a0.b + (a6.b-a0.b)*eAB, alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const t = (i - 6) / 6;
      const eL = 1 - Math.pow(1 - t, p);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*eL, a: a6.a + (a12.a-a6.a)*t, b: a6.b + (a12.b-a6.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

const b = baseline();
const best = bestModel();

console.log('step | baseline-dE | best-dE     | improvement | figma');
console.log('-----|-------------|-------------|-------------|-------');
let baseTotal = 0, bestTotal = 0;
for (let i = 0; i <= 12; i++) {
  const db = diffOklab(parse('#' + figma[i]), parse(b[i]));
  const dBest = diffOklab(parse('#' + figma[i]), parse(best[i]));
  baseTotal += db;
  bestTotal += dBest;
  const imp = db - dBest;
  console.log(`  ${String(i).padStart(2,'0')} | ${db.toFixed(4)}      | ${dBest.toFixed(4)}      | ${imp>0?'+'+imp.toFixed(4):imp.toFixed(4)}      | ${figma[i]}`);
}
console.log(`\ntotal baseline: ${baseTotal.toFixed(4)}`);
console.log(`total best:     ${bestTotal.toFixed(4)}`);
console.log(`improvement:    ${(baseTotal - bestTotal).toFixed(4)} (${((baseTotal-bestTotal)/baseTotal*100).toFixed(1)}%)`);
