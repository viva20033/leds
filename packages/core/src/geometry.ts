import type { ContourRing, Point2, ShapeGroup } from './types.js';

export function ringArea(pts: Point2[]): number {
  if (pts.length < 3) return 0;
  let a = 0;
  for (let i = 0; i < pts.length; i++) {
    const j = (i + 1) % pts.length;
    a += pts[i].x * pts[j].y - pts[j].x * pts[i].y;
  }
  return Math.abs(a * 0.5);
}

export function buildTopology(rings: ContourRing[]): ShapeGroup[] {
  const sorted = [...rings].sort((a, b) => ringArea(b.points) - ringArea(a.points));
  const used = new Set<number>();
  const groups: ShapeGroup[] = [];

  for (let i = 0; i < sorted.length; i++) {
    if (used.has(i)) continue;
    const outer = { ...sorted[i], kind: 'outer' as const };
    const holes: ContourRing[] = [];
    for (let j = i + 1; j < sorted.length; j++) {
      if (used.has(j)) continue;
      const sample = sorted[j].points[0];
      if (sample && pointInRing(sample, sorted[i].points)) {
        holes.push({ ...sorted[j], kind: 'hole' });
        used.add(j);
      }
    }
    used.add(i);
    groups.push({ outer, holes });
  }
  return groups;
}

export function pointInRing(p: Point2, ring: Point2[]): boolean {
  let inside = false;
  for (let i = 0, j = ring.length - 1; i < ring.length; j = i++) {
    const xi = ring[i].x, yi = ring[i].y;
    const xj = ring[j].x, yj = ring[j].y;
    if ((yi > p.y) !== (yj > p.y) && p.x < ((xj - xi) * (p.y - yi)) / (yj - yi + 1e-12) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

export function pointInShape(group: ShapeGroup, p: Point2): boolean {
  if (!pointInRing(p, group.outer.points)) return false;
  return !group.holes.some((h) => pointInRing(p, h.points));
}

/** Inward offset via vertex normal averaging (v0 — sufficient for rects and simple shapes). */
export function insetRing(ring: Point2[], delta: number): Point2[] {
  const n = ring.length;
  if (n < 3) return [];
  const out: Point2[] = [];
  for (let i = 0; i < n; i++) {
    const prev = ring[(i - 1 + n) % n];
    const curr = ring[i];
    const next = ring[(i + 1) % n];
    const e1 = norm({ x: curr.x - prev.x, y: curr.y - prev.y });
    const e2 = norm({ x: next.x - curr.x, y: next.y - curr.y });
    let nx = -(e1.y + e2.y);
    let ny = e1.x + e2.x;
    const len = Math.hypot(nx, ny) || 1;
    nx /= len;
    ny /= len;
    out.push({ x: curr.x + nx * delta, y: curr.y + ny * delta });
  }
  return out;
}

function norm(v: Point2): Point2 {
  const l = Math.hypot(v.x, v.y) || 1;
  return { x: v.x / l, y: v.y / l };
}

export function buildSafeZone(
  group: ShapeGroup,
  rimMm: number,
  moduleExtentMm: number,
  safetyMm: number
): Point2[] {
  const inset = rimMm + moduleExtentMm * 0.5 + safetyMm;
  let zone = insetRing(group.outer.points, -inset);
  for (const hole of group.holes) {
    const expanded = insetRing(hole.points, inset);
    zone = subtractFromZone(zone, expanded);
  }
  return zone;
}

function subtractFromZone(subject: Point2[], clip: Point2[]): Point2[] {
  // v0: keep subject points outside clip
  return subject.filter((p) => !pointInRing(p, clip));
}

export function bbox(pts: Point2[]) {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const p of pts) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  return { minX, minY, maxX, maxY };
}

export function distanceField(
  group: ShapeGroup,
  safe: Point2[],
  cellMm: number
): { width: number; height: number; originX: number; originY: number; cellMm: number; values: Float32Array } {
  const { minX, minY, maxX, maxY } = bbox(safe);
  const margin = cellMm * 2;
  const originX = minX - margin;
  const originY = minY - margin;
  const width = Math.ceil((maxX - minX + 2 * margin) / cellMm) + 1;
  const height = Math.ceil((maxY - minY + 2 * margin) / cellMm) + 1;
  const values = new Float32Array(width * height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const wx = originX + x * cellMm;
      const wy = originY + y * cellMm;
      const p = { x: wx, y: wy };
      if (!pointInShape(group, p) || !pointInRing(p, safe)) {
        values[y * width + x] = 0;
        continue;
      }
      values[y * width + x] = minDistToPolyEdges(p, safe);
    }
  }
  return { width, height, originX, originY, cellMm, values };
}

function minDistToPolyEdges(p: Point2, poly: Point2[]): number {
  let min = Infinity;
  for (let i = 0; i < poly.length; i++) {
    const a = poly[i];
    const b = poly[(i + 1) % poly.length];
    min = Math.min(min, distToSegment(p, a, b));
  }
  return min;
}

function distToSegment(p: Point2, a: Point2, b: Point2): number {
  const dx = b.x - a.x, dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-12) return Math.hypot(p.x - a.x, p.y - a.y);
  let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}
