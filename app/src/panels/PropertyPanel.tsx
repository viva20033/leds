import type { LedModule } from '@leds/core';

interface Props {
  depth: number;
  rim: number;
  module?: LedModule;
  onDepth: (v: number) => void;
  onRim: (v: number) => void;
}

export function PropertyPanel({ depth, rim, module, onDepth, onRim }: Props) {
  return (
    <div className="panel">
      <h2>Свойства</h2>
      <label>
        Глубина, мм
        <input type="number" value={depth} onChange={(e) => onDepth(+e.target.value)} />
      </label>
      <label>
        Борт, мм
        <input type="number" value={rim} onChange={(e) => onRim(+e.target.value)} />
      </label>
      {module && (
        <div className="panel-block">
          <h3>Модуль</h3>
          <p>{module.modelName}</p>
          <ul>
            <li>Шаг: {module.placement.recommendedPitchMm} мм</li>
            <li>Глубина: {module.placement.depthMinMm}–{module.placement.depthMaxMm} мм</li>
            <li>Цена: {module.pricing.unitPrice} ₽</li>
            <li>Мощность: {module.electrical.powerW} W</li>
          </ul>
        </div>
      )}
    </div>
  );
}
