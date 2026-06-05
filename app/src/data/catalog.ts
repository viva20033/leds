import type { LedModule } from '@leds/core';

/** Embedded catalog for browser (no fs). Mirrors catalog/modules/*.json */
export const catalogModules: LedModule[] = [
  {
    id: 'elf.sol-plus-dot.1smd-2835.warm',
    vendor: 'ELF',
    modelName: 'SOL+DOT 1SMD тёплый',
    footprint: { lengthMm: 16, widthMm: 8.5, heightMm: 8, orientation: 'point' },
    electrical: { voltageV: 12, powerW: 0.24, lumens: 37 },
    placement: { depthMinMm: 40, depthMaxMm: 70, recommendedPitchMm: 70, minPitchMm: 45 },
    chain: { maxCenterDistanceMm: 70, maxModulesInSeries: 50 },
    pricing: { unitPrice: 35, currency: 'RUB' },
    lightModel: { type: 'gaussian_cos_asymmetric', params: { sigmaMm: 22, kSigma: 0.1 } },
  },
  {
    id: 'elf.sol-plus.2smd-2835.white',
    vendor: 'ELF',
    modelName: 'SOL+ 2SMD белый',
    footprint: { lengthMm: 48.5, widthMm: 17.7, heightMm: 9.5, orientation: 'linear' },
    electrical: { voltageV: 12, powerW: 0.93, lumens: 140 },
    placement: { depthMinMm: 70, depthMaxMm: 130, recommendedPitchMm: 250, minPitchMm: 120 },
    chain: { maxCenterDistanceMm: 250, maxModulesInSeries: 20 },
    pricing: { unitPrice: 43, currency: 'RUB' },
    lightModel: { type: 'gaussian_cos', params: { sigmaMm: 55, kSigma: 0.12 } },
  },
  {
    id: 'elf.sol-plus.3smd-2835.cold',
    vendor: 'ELF',
    modelName: 'SOL+ 3SMD холодный',
    footprint: { lengthMm: 71.5, widthMm: 17.7, heightMm: 9.5, orientation: 'linear' },
    electrical: { voltageV: 12, powerW: 1.46, lumens: 210 },
    placement: { depthMinMm: 80, depthMaxMm: 170, recommendedPitchMm: 300, minPitchMm: 150 },
    chain: { maxCenterDistanceMm: 300, maxModulesInSeries: 10 },
    pricing: { unitPrice: 60, currency: 'RUB' },
    lightModel: { type: 'gaussian_cos', params: { sigmaMm: 70, kSigma: 0.14 } },
  },
];
