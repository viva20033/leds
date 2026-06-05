import { readFileSync } from 'node:fs';
import type { ContourRing, Point2 } from './types.js';

export interface ImportResult {
  source: string;
  format: string;
  contours: ContourRing[];
  warnings: string[];
}

/** Lightweight SVG path importer (M/L/H/V/Z commands + rect/polygon). */
export function importSvg(path: string): ImportResult {
  const text = readFileSync(path, 'utf8');
  const warnings: string[] = [];
  const contours: ContourRing[] = [];
  let idx = 0;

  const rectRe = /<rect\b([^>]*)\/?>/gi;
  let m: RegExpExecArray | null;
  while ((m = rectRe.exec(text))) {
    const attrs = m[1];
    const x = parseAttr(attrs, 'x') ?? 0;
    const y = parseAttr(attrs, 'y') ?? 0;
    const w = parseAttr(attrs, 'width');
    const h = parseAttr(attrs, 'height');
    if (w != null && h != null) contours.push(rectContour(`ring-${idx++}`, x, y, w, h));
  }

  const polyRe = /<polygon[^>]*\bpoints="([^"]+)"/gi;
  while ((m = polyRe.exec(text))) {
    const pts = parsePointList(m[1]);
    if (pts.length >= 3) contours.push({ id: `ring-${idx++}`, kind: 'outer', points: pts, closed: true });
  }

  const pathRe = /<path[^>]*\bd="([^"]+)"/gi;
  while ((m = pathRe.exec(text))) {
    const subpaths = parseSvgPath(m[1]);
    for (const pts of subpaths) {
      if (pts.length >= 3) {
        contours.push({ id: `ring-${idx++}`, kind: 'outer', points: pts, closed: true });
      } else {
        warnings.push('skipped open/short path');
      }
    }
  }

  if (!contours.length) warnings.push('no contours extracted');
  return { source: path, format: 'svg', contours, warnings };
}

function parseAttr(attrs: string, name: string): number | undefined {
  const re = new RegExp(`\\b${name}="([^"]+)"`);
  const hit = attrs.match(re);
  return hit ? parseFloat(hit[1]) : undefined;
}

function rectContour(id: string, x: number, y: number, w: number, h: number): ContourRing {
  return {
    id,
    kind: 'outer',
    closed: true,
    points: [
      { x, y },
      { x: x + w, y },
      { x: x + w, y: y + h },
      { x, y: y + h },
    ],
  };
}

function parsePointList(s: string): Point2[] {
  return s
    .trim()
    .split(/[\s,]+/)
    .reduce<Point2[]>((acc, val, i, arr) => {
      if (i % 2 === 0 && i + 1 < arr.length) acc.push({ x: parseFloat(val), y: parseFloat(arr[i + 1]) });
      return acc;
    }, []);
}

function parseSvgPath(d: string): Point2[][] {
  const tokens = d.match(/[a-zA-Z]|-?\d*\.?\d+(?:e[-+]?\d+)?/g) ?? [];
  const subpaths: Point2[][] = [];
  let current: Point2[] = [];
  let i = 0;
  let cx = 0, cy = 0, sx = 0, sy = 0;

  const read = () => parseFloat(tokens[i++]);

  while (i < tokens.length) {
    const cmd = tokens[i++];
    if (!/[a-zA-Z]/.test(cmd)) {
      i--;
      continue;
    }
    switch (cmd) {
      case 'M':
        if (current.length >= 3) subpaths.push(current);
        current = [];
        cx = read(); cy = read(); sx = cx; sy = cy;
        current.push({ x: cx, y: cy });
        break;
      case 'L':
        cx = read(); cy = read();
        current.push({ x: cx, y: cy });
        break;
      case 'H':
        cx = read();
        current.push({ x: cx, y: cy });
        break;
      case 'V':
        cy = read();
        current.push({ x: cx, y: cy });
        break;
      case 'A': {
        read(); read(); read(); read(); read();
        cx = read(); cy = read();
        current.push({ x: cx, y: cy });
        break;
      }
      case 'Z':
      case 'z':
        if (current.length && (current[0].x !== cx || current[0].y !== cy)) {
          current.push({ x: sx, y: sy });
        }
        if (current.length >= 3) subpaths.push(current);
        current = [];
        cx = sx; cy = sy;
        break;
      default:
        break;
    }
  }
  if (current.length >= 3) subpaths.push(current);
  return subpaths;
}
