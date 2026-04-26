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

function L_curve(i) {
  const p = 1.53;
  if (i <= 6) {
    return a0.l + (a6.l - a0.l) * Math.pow(i / 6, p);
  } else {
    const t = (i - 6) / 6;
    return a6.l + (a12.l - a6.l) * (1 - Math.pow(1 - t, p));
  }
}

// Ease-in power curve: fast rise near white, peak at step 8, then linear falloff
function powerBellModel(C_peak, p_rise) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const L = (i === 6) ? a6.l : L_curve(i);
    let C;
    if (i <= 8) {
      const u = i / 8;
      C = C_peak * Math.pow(u, p_rise);
    } else {
      const u = (i - 8) / 4;
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

let best = { C_peak: 0.01, p_rise: 0.5, score: Infinity };
for (let C_peak = 0.008; C_peak <= 0.03; C_peak += 0.001) {
  for (let p_rise = 0.1; p_rise <= 1.0; p_rise += 0.05) {
    const s = score(powerBellModel(C_peak, p_rise));
    if (s < best.score) best = { C_peak, p_rise, score: s };
  }
}

console.log(`Best power-bell: C_peak=${best.C_peak.toFixed(4)} p_rise=${best.p_rise.toFixed(2)} total dE=${best.score.toFixed(4)}`);

const steps = powerBellModel(best.C_peak, best.p_rise);
console.log('\nstep | generated | figma     | dE     | HSB-S    | note');
console.log('-----|-----------|-----------|--------|----------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i]));
  const hsv = toHsv(parse(steps[i]));
  const note = i === 8 ? '<< peak' : i === 2 ? '<< light mid' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${(hsv.s*100).toFixed(1)}%    | ${note}`);
}

console.log('\nFigma HSB-S:  0:0.0% 1:1.2% 2:4.1% 3:4.2% 4:4.9% 5:5.8% 6:6.3% 7:8.2% 8:10.4% 9:7.7% 10:5.3% 11:6.7% 12:11.1%');
