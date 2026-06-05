# Алгоритмы моделирования освещения и равномерности засветки

## 1. Физическая модель (упрощённая инженерная)

Для рекламного производства не требуется ray tracing в v1. Нужна **воспроизводимая инженерная модель**, калибруемая по эталонам.

### 1.1 Предпосылки

- Модуль — **Ламберт-подобный** источник с ограниченным углом `θ_half`.
- Рассеиватель смягчает пятно → увеличение `σ` (мм).
- Глубина канала `d` влияет на ширину пятна на лицевой плоскости: `σ_eff = σ_0 + k_d · d`.
- Суперпозиция линейна (без учёта насыщения материала) — достаточно для сравнения зон.

### 1.2 Модель пятна одного модуля

**Вариант A (рекомендуемый v1): Separable Gaussian × angular falloff**

```
I(x, y) = I₀ · exp(-r² / (2σ_eff²)) · cos^n(θ)
```

где:
- `r` — расстояние в плоскости лица до проекции модуля;
- `θ` — угол от нормали (для плоского модуля, направленного в лицо, θ ≈ atan(r/d));
- `n` — жёсткость диаграммы (из `angleDeg`: n ≈ 2 для 120°, выше для узкого);
- `I₀` — нормировка по `lumens` модуля.

**Вариант B:** Kernel из таблицы (LUT 64×64) — для нестандартных COB.

**Вариант C (v2):** Двухслойная модель: прямой + рассеянный (`I = I_direct + I_scatter`).

### 1.3 Учёт рассеивателя

```
I_face = T_diffuser · I_incident
σ_eff = σ_module + σ_diffuser( material, thickness )
```

Справочник материалов: `opal_3mm → T=0.65, σ_add=8mm`.

---

## 2. Суперпозиция и heatmap

### 2.1 Дискретизация

1. Bounding box изделия + margin 50 mm.
2. Сетка `W × H` (адаптивно: 2 мм/px для превью, 1 мм/px для финала).
3. Для каждого модуля `i` с позицией `(x_i, y_i)`:

```
I_total[x,y] += I_i(x - x_i, y - y_i, depth, angle_i)
```

4. Оптимизация: ограничить вклад радиусом `3·σ_eff` (compact support).

### 2.2 Параллелизация

- Разбиение поля на тайлы (rayon).
- Опционально GPU: fragment shader accumulation.

### 2.3 Нормализация для UI

```
I_norm = (I - I_min) / (I_max - I_min)   // на маске safePolygon
```

Colormap: perceptually uniform (viridis / turbo), прозрачность 40%.

---

## 3. Метрики равномерности

| Метрика | Формула | Назначение |
|---------|---------|------------|
| **Uniformity index** | `U = mean(I) / std(I)` на маске | Глобальная оценка |
| **Min ratio** | `min(I) / mean(I)` | Недосвет |
| **Max ratio** | `max(I) / mean(I)` | Пересвет |
| **Coverage** | % площади где `I >= I_min` | Полнота |
| **Hotspot index** | площадь где `I > I_max` | Пятна |

**Целевые пороги (по умолчанию):**
- `min(I)/mean(I) ≥ 0.7`
- `max(I)/mean(I) ≤ 1.35`
- `U ≥ 0.75`

---

## 4. Детекция рисков

### 4.1 Зоны недостаточной засветки

```
Underlit = { p ∈ M | I(p) < I_min · mean(I) }
```

- Connected components → bounding boxes → alerts с площадью (мм²).
- Рекомендация: «добавить модуль в (x,y)» = argmax DT в компоненте.

### 4.2 Переуплотнение

```
Overlit = { p | I(p) > I_max · mean(I) }
```

- Если компонент рядом с парой модулей на расстоянии < 0.7·pitch → suggest remove one.

### 4.3 Риск светлых пятен (hotspot)

- Локальный максимум `I` выше `1.5·mean` и площадь > 100 мм².
- Частая причина: два модуля на изгибе без поворота strip.

### 4.4 Риск тёмных зон (dark corner)

- Угол контура < 60° AND `I` на биссектрисе < 0.6·mean в полосе 30 мм.
- Связано с placement corner injection.

---

## 5. Связь с электротехникой

```
P_total = Σ powerW_i
Φ_total = Σ lumens_i
```

Подбор БП:
- Группировка по `voltage`, max `modules_per_chain` из datasheets.
- `I_chain = Σ I_module`, запас 20%.
- `PSU_count = ceil(P_total / (PSU.maxPower · derating))`

---

## 6. Себестоимость (ориентировочная)

```
Cost = Σ (n_i · price_module_i) + Σ (n_psu · price_psu) + optionalLabor
```

Труд — константа из настроек (руб/м²).

---

## 7. Калибровка модели

### Процедура (с технологом)

1. Физический образец: буква A, модуль X, глубина 80 мм.
2. Фото лица → grayscale (опционально v1.1).
3. Fit `σ_0`, `n`, `T` минимизацией MSE между `I_sim` и `I_photo` на маске.
4. Сохранение пресета `calibration/vendor_X_opal.json`.

### Без фото (v1)

- Табличные пресеты по типу рассеивателя и глубине.
- Технолог сдвигает ползунок «интенсивность пятна» ±15%.

---

## 8. Плагин световой модели

```json
{
  "lightModel": {
    "type": "gaussian_cos",
    "params": { "sigmaMm": 12, "cosPower": 2.5, "lumenScale": 1.0 },
    "depthFactor": { "k_sigma": 0.15, "k_angle": 0.02 }
  }
}
```

WASM hook (v2):
```text
fn spot(intensity_at: (x,y), module_pose, depth, params) -> f32
```

---

## 9. Псевдокод симуляции

```text
function simulate(placements, modules, params, grid):
    I = zeros(grid)
    σ_eff = module.σ + params.k_d * params.depth
    for p in placements:
        kernel = makeGaussianCos(p, σ_eff, module)
        I += stamp(kernel, grid, support=3*σ_eff)
    mask = rasterize(safePolygon)
    I *= mask
    stats = computeUniformity(I, mask)
    alerts = detectRisks(I, mask, placements, thresholds)
    return { I, stats, alerts }
```

---

## 10. Визуализация в UI

| Слой | Описание |
|------|----------|
| Heatmap | WebGL texture, blend multiply |
| Contours alerts | Красный — under, синий — over, жёлтый — hotspot |
| Legend | min / mean / max в люменах-эквив. |
| Toggle | «Показать только риски» |

---

## 11. Ограничения модели (честно для пользователя)

- Не моделирует: отражения от борта, температурный дрейф LED, старение.
- Не заменяет финальный выкрас в цехе.
- **Назначение:** сравнительная оценка вариантов и раннее выявление ошибок.

---

## 12. План улучшений post-v1

1. Import фото калибровки (homography к контуру).  
2. Monte Carlo для микроструктуры рассеивателя.  
3. Coupled optimization: placement + light в одной целевой функции (gradient-free).  
