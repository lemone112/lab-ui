import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toOklab = converter('oklab');
const toOklch = converter('oklch');
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

// L via ease-in/out (fixed p=1.53)
function L_curve(i) {
  const p = 1.53;
  if (i <= 6) {
    return a0.l + (a6.l - a0.l) * Math.pow(i / 6, p);
  } else {
    const t = (i - 6) / 6;
    return a6.l + (a12.l - a6.l) * (1 - Math.pow(1 - t, p));
  }
}

// Piecewise chroma curve for light half
function chromaCurve(t, k, p_rise) {
  const tSwitch = 2 / 6; // step 2 boundary
  const denom = 1 - Math.exp(-k);
  const Cswitch = (1 - Math.exp(-k * tSwitch)) / denom;
  if (t <= tSwitch) {
    return (1 - Math.exp(-k * t)) / denom;
  } else {
    const u = (t - tSwitch) / (1 - tSwitch);
    return Cswitch + (1 - Cswitch) * (1 - Math.pow(1 - u, p_rise));
  }
}

function generate(k, p_rise) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const L = (i === 6) ? a6.l : L_curve(i);
    let a_val, b_val;
    if (i <= 6) {
      const t = i / 6;
      const c = chromaCurve(t, k, p_rise);
      a_val = baseLch.c * c * Math.cos(baseLch.h * Math.PI / 180);
      b_val = baseLch.c * c * Math.sin(baseLch.h * Math.PI / 180);
    } else {
      const t = (i - 6) / 6;
      a_val = a6.a + (a12.a - a6.a) * t;
      b_val = a6.b + (a12.b - a6.b) * t;
    }
    const lab = { mode: 'oklab', l: L, a: a_val, b: b_val, alpha: 1 };
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

// Grid search
let best = { k: 1, p_rise: 1, score: Infinity };
for (let k = 0.5; k <= 10; k += 0.1) {
  for (let p_rise = 0.5; p_rise <= 3; p_rise += 0.1) {
    const s = score(generate(k, p_rise));
    if (s < best.score) best = { k, p_rise, score: s };
  }
}

console.log(`Best cleaned model: k=${best.k.toFixed(1)} p_rise=${best.p_rise.toFixed(1)} total dE=${best.score.toFixed(4)}`);

const steps = generate(best.k, best.p_rise);
console.log('\nstep | generated | figma     | dE     | note');
console.log('-----|-----------|-----------|--------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i]));
  const note = i === 2 ? '<< cleaned' : i === 8 ? '<< dark zone' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${note}`);
}

// HSB comparison for step 2
const toHsv = converter('hsv');
const f2 = toHsv(parse('#EBEBF5'));
const g2 = toHsv(parse(steps[2]));
console.log(`\nStep 2 HSB comparison:`);
console.log(`  Figma:  H=${f2.h?.toFixed(0)} S=${(f2.s*100).toFixed(1)}% V=${(f2.v*100).toFixed(1)}%`);
console.log(`  Cleaned: H=${g2.h?.toFixed(0)} S=${(g2.s*100).toFixed(1)}% V=${(g2.v*100).toFixed(1)}%`);
