# LEDS — LED Engineering & Design System

Система автоматического проектирования светодиодной засветки для рекламно-производственных компаний.

## Статус

**Фаза 0–1 (в работе):** рабочее ядро на TypeScript, CLI, UI-прототип.

## Rust CLI

После `cargo build`:

```bash
cargo run -- catalog
cargo run -- run tests/golden/lightbox.svg --depth 100
```

### Быстрый старт

```bash
npm install
npm run build
npm run cli -- catalog
npm run cli -- run tests/golden/lightbox.svg --depth 100
npm run dev          # UI http://localhost:5173
```

## Документация

| Документ | Содержание |
|----------|------------|
| [01_TECHNICAL_SPECIFICATION.md](./docs/01_TECHNICAL_SPECIFICATION.md) | Полное ТЗ |
| [02_ARCHITECTURE.md](./docs/02_ARCHITECTURE.md) | Архитектура, стек, API |
| [03_UML.md](./docs/03_UML.md) | UML-диаграммы (Mermaid) |
| [04_DATABASE.md](./docs/04_DATABASE.md) | SQLite, ER, DDL |
| [05_PROJECT_STRUCTURE.md](./docs/05_PROJECT_STRUCTURE.md) | Структура каталогов |
| [06_DEVELOPMENT_PLAN.md](./docs/06_DEVELOPMENT_PLAN.md) | Этапы, сроки, сложность |
| [07_RISKS.md](./docs/07_RISKS.md) | Анализ рисков |
| [08_ALGORITHMS_PLACEMENT.md](./docs/08_ALGORITHMS_PLACEMENT.md) | Гибридный алгоритм размещения (HAMP) |
| [09_ALGORITHMS_LIGHTING.md](./docs/09_ALGORITHMS_LIGHTING.md) | Модель света и heatmap |

## Ключевые решения

- **Ядро:** Rust (геометрия, размещение, симуляция).
- **UI:** Tauri 2 + React + PixiJS (CAD-подобный интерфейс).
- **Размещение:** Hybrid Adaptive Medial Placement — offset + distance transform + medial axis + adaptive pitch + light-driven refine (не сетка).
- **Свет:** суперпозиция Gaussian×cos^n с калибровкой под рассеиватель.

## Каталог модулей (ELF)

В [`catalog/`](./catalog/) загружены три популярных модуля из паспортов:

| Модуль | Глубина | Шаг | Max в цепи |
|--------|---------|-----|------------|
| SOL+ 2SMD белый | 70–130 мм | 250 мм | 20 | 43 ₽ |
| SOL+ 3SMD холодный | 80–170 мм | 300 мм | 10 | 60 ₽ |
| SOL+DOT 1SMD тёплый | 40–70 мм | 70 мм | 50 | 35 ₽ |

Подробности: [`catalog/README.md`](./catalog/README.md).

## Следующий шаг

Фаза 0: инициализация monorepo и `leds-cli import` для эталонных SVG — по согласованию.
