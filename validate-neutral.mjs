import { createNeutralLightScale } from './dist/primitives/neutral.js';
import { parse, differenceEuclidean } from 'culori';

const figma = [
  'FFFFFF','F7F8FA','EBEBF5','CDCDD6','B0B0B9','93939C','787880',
  '595961','3C3C43','303034','242426','1C1C1E','101012',
];

const scale = createNeutralLightScale({
  light: '#FFFFFF',
  base: '#787880',
  dark: '#101012',
});

const diffOklab = differenceEuclidean('oklab');

console.log('step | generated | figma     | dE-OKLab | match');
console.log('-----|-----------|-----------|----------|------');
let maxDE = 0;
let totalDE = 0;
for (let i = 0; i <= 12; i++) {
  const gen = scale.steps[i].replace('#', '').toUpperCase();
  const fig = figma[i];
  const d = diffOklab(parse('#' + fig), parse(scale.steps[i]));
  totalDE += d;
  if (d > maxDE) maxDE = d;
  const ok = d < 0.015 ? '✓' : d < 0.03 ? '~' : '✗';
  console.log(`  ${String(i).padStart(2,'0')} | ${gen}    | ${fig}    | ${d.toFixed(4)}  | ${ok}`);
}
console.log(`\ntotal dE: ${totalDE.toFixed(4)}  max dE: ${maxDE.toFixed(4)}`);
