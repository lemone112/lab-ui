import { parse, converter } from 'culori';

// HSV is equivalent to HSB in most tools
const toHsv = converter('hsv');
const toOklab = converter('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const generated = [
  'FFFFFF','F6F6F7','E4E4E7','CECED3','B4B4BA','97979E','787880',
  '5C5C63','44444A','303035','212124','161618','101012',
];

function hsv(hex) {
  const c = parse(hex);
  return toHsv(c);
}

console.log('step | figma-H   | figma-S   | figma-V   | gen-S     | gen-V     | S-ratio');
console.log('-----|-----------|-----------|-----------|-----------|-----------|--------');
for (let i = 0; i <= 12; i++) {
  const f = hsv('#' + figma[i]);
  const g = hsv('#' + generated[i]);
  const ratio = f.s / g.s;
  console.log(
    `  ${String(i).padStart(2,'0')} | ${f.h?.toFixed(1)??'--'} | ${f.s.toFixed(4)} | ${f.v.toFixed(4)} | ${g.s.toFixed(4)} | ${g.v.toFixed(4)} | ${isFinite(ratio)?ratio.toFixed(2):'inf'}`
  );
}

// Delta analysis
console.log('\n--- Figma OKLab a/b curve (normalized to base) ---');
const base = toOklab(parse('#787880'));
console.log('step | a_norm    | b_norm    | C_norm    | H');
for (let i = 0; i <= 12; i++) {
  const lab = toOklab(parse('#' + figma[i]));
  const a_n = lab.a / base.a;
  const b_n = lab.b / base.b;
  const c = Math.sqrt(lab.a*lab.a + lab.b*lab.b);
  const c_base = Math.sqrt(base.a*base.a + base.b*base.b);
  const c_n = c / c_base;
  const h = (Math.atan2(lab.b, lab.a) * 180 / Math.PI + 360) % 360;
  console.log(`  ${String(i).padStart(2,'0')} | ${a_n.toFixed(3)} | ${b_n.toFixed(3)} | ${c_n.toFixed(3)} | ${h.toFixed(1)}`);
}
