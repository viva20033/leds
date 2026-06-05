import type { LedModule } from '@leds/core';

interface Props {
  modules: LedModule[];
  selectedId: string;
  onSelect: (id: string) => void;
}

export function ModuleLibrary({ modules, selectedId, onSelect }: Props) {
  return (
    <div className="panel">
      <h2>Библиотека модулей</h2>
      <ul className="module-list">
        {modules.map((m) => (
          <li key={m.id}>
            <button
              type="button"
              className={m.id === selectedId ? 'active' : ''}
              onClick={() => onSelect(m.id)}
            >
              <strong>{m.modelName}</strong>
              <span>{m.pricing.unitPrice} ₽ · {m.placement.recommendedPitchMm} мм</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
