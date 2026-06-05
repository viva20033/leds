import { useMemo, useState } from 'react';
import { Viewport } from './viewport/Viewport';
import { PropertyPanel } from './panels/PropertyPanel';
import { ModuleLibrary } from './panels/ModuleLibrary';
import { catalogModules } from './data/catalog';

export function App() {
  const [depth, setDepth] = useState(100);
  const [rim, setRim] = useState(15);
  const [moduleId, setModuleId] = useState(catalogModules[1]?.id ?? '');
  const module = useMemo(() => catalogModules.find((m) => m.id === moduleId), [moduleId]);

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">LEDS</div>
        <span className="subtitle">v0.1 — проектирование засветки</span>
      </header>
      <aside className="sidebar left">
        <ModuleLibrary modules={catalogModules} selectedId={moduleId} onSelect={setModuleId} />
      </aside>
      <main className="workspace">
        <Viewport depthMm={depth} rimMm={rim} module={module} />
      </main>
      <aside className="sidebar right">
        <PropertyPanel depth={depth} rim={rim} module={module} onDepth={setDepth} onRim={setRim} />
      </aside>
    </div>
  );
}
