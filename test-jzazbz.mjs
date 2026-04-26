import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toJzazbz = converter('jzazbz');
const toHsv = converter('hsv');
const toOklab = converter('oklab');
const diffOklab = differenceEuclidean('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const anchors = { light: '#FFFFFF', base: '#787880', dark: '#101012' };
const a0 = toJzazbz(parse(anchors.light));
const a6 = toJzazbz(parse(anchors.base));
const a12 = toJzazbz(parse(anchors.dark));

// JzAzBz interpolation with ease-in L and different C curves
function jzModel(p_L, p_C) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let jz, az, bz;
    if (i < 6) {
      const t = i / 6;
      const eL = Math.pow(t, p_L);
      const eC = Math.pow(t, p_C);
      jz = a0.jz + (a6.jz - a0.jz) * eL;
      az = a0.az + (a6.az - a0.az) * eC;
      bz = a0.bz + (a6.bz - a0.bz) * eC;
    } else if (i === 6) {
      jz = a6.jz; az = a6.az; bz = a6.bz;
    } else {
      const t = (i - 6) / 6;
      const eL = 1 - Math.pow(1 - t, p_L);
      jz = a6.jz + (a12.jz - a6.jz) * eL;
      az = a6.az + (a12.az - a6.az) * t;
      bz = a6.bz + (a12.bz - a6.bz) * t;
    }
    const lab = { mode: 'jzazbz', jz, az, bz, alpha: 1 };
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

let best = { p_L: 1, p_C: 1, score: Infinity };
for (let p_L = 1.0; p_L <= 2.5; p_L += 0.1) {
  for (let p_C = 0.2; p_C <= 1.5; p_C += 0.1) {
    const s = score(jzModel(p_L, p_C));
    if (s < best.score) best = { p_L, p_C, score: s };
  }
}

console.log(`Best JzAzBz: p_L=${best.p_L.toFixed(1)} p_C=${best.p_C.toFixed(1)} total dE=${best.score.toFixed(4)}`);

const steps = jzModel(best.p_L, best.p_C);
console.log('\nstep | generated | figma     | dE     | HSB-S    | note');
console.log('-----|-----------|-----------|--------|----------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(steps[i]));
  const hsv = toHsv(parse(steps[i]));
  const note = i === 2 ? '<< light mid' : i === 8 ? '<< peak' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${(hsv.s*100).toFixed(1)}%    | ${note}`);
}

console.log('\nFigma HSB-S:  0:0.0% 1:1.2% 2:4.1% 3:4.2% 4:4.9% 5:5.8% 6:6.3% 7:8.2% 8:10.4% 9:7.7% 10:5.3% 11:6.7% 12:11.1%');

// HTML preview
const html = `<!DOCTYPE html>
<html><body style="background:#fff; padding:20px; font-family:sans-serif;">
<h3>Neutral Scale: Figma (top) vs JzAzBz (bottom)</h3>
<div style="display:flex; gap:4px; margin-bottom:8px;">
  <span style="width:60px; font-size:12px; color:#666;">Figma</span>
  ${figma.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:#${c};"></div>`).join('')}
</div>
<div style="display:flex; gap:4px;">
  <span style="width:60px; font-size:12px; color:#666;">JzAzBz</span>
  ${steps.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:${c};"></div>`).join('')}
</div>
</body></html>`;

await import('fs').then(fs => fs.promises.writeFile('preview-jzazbz.html', html));
console.log('\nSaved preview-jzazbz.html');
