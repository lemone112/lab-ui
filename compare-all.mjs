import { parse as culoriParse, converter } from 'culori';
import { default as Color } from 'colorjs.io';

const toHsv = converter('hsv');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

// 1. OKLab baseline
const { converter: cconv, parse: cparse, formatHex } = await import('culori');
const toOklab = cconv('oklab');
const a0o = toOklab(cparse('#FFFFFF'));
const a6o = toOklab(cparse('#787880'));
const a12o = toOklab(cparse('#101012'));

function baseline() {
  const p = 1.53;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      lab = { mode:'oklab', l: a0o.l + (a6o.l-a0o.l)*Math.pow(t,p), a: a0o.a + (a6o.a-a0o.a)*t, b: a0o.b + (a6o.b-a0o.b)*t, alpha:1 };
    } else if (i === 6) {
      lab = a6o;
    } else {
      const t = (i - 6) / 6;
      lab = { mode:'oklab', l: a6o.l + (a12o.l-a6o.l)*(1-Math.pow(1-t,p)), a: a6o.a + (a12o.a-a6o.a)*t, b: a6o.b + (a12o.b-a6o.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

// 2. HCT bell-curve
function hctModel() {
  const a0 = new Color('#FFFFFF').to('hct').coords;
  const a6 = new Color('#787880').to('hct').coords;
  const a12 = new Color('#101012').to('hct').coords;
  const C_peak = 7.5, t_peak = 0.45, sigma = 0.40, p_T = 1.6;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    const H = a6[0];
    let T;
    if (p_T === 1) {
      T = a0[2] + (a12[2] - a0[2]) * t;
    } else {
      T = t < 0.5
        ? a0[2] + (a6[2] - a0[2]) * Math.pow(2 * t, p_T)
        : a6[2] + (a12[2] - a6[2]) * (1 - Math.pow(2 - 2 * t, p_T));
      if (i === 6) T = a6[2];
    }
    const C = C_peak * Math.exp(-Math.pow((t - t_peak) / sigma, 2));
    steps.push(new Color('hct', [H, C, T]).to('srgb').toString({format: 'hex'}));
  }
  return steps;
}

// 3. CAM16-UCS sinusoid
function toUcs(hex) {
  const [J, M, h] = new Color(hex).to('cam16-jmh').coords;
  const Jp = 1.7 * J / (1 + 0.007 * J);
  const Mp = Math.log(1 + 0.0228 * M) / 0.0228;
  const hr = h * Math.PI / 180;
  return [Jp, Mp * Math.cos(hr), Mp * Math.sin(hr)];
}
function fromUcs([Jp, ap, bp]) {
  const J = Jp / (1.7 - 0.007 * Jp);
  const Mp = Math.sqrt(ap * ap + bp * bp);
  const M = (Math.exp(0.0228 * Mp) - 1) / 0.0228;
  const h = (Math.atan2(bp, ap) * 180 / Math.PI + 360) % 360;
  return new Color('cam16-jmh', [J, M, h]).to('srgb').toString({format: 'hex'});
}
function ucsModel() {
  const a0 = toUcs('#FFFFFF');
  const a6 = toUcs('#787880');
  const a12 = toUcs('#101012');
  const p_J = 1.7, t_peak = 0.50;
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
    const base_h = Math.atan2(a6[2], a6[1]);
    const ap = Mp * Math.cos(base_h);
    const bp = Mp * Math.sin(base_h);
    steps.push(fromUcs([Jp, ap, bp]));
  }
  return steps;
}

const b = baseline();
const h = hctModel();
const u = ucsModel();

console.log('step | Figma     | OKLab     | HCT       | CAM16-UCS | Figma-HSB | OKLab-HSB | HCT-HSB   | UCS-HSB');
console.log('-----|-----------|-----------|-----------|-----------|-----------|-----------|-----------|----------');
for (let i = 0; i <= 12; i++) {
  const fb = '#' + figma[i];
  const bb = b[i].toUpperCase();
  const hb = h[i].toUpperCase();
  const ub = u[i].toUpperCase();
  const fhsv = toHsv(cparse(fb));
  const bhsv = toHsv(cparse(bb));
  const hhsv = toHsv(cparse(hb));
  const uhsv = toHsv(cparse(ub));
  console.log(
    `  ${String(i).padStart(2,'0')} | ${figma[i]}    | ${bb.replace('#','')}    | ${hb.replace('#','')}    | ${ub.replace('#','')}    | ` +
    `${(fhsv.s*100).toFixed(1)}%     | ${(bhsv.s*100).toFixed(1)}%     | ${(hhsv.s*100).toFixed(1)}%     | ${(uhsv.s*100).toFixed(1)}%`
  );
}
