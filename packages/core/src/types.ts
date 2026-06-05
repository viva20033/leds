export type RingKind = 'outer' | 'hole' | 'island';

export interface Point2 {
  x: number;
  y: number;
}

export interface ContourRing {
  id: string;
  kind: RingKind;
  points: Point2[];
  closed: boolean;
  layer?: string;
}

export interface ShapeGroup {
  outer: ContourRing;
  holes: ContourRing[];
}

export interface ProductParams {
  depthMm: number;
  rimWidthMm: number;
  safetyMarginMm: number;
}

export interface LedModule {
  id: string;
  vendor: string;
  modelName: string;
  footprint: { lengthMm: number; widthMm: number; heightMm: number; orientation?: string };
  electrical: { voltageV: number; powerW: number; lumens: number; currentA?: number };
  placement: {
    depthMinMm: number;
    depthMaxMm: number;
    recommendedPitchMm: number;
    minPitchMm: number;
    maxPitchMm?: number;
  };
  chain: { maxCenterDistanceMm: number; maxModulesInSeries: number };
  pricing: { unitPrice?: number | null; currency?: string };
  lightModel: { type: string; params: Record<string, number> };
}

export interface Catalog {
  manifest: { id: string; version: string; defaults?: { preferredModuleByDepth?: { depthMaxMm: number; moduleId: string }[] } };
  modules: LedModule[];
}

export interface ModulePlacement {
  id: string;
  moduleId: string;
  x: number;
  y: number;
  angleDeg: number;
  fixed: boolean;
  userPlaced: boolean;
}

export interface PlacementResult {
  placements: ModulePlacement[];
  moduleCount: number;
  pitchUsedMm: number;
  coverageEstimate: number;
  notes: string[];
}

export interface SimulationResult {
  width: number;
  height: number;
  originX: number;
  originY: number;
  cellMm: number;
  values: Float32Array;
  minIlluminance: number;
  maxIlluminance: number;
  meanIlluminance: number;
  uniformityIndex: number;
  alerts: { alertType: string; x: number; y: number; severity: number; message: string }[];
}

export interface PowerPlan {
  groups: { chainId: number; moduleCount: number; totalPowerW: number; maxSpanMm: number }[];
  totalPowerW: number;
  totalModules: number;
  psuCountEstimate: number;
}

export interface CostBreakdown {
  moduleCost: number;
  psuCost: number;
  totalCost: number;
  currency: string;
  moduleCount: number;
}
