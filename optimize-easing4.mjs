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

function easeIn(t, p)  { return Math.pow(t, p); }
function easeOut(t, p) { return 1 - Math.pow(1 - t, p); }

function generate(pLowL, pHighL, pLowAB, pHighAB) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i <= 6) {
      const t = i / 6;
      const eL = easeIn(t, pLowL);
      const eAB = easeIn(t, pLowAB);
      lab = {
        mode: 'oklab',
        l: a0.l + (a6.l - a0.l) * eL,
        a: a0.a + (a6.a - a0.a) * eAB,
        b: a0.b + (a6.b - a0.b) * eAB,
        alpha: 1,
      };
    } else {
      const t = (i - 6) / 6;
      const eL = easeOut(t, pHighL);
      const eAB = easeOut(t, pHighAB);
      lab = {
        mode: 'oklab',
        l: a6.l + (a12.l - a6.l) * eL,
        a: a6.a + (a12.a - a6.a) * eAB,
        b: a6.b + (a12.b - a6.b) * eAB,
        alpha: 1,
      };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

function score(pLowL, pHighL, pLowAB, pHighAB) {
  const steps = generate(pLowL, pHighL, pLowAB, pHighAB);
  let sumDE = 0;
  for (let i = 0; i <= 12; i++) {
    sumDE += diffOklab(parse('#' + figma[i]), parse(steps[i]));
  }
  return sumDE;
}

let best = { pLowL: 1, pHighL: 1, pLowAB: 1, pHighAB: 1, score: Infinity };
for (let pLowL = 1.0; pLowL <= 3.0; pLowL += 0.05) {
  for (let pHighL = 1.0; pHighL <= 3.0; pHighL += 0.05) {
    for (let pLowAB = 1.0; pLowAB <= 3.0; pLowAB += 0.05) {
      for (let pHighAB = 1.0; pHighAB <= 3.0; pHighAB += 0.05) {
        const s = score(pLowL, pHighL, pLowAB, pHighAB);
        if (s < best.score) best = { pLowL, pHighL, pLowAB, pHighAB, score: s };
      }
    }
  }
}

console.log(`Best pLowL=${best.pLowL.toFixed(2)} pHighL=${best.pHighL.toFixed(2)} pLowAB=${best.pLowAB.toFixed(2)} pHighAB=${best.pHighAB.toFixed(2)} total dE=${best.score.toFixed(4)}`);

const steps = generate(best.pLowL, best.pHighL, best.pLowAB, best.pHighAB);
console.log('\nstep | generated | figma     | dE');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i])).toFixed(4);
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d}`);
}
