import { parse, converter, formatHex, differenceEuclidean } from 'culori';

const toOklab = converter('oklab');
const toOklch = converter('oklch');
const diffOklab = differenceEuclidean('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const anchors = { light: '#FFFFFF', base: '#787880', dark: '#101012' };
const a0 = toOklab(parse(anchors.light));
const a6 = toOklab(parse(anchors.base));
const a12 = toOklab(parse(anchors.dark));
const baseLch = toOklch(parse(anchors.base));
const darkLch = toOklch(parse(anchors.dark));

function score(steps) {
  let sum = 0;
  for (let i = 0; i <= 12; i++) {
    sum += diffOklab(parse('#' + figma[i]), parse(steps[i]));
  }
  return sum;
}

// --- Baseline: L via ease-in/out p=1.53, a/b linear
function baseline() {
  const p = 1.53;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    const t = i / 12;
    if (i < 6) {
      const e = Math.pow(i / 6, p);
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*e, a: a0.a + (a6.a-a0.a)*(i/6), b: a0.b + (a6.b-a0.b)*(i/6), alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const e = 1 - Math.pow(1 - (i-6)/6, p);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*e, a: a6.a + (a12.a-a6.a)*((i-6)/6), b: a6.b + (a12.b-a6.b)*((i-6)/6), alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Family 1: Exponential approach for a/b (light half)
// f(t) = (1 - exp(-k*t)) / (1 - exp(-k))
function family1(k) {
  const p = 1.53;
  const denom = 1 - Math.exp(-k);
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      const eL = Math.pow(t, p);
      const eAB = (1 - Math.exp(-k * t)) / denom;
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*eL, a: a0.a + (a6.a-a0.a)*eAB, b: a0.b + (a6.b-a0.b)*eAB, alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const t = (i - 6) / 6;
      const eL = 1 - Math.pow(1 - t, p);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*eL, a: a6.a + (a12.a-a6.a)*t, b: a6.b + (a12.b-a6.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Family 2: Gaussian bump (multiplicative) for a/b light half
// f(t) = t^p * (1 + A * exp(-((t-t0)/s)^2)) / norm
function family2(p, A, t0, s) {
  const norm = (1 + A * Math.exp(-Math.pow((1 - t0)/s, 2)));
  const bump = (t) => Math.pow(t, p) * (1 + A * Math.exp(-Math.pow((t - t0)/s, 2))) / norm;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      const eL = Math.pow(t, 1.53);
      const eAB = bump(t);
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*eL, a: a0.a + (a6.a-a0.a)*eAB, b: a0.b + (a6.b-a0.b)*eAB, alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const t = (i - 6) / 6;
      const eL = 1 - Math.pow(1 - t, 1.53);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*eL, a: a6.a + (a12.a-a6.a)*t, b: a6.b + (a12.b-a6.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Family 3: OKLCH interpolation with bell C curve
// L: ease-in/out, H: fixed, C: bell-shaped
function family3(CpeakRatio, tPeak, sigma) {
  const p = 1.53;
  const Cpeak = baseLch.c * CpeakRatio;
  const Cdark = darkLch.c;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    let L;
    if (i <= 6) {
      L = a0.l + (a6.l - a0.l) * Math.pow(t / 0.5, p);
    } else {
      L = a6.l + (a12.l - a6.l) * (1 - Math.pow(1 - (t - 0.5) / 0.5, p));
    }
    if (i === 6) L = a6.l;
    // Bell C curve: peak at tPeak, base at t=0.5, Cdark at t=1
    let C;
    if (t <= 0.5) {
      const u = t / 0.5;
      const bell = Cpeak * Math.exp(-Math.pow((t - tPeak) / sigma, 2));
      C = baseLch.c * u + bell * (1 - u);
    } else {
      const u = (t - 0.5) / 0.5;
      C = baseLch.c * (1 - u) + Cdark * u;
    }
    const lch = { mode: 'oklch', l: L, c: Math.max(0, C), h: baseLch.h, alpha: 1 };
    const lab = toOklab(lch);
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Family 4: Separate power for a and b (light half)
function family4(pa, pb) {
  const p = 1.53;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    let lab;
    if (i < 6) {
      const t = i / 6;
      const eL = Math.pow(t, p);
      lab = { mode:'oklab', l: a0.l + (a6.l-a0.l)*eL, a: a0.a + (a6.a-a0.a)*Math.pow(t,pa), b: a0.b + (a6.b-a0.b)*Math.pow(t,pb), alpha:1 };
    } else if (i === 6) {
      lab = a6;
    } else {
      const t = (i - 6) / 6;
      const eL = 1 - Math.pow(1 - t, p);
      lab = { mode:'oklab', l: a6.l + (a12.l-a6.l)*eL, a: a6.a + (a12.a-a6.a)*t, b: a6.b + (a12.b-a6.b)*t, alpha:1 };
    }
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Family 5: Piecewise OKLCH with C = C_base * envelope(t)
// envelope: quick rise to 1+overshoot, then settle to 1, then fall to Cdark
function family5(overshoot, riseP, fallP) {
  const p = 1.53;
  const Cdark = darkLch.c;
  const steps = [];
  for (let i = 0; i <= 12; i++) {
    const t = i / 12;
    let L;
    if (i <= 6) {
      L = a0.l + (a6.l - a0.l) * Math.pow(t / 0.5, p);
    } else {
      L = a6.l + (a12.l - a6.l) * (1 - Math.pow(1 - (t - 0.5) / 0.5, p));
    }
    if (i === 6) L = a6.l;
    // C envelope
    let C;
    if (t <= 0.5) {
      const u = t / 0.5;
      // Rise quickly: overshoot then settle to base
      const env = (1 + overshoot) * Math.pow(u, riseP) / (1 + overshoot * Math.pow(u, riseP));
      C = baseLch.c * env;
    } else {
      const u = (t - 0.5) / 0.5;
      C = baseLch.c * (1 - u) + Cdark * u;
    }
    const lch = { mode: 'oklch', l: L, c: Math.max(0, C), h: baseLch.h, alpha: 1 };
    const lab = toOklab(lch);
    steps.push(formatHex(lab));
  }
  return steps;
}

// --- Grid search ---
const b = score(baseline());
console.log(`Baseline (power L, linear a/b): total dE = ${b.toFixed(4)}`);

let best1 = { k: 1, score: Infinity };
for (let k = 0.5; k <= 20; k += 0.1) {
  const s = score(family1(k));
  if (s < best1.score) best1 = { k, score: s };
}
console.log(`Family 1 (exp a/b, light half): k=${best1.k.toFixed(1)}, dE=${best1.score.toFixed(4)}`);

let best2 = { p: 1, A: 0, t0: 0.2, s: 0.1, score: Infinity };
for (let p = 0.2; p <= 2; p += 0.1) {
  for (let A = 0.1; A <= 3; A += 0.1) {
    for (let t0 = 0.1; t0 <= 0.4; t0 += 0.05) {
      for (let s = 0.05; s <= 0.3; s += 0.05) {
        const s2 = score(family2(p, A, t0, s));
        if (s2 < best2.score) best2 = { p, A, t0, s, score: s2 };
      }
    }
  }
}
console.log(`Family 2 (Gaussian bump): p=${best2.p.toFixed(1)} A=${best2.A.toFixed(1)} t0=${best2.t0.toFixed(2)} s=${best2.s.toFixed(2)}, dE=${best2.score.toFixed(4)}`);

let best3 = { CpeakRatio: 1, tPeak: 0.17, sigma: 0.1, score: Infinity };
for (let CpeakRatio = 1.0; CpeakRatio <= 1.5; CpeakRatio += 0.05) {
  for (let tPeak = 0.1; tPeak <= 0.3; tPeak += 0.02) {
    for (let sigma = 0.03; sigma <= 0.2; sigma += 0.02) {
      const s = score(family3(CpeakRatio, tPeak, sigma));
      if (s < best3.score) best3 = { CpeakRatio, tPeak, sigma, score: s };
    }
  }
}
console.log(`Family 3 (OKLCH bell C): peak=${best3.CpeakRatio.toFixed(2)} tPeak=${best3.tPeak.toFixed(2)} sigma=${best3.sigma.toFixed(2)}, dE=${best3.score.toFixed(4)}`);

let best4 = { pa: 1, pb: 1, score: Infinity };
for (let pa = 0.1; pa <= 3; pa += 0.1) {
  for (let pb = 0.1; pb <= 3; pb += 0.1) {
    const s = score(family4(pa, pb));
    if (s < best4.score) best4 = { pa, pb, score: s };
  }
}
console.log(`Family 4 (sep power a/b): pa=${best4.pa.toFixed(1)} pb=${best4.pb.toFixed(1)}, dE=${best4.score.toFixed(4)}`);

let best5 = { overshoot: 0.1, riseP: 0.5, fallP: 1, score: Infinity };
for (let overshoot = 0.0; overshoot <= 0.5; overshoot += 0.05) {
  for (let riseP = 0.2; riseP <= 2; riseP += 0.1) {
    for (let fallP = 0.5; fallP <= 3; fallP += 0.1) {
      const s = score(family5(overshoot, riseP, fallP));
      if (s < best5.score) best5 = { overshoot, riseP, fallP, score: s };
    }
  }
}
console.log(`Family 5 (piecewise OKLCH C): ov=${best5.overshoot.toFixed(2)} riseP=${best5.riseP.toFixed(1)} fallP=${best5.fallP.toFixed(1)}, dE=${best5.score.toFixed(4)}`);
