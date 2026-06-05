#!/usr/bin/env node
import { resolve, isAbsolute } from 'node:path';
import { findProjectRoot, loadCatalog, importSvg, runPipeline } from '@leds/core';

const [, , cmd, ...args] = process.argv;
const root = findProjectRoot();

function resolvePath(p: string) {
  return isAbsolute(p) ? p : resolve(root, p);
}

function usage() {
  console.log(`LEDS CLI v0.1

  leds catalog          — list modules
  leds import <svg>     — import geometry
  leds run <svg>        — full pipeline
    --depth 100 --rim 15 --module <id>
`);
}

try {
  if (cmd === 'catalog') {
    const c = loadCatalog();
    console.log('ID                                      PRICE  PITCH  DEPTH');
    for (const m of c.modules) {
      console.log(
        `${m.id.padEnd(40)} ${String(m.pricing.unitPrice ?? 0).padStart(5)} ₽  ${String(m.placement.recommendedPitchMm).padStart(4)}  ${m.placement.depthMinMm}-${m.placement.depthMaxMm} mm`
      );
    }
  } else if (cmd === 'import') {
    const path = args[0];
    if (!path) throw new Error('path required');
    console.log(JSON.stringify(importSvg(resolvePath(path)), null, 2));
  } else if (cmd === 'run') {
    const path = args.find((a) => !a.startsWith('--'));
    if (!path) throw new Error('path required');
    const depth = numArg('--depth', 100);
    const rim = numArg('--rim', 15);
    const modIdx = args.indexOf('--module');
    const moduleId = modIdx >= 0 ? args[modIdx + 1] : undefined;
    const report = runPipeline(resolvePath(path), { depthMm: depth, rimWidthMm: rim, moduleId });
    console.log(JSON.stringify({
      source: report.import.source,
      contours: report.import.contours.length,
      module: report.module,
      modules_placed: report.placement.moduleCount,
      coverage: +report.placement.coverageEstimate.toFixed(3),
      uniformity: +report.simulation.uniformityIndex.toFixed(2),
      alerts: report.simulation.alerts.length,
      power_w: +report.power.totalPowerW.toFixed(2),
      psu_count: report.power.psuCountEstimate,
      cost_rub: Math.round(report.cost.totalCost),
      notes: report.placement.notes,
    }, null, 2));
  } else {
    usage();
    process.exit(cmd ? 1 : 0);
  }
} catch (e) {
  console.error('error:', e instanceof Error ? e.message : e);
  process.exit(1);
}

function numArg(flag: string, def: number): number {
  const i = args.indexOf(flag);
  return i >= 0 ? parseFloat(args[i + 1]) : def;
}
