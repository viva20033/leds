import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadCatalog } from './catalog.js';
import { runPipeline } from './pipeline.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '../../..');
const svg = join(root, 'tests/golden/lightbox.svg');

const catalog = loadCatalog();
if (catalog.modules.length !== 3) throw new Error('catalog modules');
const report = runPipeline(svg, { depthMm: 100 });
if (report.placement.moduleCount < 1) throw new Error('expected placements');
console.log('smoke ok:', report.placement.moduleCount, 'modules, cost', Math.round(report.cost.totalCost), 'RUB');
