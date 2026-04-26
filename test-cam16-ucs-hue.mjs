import { parse as culoriParse, converter } from 'culori';
import { default as Color } from 'colorjs.io';

const toHsv = converter('hsv');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

function toUcs(hex) {
  const [J, M, h] = new Color(hex).to('cam16-jmh').coords;
  const Jp = 1.7 * J / (1 + 0.007 * J);
  const Mp = Math.log(1 + 0.0228 * M) / 0.0228;
  const hr = h * Math.PI / 180;
  return [Jp, Mp * Math.cos(hr), Mp * Math.sin(hr), h];
}

function fromUcs([Jp, ap, bp]) {
  const J = Jp / (1.7 - 0.007 * Jp);
  const Mp = Math.sqrt(ap * ap + bp * bp);
  const M = (Math.exp(0.0228 * Mp) - 1) / 0.0228;
  const h = (Math.atan2(bp, ap) * 180 / Math.PI + 360) % 360;
  return new Color('cam16-jmh', [J, M, h]).to('srgb').toString({format: 'hex'});
}

const a0 = toUcs('#FFFFFF');
const a6 = toUcs('#787880');
const a12 = toUcs('#101012');

console.log('Hue anchors:', {white: a0[3].toFixed(1), base: a6[3].toFixed(1), dark: a12[3].toFixed(1)});

// Interpolate hue from white→base (light half), base→dark (dark half)
function ucsHueModel(p_J, t_peak, p_H) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    let Jp;
    if (i <= 6) {
      Jp = a0[0] + (a6[0] - a0[0]) * Math.pow(t / 0.5, p_J);
    } else {
      Jp = a6[0] + (a12[0] - a6[0]) * (1 - Math.pow(1 - (t - 0.5) / 0.5, p_J));
    }
    if (i === 6) Jp = a6[0];
    
    const Mp_dark = Math.sqrt(a12[1] * a12[1] + a12[2] * a12[2]);
    const Mp_base = Math.sqrt(a6[1] * a6[1] + a6[2] * a6[2]);
    const Mp_peak = Mp_base * 1.2;
    let env;
    if (t <= t_peak) {
      env = Math.sin((Math.PI * t) / (2 * t_peak));
    } else {
      env = Math.sin((Math.PI * (1 - t)) / (2 * (1 - t_peak)));
    }
    const Mp = Mp_dark + (Mp_peak - Mp_dark) * env;
    
    // Hue interpolation
    let h_deg;
    if (i <= 6) {
      const u = i / 6;
      h_deg = a0[3] + (a6[3] - a0[3]) * Math.pow(u, p_H);
    } else {
      const u = (i - 6) / 6;
      h_deg = a6[3] + (a12[3] - a6[3]) * u;
    }
    if (i === 6) h_deg = a6[3];
    const hr = h_deg * Math.PI / 180;
    const ap = Mp * Math.cos(hr);
    const bp = Mp * Math.sin(hr);
    
    steps.push(fromUcs([Jp, ap, bp]));
  }
  return steps;
}

import { differenceEuclidean } from 'culori';
const diffOklab = differenceEuclidean('oklab');

function score(steps) {
  let sum = 0;
  for (let i = 0; i <= 12; i++) {
    sum += diffOklab(culoriParse('#' + figma[i]), culoriParse(steps[i]));
  }
  return sum;
}

let best = { p_J: 1, t_peak: 0.5, p_H: 1, score: Infinity };
for (let p_J = 1.0; p_J <= 2.5; p_J += 0.1) {
  for (let t_peak = 0.35; t_peak <= 0.7; t_peak += 0.05) {
    for (let p_H = 0.1; p_H <= 1.5; p_H += 0.1) {
      const s = score(ucsHueModel(p_J, t_peak, p_H));
      if (s < best.score) best = { p_J, t_peak, p_H, score: s };
    }
  }
}

console.log(`\nBest CAM16-UCS+Hue: p_J=${best.p_J.toFixed(1)} t_peak=${best.t_peak.toFixed(2)} p_H=${best.p_H.toFixed(1)} total dE=${best.score.toFixed(4)}`);

const steps = ucsHueModel(best.p_J, best.t_peak, best.p_H);
console.log('\nstep | generated | figma     | HSB-S    | HSB-H    | note');
console.log('-----|-----------|-----------|----------|----------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const hsv = toHsv(culoriParse(steps[i]));
  const note = i === 2 ? '<< light mid' : i === 8 ? '<< peak' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${(hsv.s*100).toFixed(1)}%    | ${hsv.h?.toFixed(0)??'--'}      | ${note}`);
}

console.log('\nFigma HSB:  S=0.0/1.2/4.1/4.2/4.9/5.8/6.3/8.2/10.4/7.7/5.3/6.7/11.1');
console.log('Figma HSB:  H=--/220/240/240/240/240/240/240/240/240/240/240/240');

// HTML preview
const html = `<!DOCTYPE html>
<html><body style="background:#fff; padding:20px; font-family:sans-serif;">
<h3>Neutral Scale: Figma (top) vs CAM16-UCS+Hue (bottom)</h3>
<div style="display:flex; gap:4px; margin-bottom:8px;">
  <span style="width:60px; font-size:12px; color:#666;">Figma</span>
  ${figma.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:#${c};"></div>`).join('')}
</div>
<div style="display:flex; gap:4px;">
  <span style="width:60px; font-size:12px; color:#666;">CAM16</span>
  ${steps.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:${c};"></div>`).join('')}
</div>
</body></html>`;

await import('fs').then(fs => fs.promises.writeFile('preview-cam16-ucs-hue.html', html));
console.log('\nSaved preview-cam16-ucs-hue.html');
