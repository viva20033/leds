import type { CostBreakdown, LedModule, ModulePlacement, PowerPlan } from './types.js';

export function planPower(placements: ModulePlacement[], module: LedModule): PowerPlan {
  const maxSeries = module.chain.maxModulesInSeries;
  const groups: PowerPlan['groups'] = [];
  let chainId = 1;
  for (let i = 0; i < placements.length; i += maxSeries) {
    const chunk = placements.slice(i, i + maxSeries);
    groups.push({
      chainId: chainId++,
      moduleCount: chunk.length,
      totalPowerW: chunk.length * module.electrical.powerW,
      maxSpanMm: maxSpan(chunk),
    });
  }
  const totalPowerW = placements.length * module.electrical.powerW;
  const psuCountEstimate = Math.max(1, Math.ceil(totalPowerW / (100 * 0.8)));
  return { groups, totalPowerW, totalModules: placements.length, psuCountEstimate };
}

export function estimateCost(placements: ModulePlacement[], module: LedModule, psuCount: number): CostBreakdown {
  const unit = module.pricing.unitPrice ?? 0;
  const moduleCost = placements.length * unit;
  const psuCost = psuCount * 850;
  return {
    moduleCost, psuCost, totalCost: moduleCost + psuCost,
    currency: module.pricing.currency ?? 'RUB',
    moduleCount: placements.length,
  };
}

function maxSpan(placements: ModulePlacement[]): number {
  let max = 0;
  for (let i = 0; i < placements.length; i++) {
    for (let j = i + 1; j < placements.length; j++) {
      max = Math.max(max, Math.hypot(placements[i].x - placements[j].x, placements[i].y - placements[j].y));
    }
  }
  return max;
}
