import { differenceEuclidean, parse as culoriParse, converter } from 'culori';
import { default as Color } from 'colorjs.io';

const diffOklab = differenceEuclidean('oklab');
const toHsv = converter('hsv');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

function toHct(hex) {
  return new Color(hex).to('hct').coords;
}

function toHex([h, c, t]) {
  const col = new Color('hct', [h, c, t]);
  return col.to('srgb').toString({format: 'hex'});
}

const a0 = toHct('#FFFFFF');
const a6 = toHct('#787880');
const a12 = toHct('#101012');

console.log('Anchors HCT:');
console.log('  light:', a0.map(v => v.toFixed(2)).join(', '));
console.log('  base: ', a6.map(v => v.toFixed(2)).join(', '));
console.log('  dark: ', a12.map(v => v.toFixed(2)).join(', '));

// HCT model: H fixed, T linear, C via bell curve
function hctModel(C_peak, t_peak, sigma, p_T) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    const H = a6[0]; // fixed base hue
    
    // Tone: ease-in-out for perceptual lightness
    let T;
    if (p_T === 1) {
      T = a0[2] + (a12[2] - a0[2]) * t;
    } else {
      // ease-in-out
      T = t < 0.5
        ? a0[2] + (a6[2] - a0[2]) * Math.pow(2 * t, p_T)
        : a6[2] + (a12[2] - a6[2]) * (1 - Math.pow(2 - 2 * t, p_T));
      if (i === 6) T = a6[2];
    }
    
    // Chroma: bell curve with peak
    const u = t;
    const C = C_peak * Math.exp(-Math.pow((u - t_peak) / sigma, 2));
    
    steps.push(toHex([H, C, T]));
  }
  return steps;
}

function score(steps) {
  let sum = 0;
  for (let i = 0; i <= 12; i++) {
    sum += diffOklab(culoriParse('#' + figma[i]), culoriParse(steps[i]));
  }
  return sum;
}

let best = { C_peak: 7, t_peak: 0.35, sigma: 0.3, p_T: 1, score: Infinity };
for (let C_peak = 2; C_peak <= 12; C_peak += 0.5) {
  for (let t_peak = 0.15; t_peak <= 0.5; t_peak += 0.05) {
    for (let sigma = 0.1; sigma <= 0.4; sigma += 0.05) {
      for (let p_T = 1; p_T <= 2; p_T += 0.3) {
        const s = score(hctModel(C_peak, t_peak, sigma, p_T));
        if (s < best.score) best = { C_peak, t_peak, sigma, p_T, score: s };
      }
    }
  }
}

console.log(`\nBest HCT: C_peak=${best.C_peak.toFixed(1)} t_peak=${best.t_peak.toFixed(2)} sigma=${best.sigma.toFixed(2)} p_T=${best.p_T.toFixed(1)} total dE=${best.score.toFixed(4)}`);

const steps = hctModel(best.C_peak, best.t_peak, best.sigma, best.p_T);
console.log('\nstep | generated | figma     | dE     | HSB-S    | HCT-C    | HCT-T   | note');
console.log('-----|-----------|-----------|--------|----------|----------|---------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(culoriParse('#' + fig), culoriParse(steps[i]));
  const hsv = toHsv(culoriParse(steps[i]));
  const hct = toHct(steps[i]);
  const note = i === 2 ? '<< light mid' : i === 8 ? '<< peak' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${(hsv.s*100).toFixed(1)}%    | ${hct[1].toFixed(2)}    | ${hct[2].toFixed(1)}   | ${note}`);
}

console.log('\nFigma HSB-S:  0:0.0% 1:1.2% 2:4.1% 3:4.2% 4:4.9% 5:5.8% 6:6.3% 7:8.2% 8:10.4% 9:7.7% 10:5.3% 11:6.7% 12:11.1%');

// HTML preview
const html = `<!DOCTYPE html>
<html><body style="background:#fff; padding:20px; font-family:sans-serif;">
<h3>Neutral Scale: Figma (top) vs HCT Bell-C (bottom)</h3>
<div style="display:flex; gap:4px; margin-bottom:8px;">
  <span style="width:60px; font-size:12px; color:#666;">Figma</span>
  ${figma.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:#${c};"></div>`).join('')}
</div>
<div style="display:flex; gap:4px;">
  <span style="width:60px; font-size:12px; color:#666;">HCT</span>
  ${steps.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:${c};"></div>`).join('')}
</div>
</body></html>`;

await import('fs').then(fs => fs.promises.writeFile('preview-hct.html', html));
console.log('\nSaved preview-hct.html');
