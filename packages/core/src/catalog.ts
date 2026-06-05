import { existsSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Catalog, LedModule } from './types.js';

export function findProjectRoot(start = process.cwd()): string {
  let dir = start;
  while (dir !== dirname(dir)) {
    if (existsSync(join(dir, 'catalog', 'manifest.json'))) return dir;
    dir = dirname(dir);
  }
  return join(dirname(fileURLToPath(import.meta.url)), '../../..');
}

export function defaultCatalogPath(): string {
  return join(findProjectRoot(), 'catalog');
}

export function loadCatalog(dir = defaultCatalogPath()): Catalog {
  const manifest = JSON.parse(readFileSync(join(dir, 'manifest.json'), 'utf8'));
  const modules: LedModule[] = manifest.modules.map((rel: string) =>
    JSON.parse(readFileSync(join(dir, rel), 'utf8'))
  );
  return { manifest, modules };
}

export function moduleForDepth(catalog: Catalog, depthMm: number): LedModule | undefined {
  const rules = catalog.manifest.defaults?.preferredModuleByDepth;
  if (!rules) return catalog.modules[0];
  const rule = rules.find((r) => depthMm <= r.depthMaxMm);
  return catalog.modules.find((m) => m.id === rule?.moduleId);
}

export function getModule(catalog: Catalog, id: string): LedModule | undefined {
  return catalog.modules.find((m) => m.id === id);
}
