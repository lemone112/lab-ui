import { parse, converter, clampChroma } from 'culori';

const toOklch = converter('oklch');
const toOklab = converter('oklab');

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const baseOklch = toOklch(parse('#787880'));
console.log('Base OKLCH:', baseOklch);

console.log('\nstep | figma-L   | figma-C   | C_max(sRGB)| in-gamut? | headroom');
console.log('-----|-----------|-----------|------------|-----------|----------');
for (let i = 0; i <= 12; i++) {
  const lab = toOklab(parse('#' + figma[i]));
  const lch = toOklch(parse('#' + figma[i]));
  // Compute max C in sRGB at this L and H
  const maxC = clampChroma({ mode: 'oklch', l: lch.l, c: 0.4, h: baseOklch.h, alpha: 1 }, 'oklch');
  const headroom = maxC.c / lch.c;
  const inGamut = headroom >= 0.99 && headroom <= 1.01;
  console.log(
    `  ${String(i).padStart(2,'0')} | ${lch.l.toFixed(4)} | ${lch.c.toFixed(5)} | ${maxC.c.toFixed(5)}    | ${inGamut?'BOUNDARY':'inside  '} | ${headroom.toFixed(2)}x`
  );
}
