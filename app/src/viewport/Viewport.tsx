import { useEffect, useRef } from 'react';
import type { LedModule } from '@leds/core';
import { buildTopology, buildSafeZone, placeModules, simulate } from '@leds/core';

const DEMO_OUTER = [
  { x: 40, y: 40 },
  { x: 360, y: 40 },
  { x: 360, y: 200 },
  { x: 40, y: 200 },
];

const DEMO_HOLE = [
  { x: 120, y: 80 },
  { x: 280, y: 80 },
  { x: 280, y: 160 },
  { x: 120, y: 160 },
];

interface Props {
  depthMm: number;
  rimMm: number;
  module?: LedModule;
}

export function Viewport({ depthMm, rimMm, module }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !module) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const group = buildTopology([
      { id: 'o', kind: 'outer', points: DEMO_OUTER, closed: true },
      { id: 'h', kind: 'outer', points: DEMO_HOLE, closed: true },
    ])[0];

    const extent = Math.max(module.footprint.lengthMm, module.footprint.widthMm);
    const safe = buildSafeZone(group, rimMm, extent, 2);
    const placement = placeModules(group, { depthMm, rimWidthMm: rimMm, safetyMarginMm: 2 }, module, 8);
    const sim = simulate(safe, placement.placements, module, depthMm, 8);

    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    ctx.scale(dpr, dpr);
    ctx.fillStyle = '#1a1d23';
    ctx.fillRect(0, 0, w, h);

    const scale = Math.min((w - 80) / 400, (h - 80) / 200);
    const ox = 40, oy = 40;
    const tx = (x: number) => ox + x * scale;
    const ty = (y: number) => oy + y * scale;

    // heatmap
    for (let y = 0; y < sim.height; y++) {
      for (let x = 0; x < sim.width; x++) {
        const v = sim.values[y * sim.width + x];
        if (v <= 0) continue;
        const t = Math.min(1, v / (sim.maxIlluminance || 1));
        ctx.fillStyle = `rgba(255, ${Math.floor(80 + 140 * t)}, 40, 0.35)`;
        const wx = sim.originX + x * sim.cellMm;
        const wy = sim.originY + y * sim.cellMm;
        ctx.fillRect(tx(wx), ty(wy), sim.cellMm * scale, sim.cellMm * scale);
      }
    }

    drawPoly(ctx, DEMO_OUTER, tx, ty, '#5b8def', 2);
    drawPoly(ctx, DEMO_HOLE, tx, ty, '#5b8def', 1);
    drawPoly(ctx, safe, tx, ty, '#3dd68c55', 1, true);

    for (const pl of placement.placements) {
      ctx.save();
      ctx.translate(tx(pl.x), ty(pl.y));
      ctx.rotate((pl.angleDeg * Math.PI) / 180);
      ctx.fillStyle = '#ffd166';
      ctx.strokeStyle = '#1a1d23';
      ctx.lineWidth = 1;
      const lw = module.footprint.lengthMm * scale * 0.3;
      const hw = module.footprint.widthMm * scale * 0.3;
      ctx.fillRect(-lw / 2, -hw / 2, lw, hw);
      ctx.strokeRect(-lw / 2, -hw / 2, lw, hw);
      ctx.restore();
    }

    ctx.fillStyle = '#9aa3b2';
    ctx.font = '12px Segoe UI, system-ui';
    ctx.fillText(`Модулей: ${placement.moduleCount} · U=${sim.uniformityIndex.toFixed(2)} · ~${Math.round(placement.moduleCount * (module.pricing.unitPrice ?? 0))} ₽`, 12, h - 12);
  }, [depthMm, rimMm, module]);

  return (
    <div className="viewport-wrap">
      <canvas ref={canvasRef} className="viewport-canvas" />
      <div className="viewport-toolbar">
        <span>Демо: буква с отверстием (синтетика)</span>
      </div>
    </div>
  );
}

function drawPoly(
  ctx: CanvasRenderingContext2D,
  pts: { x: number; y: number }[],
  tx: (n: number) => number,
  ty: (n: number) => number,
  color: string,
  width: number,
  fill = false
) {
  ctx.beginPath();
  pts.forEach((p, i) => (i ? ctx.lineTo(tx(p.x), ty(p.y)) : ctx.moveTo(tx(p.x), ty(p.y))));
  ctx.closePath();
  if (fill) ctx.fillStyle = color;
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  if (fill) ctx.fill();
  ctx.stroke();
}
