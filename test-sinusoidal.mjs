import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toOklab = converter('oklab');
const toOklch = converter('oklch');
const toHsv = converter('hsv');
const diffOklab = differenceEuclidean('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const anchors = { light: '#FFFFFF', base: '#787880', dark: '#101012' };
const a0 = toOklab(parse(anchors.light));
const a6 = toOklab(parse(anchors.base));
const a12 = toOklab(parse(anchors.dark));
const baseLch = toOklch(parse(anchors.base));
const darkLch = toOklch(parse(anchors.dark));

// L via ease-in/out (p=1.53)
function L_curve(i) {
  const p = 1.53;
  if (i <= 6) {
    return a0.l + (a6.l - a0.l) * Math.pow(i / 6, p);
  } else {
    const t = (i - 6) / 6;
    return a6.l + (a12.l - a6.l) * (1 - Math.pow(1 - t, p));
  }
}

// Sinusoidal bell model: C rises as sin^2 from 0 to peak at step 8, then falls to C_dark at step 12
function sinusoidalModel(C_peak) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const L = (i === 6) ? a6.l : L_curve(i);
    let C;
    if (i <= 8) {
      const u = i / 16; // 0..0.5 for i=0..8
      C = C_peak * Math.pow(Math.sin(Math.PI * u), 2);
    } else {
      const u = (i - 8) / 4; // 0..1 for i=8..12
      C = C_peak * (1 - u) + darkLch.c * u;
    }
    const lch = { mode: 'oklch', l: L, c: Math.max(0, C), h: baseLch.h, alpha: 1 };
    const lab = toOklab(lch);
    steps.push(formatHex(lab));
  }
  return steps;
}

function score(steps) {
  let sum = 0;
  for (let i = 0; i <= 12; i++) {
    sum += diffOklab(parse('#' + figma[i]), parse(steps[i]));
  }
  return sum;
}

// Grid search C_peak
let best = { C_peak: baseLch.c, score: Infinity };
for (let C_peak = 0.005; C_peak <= 0.05; C_peak += 0.001) {
  const s = score(sinusoidalModel(C_peak));
  if (s < best.score) best = { C_peak, score: s };
}

console.log(`Best sinusoidal model: C_peak=${best.C_peak.toFixed(4)} total dE=${best.score.toFixed(4)}`);

const steps = sinusoidalModel(best.C_peak);
console.log('\nstep | generated | figma     | dE     | HSB-S    | note');
console.log('-----|-----------|-----------|--------|----------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i]));
  const hsv = toHsv(parse(steps[i]));
  const note = i === 8 ? '<< peak' : i === 12 ? '<< dark edge' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${(hsv.s*100).toFixed(1)}%    | ${note}`);
}

// Also show Figma HSB-S for comparison
console.log('\nFigma HSB-S for comparison:');
for (let i = 0; i <= 12; i++) {
  const hsv = toHsv(parse('#' + figma[i]));
  console.log(`  ${String(i).padStart(2,'0')}: ${(hsv.s*100).toFixed(1)}%`);
}
