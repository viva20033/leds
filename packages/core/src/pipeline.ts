import { loadCatalog, moduleForDepth, getModule } from './catalog.js';
import { importSvg } from './import-svg.js';
import { buildTopology, buildSafeZone } from './geometry.js';
import { placeModules } from './placement.js';
import { simulate } from './lighting.js';
import { planPower, estimateCost } from './electrical.js';
import type { Catalog, ProductParams } from './types.js';

export interface RunOptions {
  depthMm?: number;
  rimWidthMm?: number;
  moduleId?: string;
  catalog?: Catalog;
}

export function runPipeline(svgPath: string, options: RunOptions = {}) {
  const catalog = options.catalog ?? loadCatalog();
  const depthMm = options.depthMm ?? 100;
  const rimWidthMm = options.rimWidthMm ?? 15;
  const importResult = importSvg(svgPath);
  const groups = buildTopology(importResult.contours);
  if (!groups.length) throw new Error('no shape groups');

  const module =
    (options.moduleId && getModule(catalog, options.moduleId)) ||
    moduleForDepth(catalog, depthMm);
  if (!module) throw new Error('module not found');

  const params: ProductParams = { depthMm, rimWidthMm, safetyMarginMm: 2 };
  const group = groups[0];
  const extent = Math.max(module.footprint.lengthMm, module.footprint.widthMm);
  const safe = buildSafeZone(group, rimWidthMm, extent, 2);
  const placement = placeModules(group, params, module);
  const simulation = simulate(safe, placement.placements, module, depthMm);
  const power = planPower(placement.placements, module);
  const cost = estimateCost(placement.placements, module, power.psuCountEstimate);

  return {
    import: importResult,
    groups,
    module: module.id,
    placement,
    simulation,
    power,
    cost,
  };
}
