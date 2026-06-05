# Алгоритмы размещения модулей — исследование и выбор стратегии

## 1. Сравнительный анализ методов

| Метод | Суть | Плюсы | Минусы | Применимость к вывескам |
|-------|------|-------|--------|-------------------------|
| **1. Skeletonization / MAT** | Ось, равноудалённая от границ | Естественная «линия» в буквах; хорош для ветвлений | Нестабилен на острых углах; шум на дугах | **Высокая** — как первичный генератор кандидатов |
| **2. Voronoi Diagram** | Ячейки относительно границ | Центры максимальных кругов внутри | Сложность на кривых; дискретизация | **Высокая** — эквивалент MAT для сегментов |
| **3. Distance Transform** | Поле расстояния до границы | Быстро; зоны тонких штрихов; недосвет | Не даёт порядок размещения сам по себе | **Очень высокая** — оценка качества и fill |
| **4. Polygon Offset** | Внутренние параллельные кривые | Учёт борта; физические границы | Исчезающие контуры при большом offset | **Обязательно** — safe zone |
| **5. Medial Axis Transform** | Непрерывная ось | Теоретически оптимально | Чувствительность к шуму | = п.1 |
| **6. Poisson Disk Sampling** | Минимальное расстояние между точками | Равномерность «вслепую» | Игнорирует форму и углы | **Низкая** alone; **средняя** как уплотнитель |
| **7. Adaptive Placement** | Шаг зависит от ширины (DT) | Тонкие/широкие зоны | Нужна модель модуля | **Высокая** |
| **8. Hybrid** | Комбинация | Баланс всех кейсов | Сложность реализации | **Выбрано для LEDS** |

### Почему недопустима «примитивная сетка»

Равномерная решётка:
- не следует скелету буквы «S», «B»;
- заливает углы или пропускает их;
- не сжимается в тонких штрихах → коллизии с бортом;
- не разрежается в широких полях → перерасход модулей.

---

## 2. Рекомендуемая стратегия: **Hybrid Adaptive Medial Placement (HAMP)**

### Фаза A — Подготовка геометрии

```
Input: contours (outer + holes), rimWidth, moduleFootprint
1. offset_in = rimWidth + max(footprint.width, footprint.height)/2 + safetyMargin
2. safePolygon = offset(outer, -offset_in) \ holes_offset
3. Если safePolygon пуст → ERROR «слишком узкий канал»
4. Rasterize safePolygon → binary mask M
5. Compute DT[x] = distance to nearest boundary (Euclidean, Felzenszwalb)
```

### Фаза B — Извлечение структуры

```
6. Medial axis: Voronoi of boundary segments OR Zhang-Suen thinning on M
7. Prune spurs shorter than pitch_min / 2
8. Classify pixels/segments:
   - thin:  DT < 1.5 × pitch_min
   - wide:  DT >= 2 × pitch_min
   - corner: angle < 60° at boundary sample
```

### Фаза C — Начальное размещение (seeding)

**Вдоль скелета (wide + medium):**
- Обход ветвей skeleton graph от узлов высокого DT.
- Позиции: локальные максимумы DT на оси, шаг `s = clamp(pitch_recommended, pitch_min, α·DT)` где α ≈ 0.8–1.2.
- Ориентация модуля: касательная к оси (для линейных strip) или 0° для point LEDs.

**Тонкие зоны (thin):**
- 1D: ось = сегмент скелета; ровно `n = floor(length / pitch_min)` модулей, центрирование.
- Если `length < pitch_min` → одна точка в максимуме DT; alert «критический штрих».

**Углы (corner):**
- На биссектрисе угла точка на `d = k·depth` (калибруемый k).
- Не дублировать, если уже есть модуль в радиусе `pitch_min/2`.

### Фаза D — Уплотнение и Poisson-подобная доводка

