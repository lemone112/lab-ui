import { parse, converter } from 'culori';

const toOklab = converter('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const linear = [
  'FFFFFF','E7E7E9','D0D0D3','B9B9BE','A3A3A9','8D8D94','787880',
  '65656C','525258','404045','2F2F33','1F1F22','101012',
];

console.log('step | figma-L   | linear-L  | diff-L   | a         | b');
console.log('-----|-----------|-----------|----------|-----------|----------');
for (let i = 0; i <= 12; i++) {
  const f = toOklab(parse('#' + figma[i]));
  const l = toOklab(parse('#' + linear[i]));
  console.log(
    `  ${String(i).padStart(2,'0')} | ${f.l.toFixed(6)} | ${l.l.toFixed(6)} | ${(f.l - l.l).toFixed(6)} | ${f.a.toFixed(6)} | ${f.b.toFixed(6)}`
  );
}

console.log('\n--- Normalized t (step/12) vs figma-L ---');
console.log('t    | L');
for (let i = 0; i <= 12; i++) {
  const f = toOklab(parse('#' + figma[i]));
  console.log(`${(i/12).toFixed(4)} | ${f.l.toFixed(6)}`);
}
