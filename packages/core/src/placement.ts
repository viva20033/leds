import { v4 as uuid } from 'uuid';
import type { LedModule, ModulePlacement, PlacementResult, Point2, ProductParams, ShapeGroup } from './types.js';
import { bbox, buildSafeZone, distanceField, pointInRing } from './geometry.js';

export function placeModules(
  group: ShapeGroup,
  params: ProductParams,
  module: LedModule,
  cellMm = 5
): PlacementResult {
  const extent = Math.max(module.footprint.lengthMm, module.footprint.widthMm);
  const safe = buildSafeZone(group, params.rimWidthMm, extent, params.safetyMarginMm);
  if (safe.length < 3) throw new Error('safe zone collapsed');

  const pitch = module.placement.recommendedPitchMm;
  const minPitch = module.placement.minPitchMm;
  const field = distanceField(group, safe, cellMm);

  const seeds = localMaxima(field, minPitch * 0.5);
  if (!seeds.length) seeds.push(centroid(safe));

  const placements: ModulePlacement[] = [];
  for (const seed of seeds) {
    addWithRadial(placements, seed, module, pitch, minPitch, safe);
  }
  gridFill(placements, safe, module, pitch, minPitch);
  fillGaps(placements, group, safe, module, minPitch, field);
  prune(placements, minPitch * 0.7);

  if (!placements.length) {
    const c = centroid(safe);
    placements.push({
      id: uuid(),
      moduleId: module.id,
      x: c.x,
      y: c.y,
      angleDeg: 0,
      fixed: false,
      userPlaced: false,
    });
  }

  return {
    placements,
    moduleCount: placements.length,
    pitchUsedMm: pitch,
    coverageEstimate: estimateCoverage(placements, module, safe),
    notes: [`HAMP v0.1 (TS): ${seeds.length} seeds, pitch ${pitch} mm`],
  };
}

function localMaxima(
  field: { width: number; height: number; originX: number; originY: number; cellMm: number; values: Float32Array },
  threshold: number
): Point2[] {
  const out: Point2[] = [];
  const { width, height, originX, originY, cellMm, values } = field;
  for (let y = 1; y < height - 1; y++) {
    for (let x = 1; x < width - 1; x++) {
      const v = values[y * width + x];
      if (v < threshold) continue;
      let max = true;
      for (let dy = -1; dy <= 1 && max; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          if (!dx && !dy) continue;
          if (values[(y + dy) * width + (x + dx)] > v) max = false;
        }
      }
      if (max) out.push({ x: originX + x * cellMm, y: originY + y * cellMm });
    }
  }
  return out;
}

function addWithRadial(
  placements: ModulePlacement[],
  seed: Point2,
  module: LedModule,
  pitch: number,
  minPitch: number,
  safe: Point2[]
) {
  const add = (p: Point2, angle: number) => {
    if (!pointInRing(p, safe)) return;
    if (placements.some((pl) => dist(pl, p) < minPitch)) return;
    placements.push({
      id: uuid(),
      moduleId: module.id,
      x: p.x,
      y: p.y,
      angleDeg: angle,
      fixed: false,
      userPlaced: false,
    });
  };
  add(seed, 0);
  for (const [dx, dy, ang] of [[1, 0, 0], [-1, 0, 0], [0, 1, 90], [0, -1, 90]] as const) {
    for (let t = pitch; t < pitch * 3; t += pitch) {
      add({ x: seed.x + dx * t, y: seed.y + dy * t }, ang);
    }
  }
}

function gridFill(
  placements: ModulePlacement[],
  safe: Point2[],
  module: LedModule,
  pitch: number,
  minPitch: number
) {
  const { minX, minY, maxX, maxY } = bbox(safe);
  for (let y = minY + pitch / 2; y <= maxY; y += pitch) {
    for (let x = minX + pitch / 2; x <= maxX; x += pitch) {
      const p = { x, y };
      if (!pointInRing(p, safe)) continue;
      if (placements.some((pl) => dist(pl, p) < minPitch)) continue;
      placements.push({
        id: uuid(),
        moduleId: module.id,
        x: p.x,
        y: p.y,
        angleDeg: 0,
        fixed: false,
        userPlaced: false,
      });
    }
  }
}

function fillGaps(
  placements: ModulePlacement[],
  group: ShapeGroup,
  safe: Point2[],
  module: LedModule,
  minPitch: number,
  field: ReturnType<typeof distanceField>
) {
  for (let n = 0; n < 40; n++) {
    let best: Point2 | null = null;
    let bestD = 0;
    for (let y = 0; y < field.height; y++) {
      for (let x = 0; x < field.width; x++) {
        const wx = field.originX + x * field.cellMm;
        const wy = field.originY + y * field.cellMm;
        const p = { x: wx, y: wy };
        if (!pointInRing(p, safe)) continue;
        const d = field.values[y * field.width + x];
        if (d < minPitch * 0.5) continue;
        const nearest = Math.min(...placements.map((pl) => dist(pl, p)), Infinity);
        if (nearest > module.placement.recommendedPitchMm * 1.05 && d > bestD) {
          best = p;
          bestD = d;
        }
      }
    }
    if (!best) break;
    placements.push({
      id: uuid(),
      moduleId: module.id,
      x: best.x,
      y: best.y,
      angleDeg: 0,
      fixed: false,
      userPlaced: false,
    });
  }
}

function prune(placements: ModulePlacement[], minDist: number) {
  const kept: ModulePlacement[] = [];
  for (const p of placements) {
    if (kept.some((k) => dist(k, p) < minDist)) continue;
    kept.push(p);
  }
  placements.length = 0;
  placements.push(...kept);
}

function dist(a: { x: number; y: number }, b: Point2) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function centroid(pts: Point2[]): Point2 {
  const s = pts.reduce((a, p) => ({ x: a.x + p.x, y: a.y + p.y }), { x: 0, y: 0 });
  return { x: s.x / pts.length, y: s.y / pts.length };
}

function estimateCoverage(placements: ModulePlacement[], module: LedModule, safe: Point2[]): number {
  const sigma = (module.lightModel.params.sigmaMm as number) ?? 50;
  const r = sigma * 2;
  const { minX, minY, maxX, maxY } = bbox(safe);
  let covered = 0, total = 0;
  for (let y = minY; y <= maxY; y += 10) {
    for (let x = minX; x <= maxX; x += 10) {
      if (!pointInRing({ x, y }, safe)) continue;
      total++;
      if (placements.some((pl) => Math.hypot(pl.x - x, pl.y - y) <= r)) covered++;
    }
  }
  return total ? covered / total : 0;
}
