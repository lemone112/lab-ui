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

function easeInOut(t, p) {
  return t < 0.5
    ? Math.pow(2, p - 1) * Math.pow(t, p)
    : 1 - Math.pow(-2 * t + 2, p) / 2;
}

function generate(p) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    const e = easeInOut(t, p);
    // Interpolate L through eased curve; a/b through linear OKLab
    const lab = {
      mode: 'oklab',
      l: a0.l + (a12.l - a0.l) * e,
      a: a0.a + (a12.a - a0.a) * t,
      b: a0.b + (a12.b - a0.b) * t,
      alpha: 1,
    };
    steps.push(formatHex(lab));
  }
  return steps;
}

function score(p) {
  const steps = generate(p);
  let sumDE = 0;
  for (let i = 0; i <= 12; i++) {
    sumDE += diffOklab(parse('#' + figma[i]), parse(steps[i]));
  }
  return sumDE;
}

// Grid search
let best = { p: 1, score: Infinity };
for (let p = 1.0; p <= 5.0; p += 0.01) {
  const s = score(p);
  if (s < best.score) best = { p, score: s };
}

console.log(`Best power p=${best.p.toFixed(2)}, total dE=${best.score.toFixed(4)}`);

const steps = generate(best.p);
console.log('\nstep | generated | figma     | dE');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i])).toFixed(4);
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d}`);
}