```
9. Построить occupancy: каждый модуль — ядро интенсивности (для коллизий — footprint AABB rotated)
10. While exists region where simulated illuminance < I_min:
       candidate = argmax(DT) in underlit cell (grid 5mm)
       if feasible(candidate): add module
       else: mark failed
11. While exists pair with dist < pitch_min * 0.7 AND local illuminance > I_max:
       remove weaker contributor (меньше прироста к равномерности)
```

### Фаза E — Глобальная оптимизация (локальная)

- **Variable neighborhood search:** сдвиги ±5 мм вдоль скелета для снижения `σ(I)`.
- Целевая функция:

```
J = w1·U_uniformity + w2·N_modules + w3·P_penetration + w4·P_overlap_light
```

где:
- `U_uniformity = mean(I) / std(I)` на маске (выше — лучше);
- `N_modules` — штраф за количество;
- `P_penetration` — выход за safePolygon;
- `P_overlap_light` — превышение локального порога.

- Ограничения: fixed модули не двигаются (режим 2).

### Фаза F — Валидация

- Каждый модуль: centroid ∈ safePolygon.
- Попарная дистанция ≥ `pitch_min` (с исключением для corner injection).
- Отчёт: count, coverage %, estimated uniformity.

---

## 3. Режимы работы алгоритма

| Режим | Поведение HAMP |
|-------|----------------|
| **Auto** | A→F полностью |
| **SemiAuto** | A→F, затем freeze `fixed`; при move — только F + local light recalc |
| **Expert** | Только F на пользовательских позициях |

---

## 4. Псевдокод (сжатый)

```text
function placeAuto(contours, params, module):
    safe = buildSafeZone(contours, params, module)
    dt = distanceTransform(safe)
    axis = medialAxis(safe)
    placements = []
    for branch in axis.branches:
        placements += sampleAlong(branch, dt, module.pitch)
    placements += placeThinSegments(safe, dt, module)
    placements += placeCorners(safe, params.depth)
    placements = poissonRefine(placements, dt, module)
    placements = optimizeLocal(placements, objective J)
    return validate(placements, safe)
```

---

## 5. Альтернативы, отклонённые как основные

| Подход | Причина отклонения как primary |
|--------|-------------------------------|
| Только Poisson | Не привязан к топологии букв |
| Только Voronoi peaks | Слишком много пиков на кривых |
| Только offset loops | Не заполняет центр широких зон |
| ML (нейросеть) | Нет воспроизводимости, нужны тысячи размеченных данных |

**ML (фаза 2+):** можно предсказывать `pitch factor` по классу шрифта, но не координаты напрямую.

---

## 6. Параметры, настраиваемые технологом

| Параметр | По умолчанию | Диапазон |
|----------|--------------|----------|
| `safetyMarginMm` | 2 | 0–5 |
| `I_min` (отн.) | 0.7 | 0.5–0.9 |
| `I_max` (отн.) | 1.3 | 1.1–1.5 |
| `uniformityTarget` | 0.75 | 0.6–0.9 |
| `cornerInject` | on | on/off |
| `maxModules` | ∞ | лимит бюджета |

---

## 7. Тестовые сценарии для регрессии

1. Буква **O** 300 мм — кольцо равномерно.  
2. **I** ширина 40 мм — линия по оси.  
3. **M** — три ветки без пропуска стыка.  
4. Декоративный шрифт — переменная ширина.  
5. Логотип с островами и отверстиями.  
6. Острый **V** — нет тёмного угла.  
7. Световой короб 2×1 м — разумное N, время < 30 с.

---

## 8. Библиотеки и ссылки (реализация)

- Clipper2 — offset  
- Felzenszwalb & Huttenlocher — DT O(n)  
- Segment Voronoi — `geo` + custom или CGAL (если понадобится)  
- Local optimization — собственный VNS, 2–3 итерации  

**Итог:** оптимальная комбинация = **Offset + Distance Transform + Medial Axis + Adaptive sampling + Light-driven refine + Local optimization**.
