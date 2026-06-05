# Каталог LED-модулей LEDS

Встроенный справочник для разработки и тестов. Данные из паспортов ELF (популярные позиции производства).

## Модули

| ID | Модель | Глубина | Шаг | Серия max | Применение |
|----|--------|---------|-----|-----------|------------|
| `elf.sol-plus.2smd-2835.white` | SOL+ 2SMD белый | 70–130 мм | 250 мм | 20 | 43 ₽ | Буквы, короба — **основной** |
| `elf.sol-plus.3smd-2835.cold` | SOL+ 3SMD холодный | 80–170 мм | 300 мм | 10 | 60 ₽ | Глубокие буквы, крупные короба |
| `elf.sol-plus-dot.1smd-2835.warm` | SOL+DOT 1SMD тёплый | 40–70 мм | 70 мм | 50 | 35 ₽ | Мелкие буквы, тонкие элементы |

## Важные поля для алгоритмов

- **`placement.recommendedPitchMm`** — шаг для HAMP (из паспорта «расстояние между центрами в цепи»).
- **`placement.minPitchMm`** — нижняя граница (~50% рекомендуемого) для тонких зон и углов.
- **`chain.maxCenterDistanceMm`** — лимит проводки в одной цепи (электрика).
- **`chain.maxModulesInSeries`** — разбиение цепей в `leds-electrical`.
- **`footprint`** — collision detection и offset safe zone.
- **`optics.beamAngle`** — симметричный (2SMD, 3SMD) или **асимметричный 170×130°** (DOT).
- **`lightModel.params.sigma*`** — стартовые, **требуют калибровки** на реальных образцах.

## Автовыбор по глубине

См. `manifest.json` → `defaults.preferredModuleByDepth`:

- ≤ 70 мм → DOT  
- ≤ 130 мм → 2SMD  
- > 130 мм → 3SMD  

## Цены (закупочные)

| LED | Модуль | Цена |
|-----|--------|------|
| 1 | SOL+DOT | 35 ₽ |
| 2 | SOL+ 2SMD | 43 ₽ |
| 3 | SOL+ 3SMD | 60 ₽ |

Используются в расчёте себестоимости и КП (`pricing.unitPrice`).

## Схема

JSON Schema: [`schema/module.schema.json`](./schema/module.schema.json)
