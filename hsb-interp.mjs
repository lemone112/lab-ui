import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toHsv = converter('hsv');
const toOklab = converter('oklab');
const diffOklab = differenceEuclidean('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const anchors = { light: '#FFFFFF', base: '#787880', dark: '#101012' };
const h0 = toHsv(parse(anchors.light));
const h6 = toHsv(parse(anchors.base));
const h12 = toHsv(parse(anchors.dark));

console.log('Anchor HSV:', {h0, h6, h12});

// HSB interpolation: H fixed, S via ease, B via ease
function hsbModel(sPower, bPower) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    let h, s, v;
    if (i <= 6) {
      const u = i / 6;
      h = h6.h; // fixed hue
      s = h6.s * Math.pow(u, sPower);
      v = h0.v + (h6.v - h0.v) * Math.pow(u, bPower);
    } else {
      const u = (i - 6) / 6;
      h = h6.h;
      s = h6.s + (h12.s - h6.s) * (1 - Math.pow(1 - u, sPower));
      v = h6.v + (h12.v - h6.v) * (1 - Math.pow(1 - u, bPower));
    }
    const hsv = { mode: 'hsv', h, s, v, alpha: 1 };
    steps.push(formatHex(hsv));
  }
  return steps;
}

let best = { sP: 1, bP: 1, score: Infinity };
for (let sP = 0.1; sP <= 3; sP += 0.05) {
  for (let bP = 0.1; bP <= 3; bP += 0.05) {
    const steps = hsbModel(sP, bP);
    let sum = 0;
    for (let i = 0; i <= 12; i++) {
      sum += diffOklab(parse('#' + figma[i]), parse(steps[i]));
    }
    if (sum < best.score) best = { sP, bP, score: sum };
  }
}

console.log(`\nBest HSB model: S-power=${best.sP.toFixed(2)} B-power=${best.bP.toFixed(2)} total dE=${best.score.toFixed(4)}`);

const steps = hsbModel(best.sP, best.bP);
console.log('\nstep | generated | figma     | dE');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i])).toFixed(4);
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d}`);
}
