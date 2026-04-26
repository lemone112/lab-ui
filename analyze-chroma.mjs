import { parse, converter } from 'culori';

const toOklab = converter('oklab');
const toOklch = converter('oklch');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const linear = [
  'FFFFFF','F6F6F7','E4E4E7','CECED3','B4B4BA','97979E','787880',
  '5C5C63','44444A','303035','212124','161618','101012',
];

console.log('step | figma-L   | gen-L     | figma-C   | gen-C     | figma-H   | gen-H');
console.log('-----|-----------|-----------|-----------|-----------|-----------|--------');
for (let i = 0; i <= 12; i++) {
  const f = toOklch(toOklab(parse('#' + figma[i])));
  const g = toOklch(toOklab(parse('#' + linear[i])));
  console.log(
    `  ${String(i).padStart(2,'0')} | ${f.l.toFixed(4)} | ${g.l.toFixed(4)} | ${f.c.toFixed(5)} | ${g.c.toFixed(5)} | ${f.h?.toFixed(1)??'--'} | ${g.h?.toFixed(1)??'--'}`
  );
}
