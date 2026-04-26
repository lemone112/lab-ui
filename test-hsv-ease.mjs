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

// Interpolate in HSV: H fixed, S via ease-in, V via ease-in-out
function hsvModel(S_peak, p_S, p_V) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    let h, s, v;
    
    h = h6.h; // fixed hue
    
    // S: ease-in to peak at step 8, then falloff
    if (i <= 8) {
      const u = i / 8;
      s = S_peak * Math.pow(u, p_S);
    } else {
      const u = (i - 8) / 4;
      s = S_peak * (1 - u) + h12.s * u;
    }
    
    // V: ease-in-out via power
    if (i <= 6) {
      const u = i / 6;
      v = h0.v + (h6.v - h0.v) * Math.pow(u, p_V);
    } else {
      const u = (i - 6) / 6;
      v = h6.v + (h12.v - h6.v) * (1 - Math.pow(1 - u, p_V));
    }
    if (i === 6) v = h6.v;
    
    const hsv = { mode: 'hsv', h, s, v, alpha: 1 };
    steps.push(formatHex(hsv));
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

let best = { S_peak: 0.1, p_S: 0.5, p_V: 1.5, score: Infinity };
for (let S_peak = 0.08; S_peak <= 0.15; S_peak += 0.005) {
  for (let p_S = 0.2; p_S <= 0.8; p_S += 0.05) {
    for (let p_V = 1.0; p_V <= 2.0; p_V += 0.1) {
      const s = score(hsvModel(S_peak, p_S, p_V));
      if (s < best.score) best = { S_peak, p_S, p_V, score: s };
    }
  }
}

console.log(`Best HSV model: S_peak=${best.S_peak.toFixed(3)} p_S=${best.p_S.toFixed(2)} p_V=${best.p_V.toFixed(1)} total dE=${best.score.toFixed(4)}`);

const steps = hsvModel(best.S_peak, best.p_S, best.p_V);
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
<h3>Neutral Scale: Figma (top) vs HSV EaseIn (bottom)</h3>
<div style="display:flex; gap:4px; margin-bottom:8px;">
  <span style="width:60px; font-size:12px; color:#666;">Figma</span>
  ${figma.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:#${c};"></div>`).join('')}
</div>
<div style="display:flex; gap:4px;">
  <span style="width:60px; font-size:12px; color:#666;">HSV</span>
  ${steps.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:${c};"></div>`).join('')}
</div>
</body></html>`;

await import('fs').then(fs => fs.promises.writeFile('preview-hsv.html', html));
console.log('\nSaved preview-hsv.html');
