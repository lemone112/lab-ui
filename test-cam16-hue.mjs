import { differenceEuclidean, parse as culoriParse, converter } from 'culori';
import { default as Color } from 'colorjs.io';

const diffOklab = differenceEuclidean('oklab');
const toHsv = converter('hsv');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

function toCam(hex) {
  return new Color(hex).to('cam16-jmh').coords;
}
function toHex([j, m, h]) {
  return new Color('cam16-jmh', [j, m, h]).to('srgb').toString({format: 'hex'});
}

const a0 = toCam('#FFFFFF');
const a6 = toCam('#787880');
const a12 = toCam('#101012');

// Hue ease-in (power < 1): fast jump to base hue
function camHueModel(p_J, p_M, p_H) {
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let J, M, H;
    if (i < 6) {
      const t = i / 6;
      J = a0[0] + (a6[0] - a0[0]) * Math.pow(t, p_J);
      M = a0[1] + (a6[1] - a0[1]) * Math.pow(t, p_M);
      H = a0[2] + (a6[2] - a0[2]) * Math.pow(t, p_H);
    } else if (i === 6) {
      J = a6[0]; M = a6[1]; H = a6[2];
    } else {
      const t = (i - 6) / 6;
      J = a6[0] + (a12[0] - a6[0]) * (1 - Math.pow(1 - t, p_J));
      M = a6[1] + (a12[1] - a6[1]) * (1 - Math.pow(1 - t, p_M));
      H = a6[2] + (a12[2] - a6[2]) * (1 - Math.pow(1 - t, p_H));
    }
    steps.push(toHex([J, M, H]));
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

let best = { p_J: 1, p_M: 1, p_H: 1, score: Infinity };
for (let p_J = 1.2; p_J <= 2.0; p_J += 0.1) {
  for (let p_M = 0.3; p_M <= 1.0; p_M += 0.1) {
    for (let p_H = 0.1; p_H <= 0.5; p_H += 0.05) {
      const s = score(camHueModel(p_J, p_M, p_H));
      if (s < best.score) best = { p_J, p_M, p_H, score: s };
    }
  }
}

console.log(`Best CAM16+Hue: p_J=${best.p_J.toFixed(1)} p_M=${best.p_M.toFixed(1)} p_H=${best.p_H.toFixed(2)} total dE=${best.score.toFixed(4)}`);

const steps = camHueModel(best.p_J, best.p_M, best.p_H);
console.log('\nstep | generated | figma     | dE     | HSB-S    | CAM-h    | note');
console.log('-----|-----------|-----------|--------|----------|----------|------');
for (let i = 0; i <= 12; i++) {
  const gen = steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(culoriParse('#' + fig), culoriParse(steps[i]));
  const hsv = toHsv(culoriParse(steps[i]));
  const cam = toCam(steps[i]);
  const note = i === 2 ? '<< light mid' : i === 8 ? '<< peak' : '';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)} | ${(hsv.s*100).toFixed(1)}%    | ${cam[2].toFixed(1)}   | ${note}`);
}

console.log('\nFigma HSB-S:  0:0.0% 1:1.2% 2:4.1% 3:4.2% 4:4.9% 5:5.8% 6:6.3% 7:8.2% 8:10.4% 9:7.7% 10:5.3% 11:6.7% 12:11.1%');

// HTML preview
const html = `<!DOCTYPE html>
<html><body style="background:#fff; padding:20px; font-family:sans-serif;">
<h3>Neutral Scale: Figma (top) vs CAM16+HueShift (bottom)</h3>
<div style="display:flex; gap:4px; margin-bottom:8px;">
  <span style="width:60px; font-size:12px; color:#666;">Figma</span>
  ${figma.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:#${c};"></div>`).join('')}
</div>
<div style="display:flex; gap:4px;">
  <span style="width:60px; font-size:12px; color:#666;">CAM16</span>
  ${steps.map(c => `<div style="width:48px;height:48px;border-radius:50%;background:${c};"></div>`).join('')}
</div>
</body></html>`;

await import('fs').then(fs => fs.promises.writeFile('preview-cam16-hue.html', html));
console.log('\nSaved preview-cam16-hue.html');
