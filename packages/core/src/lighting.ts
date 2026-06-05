import type { LedModule, ModulePlacement, Point2, ShapeGroup, SimulationResult } from './types.js';
import { bbox, pointInRing } from './geometry.js';

export function simulate(
  safe: Point2[],
  placements: ModulePlacement[],
  module: LedModule,
  depthMm: number,
  cellMm = 5
): SimulationResult {
  const { minX, minY, maxX, maxY } = bbox(safe);
  const margin = 20;
  const originX = minX - margin;
  const originY = minY - margin;
  const width = Math.ceil((maxX - minX + 2 * margin) / cellMm) + 1;
  const height = Math.ceil((maxY - minY + 2 * margin) / cellMm) + 1;
  const values = new Float32Array(width * height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const wx = originX + x * cellMm;
      const wy = originY + y * cellMm;
      if (!pointInRing({ x: wx, y: wy }, safe)) continue;
      let sum = 0;
      for (const pl of placements) {
        sum += spot(module, pl, { x: wx, y: wy }, depthMm);
      }
      values[y * width + x] = sum;
    }
  }

  const mask = [...values].filter((v) => v > 0);
  const mean = mask.reduce((a, b) => a + b, 0) / (mask.length || 1);
  const minI = mask.length ? Math.min(...mask) : 0;
  const maxI = mask.length ? Math.max(...mask) : 0;
  const std = Math.sqrt(mask.reduce((a, v) => a + (v - mean) ** 2, 0) / (mask.length || 1)) || 1;

  const alerts: SimulationResult['alerts'] = [];
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const v = values[y * width + x];
      if (v <= 0) continue;
      const ratio = v / mean;
      const wx = originX + x * cellMm;
      const wy = originY + y * cellMm;
      if (ratio < 0.7) alerts.push({ alertType: 'underlit', x: wx, y: wy, severity: 0.7 - ratio, message: 'Недосвет' });
      else if (ratio > 1.35) alerts.push({ alertType: 'overlit', x: wx, y: wy, severity: ratio - 1.35, message: 'Переуплотнение' });
      if (alerts.length >= 25) break;
    }
  }

  return {
    width, height, originX, originY, cellMm, values,
    minIlluminance: minI, maxIlluminance: maxI, meanIlluminance: mean,
    uniformityIndex: mean / std, alerts,
  };
}

function spot(module: LedModule, pl: ModulePlacement, p: Point2, depthMm: number): number {
  const sigma0 = (module.lightModel.params.sigmaMm as number) ?? 40;
  const k = (module.lightModel.params.kSigma as number) ?? 0.12;
  const sigma = sigma0 + k * depthMm;
  const r2 = (p.x - pl.x) ** 2 + (p.y - pl.y) ** 2;
  return module.electrical.lumens * Math.exp(-r2 / (2 * sigma * sigma));
}
