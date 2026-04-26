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

function easeOut(t, p) { return 1 - Math.pow(1 - t, p); }
function easeIn(t, p)  { return Math.pow(t, p); }

function generate(pLow, pHigh) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i <= 6) {
      const t = i / 6;
      const e = easeOut(t, pLow);
      lab = {
        mode: 'oklab',
        l: a0.l + (a6.l - a0.l) * e,
        a: a0.a + (a6.a - a0.a) * t,
        b: a0.b + (a6.b - a0.b) * t,
        alpha: 1,
      };
    } else {
      const t = (i - 6) / 6;
      const e = easeIn(t, pHigh);
      lab = {
        mode: 'oklab',
        l: a6.l + (a12.l - a6.l) * e,
        a: a6.a + (a12.a - a6.a) * t,
        b: a6.b + (a12.b - a6.b) * t,
        alpha: 1,
      };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

function score(pLow, pHigh) {
  const steps = generate(pLow, pHigh);
  let sumDE = 0;
  for (let i = 0; i <= 12; i++) {
    sumDE += diffOklab(parse('#' + figma[i]), parse(steps[i]));
  }
  return sumDE;
}

let best = { pLow: 1, pHigh: 1, score: Infinity };
for (let pLow = 1.0; pLow <= 4.0; pLow += 0.01) {
  for (let pHigh = 1.0; pHigh <= 4.0; pHigh += 0.01) {
    const s = score(pLow, pHigh);
    if (s < best.score) best = { pLow, pHigh, score: s };
  }
}

console.log(`Best pLow=${best.pLow.toFixed(2)} pHigh=${best.pHigh.toFixed(2)} total dE=${best.score.toFixed(4)}`);

const steps = generate(best.pLow, best.pHigh);
console.log('\nstep | generated | figma     | dE');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i])).toFixed(4);
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d}`);
}
