# Структура каталогов проекта LEDS

```
leds/
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci-core.yml          # cargo test, clippy
│       ├── ci-ui.yml            # npm test, lint
│       └── release.yml          # Tauri build, sign
│
├── docs/                        # Проектная документация (этот пакет)
│   ├── 01_TECHNICAL_SPECIFICATION.md
│   ├── 02_ARCHITECTURE.md
│   ├── 03_UML.md
│   ├── 04_DATABASE.md
│   ├── 05_PROJECT_STRUCTURE.md
│   ├── 06_DEVELOPMENT_PLAN.md
│   ├── 07_RISKS.md
│   ├── 08_ALGORITHMS_PLACEMENT.md
│   └── 09_ALGORITHMS_LIGHTING.md
│
├── crates/                      # Rust workspace
│   ├── leds-core/               # Фасад: orchestration API
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs
│   │       └── commands.rs
│   ├── leds-geometry/           # Топология, offset, MAT, DT
│   │   └── src/
│   │       ├── contour.rs
│   │       ├── topology.rs
│   │       ├── offset.rs
│   │       ├── medial_axis.rs
│   │       ├── distance_transform.rs
│   │       └── features.rs
│   ├── leds-import/             # SVG, DXF
│   │   └── src/
│   │       ├── svg.rs
│   │       ├── dxf.rs
│   │       └── normalize.rs
│   ├── leds-placement/          # Гибридный размещение
│   │   └── src/
│   │       ├── hybrid.rs
│   │       ├── seeding.rs
│   │       ├── optimizer.rs
│   │       └── validation.rs
│   ├── leds-lighting/           # Симуляция, heatmap, alerts
│   │   └── src/
│   │       ├── spot.rs
│   │       ├── superposition.rs
│   │       └── uniformity.rs
│   ├── leds-electrical/         # Цепи, БП
│   ├── leds-documents/          # PDF генерация
│   ├── leds-storage/            # SQLite, .leds ZIP
│   ├── leds-catalog/            # Парсинг справочников
│   ├── leds-plugin-api/         # Trait + manifest schema
│   └── leds-cli/                # Headless для тестов и CI
│
├── app/                         # Tauri + React
│   ├── src-tauri/
│   │   ├── src/
│   │   │   └── main.rs          # Tauri commands → leds-core
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   └── src/                     # Frontend
│       ├── main.tsx
│       ├── app/
│       ├── viewport/
│       ├── panels/
│       ├── stores/
│       ├── commands/
│       └── styles/
│
├── catalog/                     # Встроенный справочник по умолчанию
│   ├── manifest.json
│   ├── modules/
│   └── psu/
│
├── assets/                      # Иконки, шрифты UI, шаблоны Typst
│   ├── icons/
│   └── templates/
│       ├── technical_spec.typ
│       ├── bom.typ
│       ├── power_calc.typ
│       ├── wiring.typ
│       └── commercial_offer.typ
│
├── tests/
│   ├── golden/                  # Эталонные SVG + expected JSON
│   │   ├── letter_O.svg
│   │   ├── thin_script.svg
│   │   └── logo_complex.svg
│   └── integration/
│
├── benchmarks/                  # Criterion benches для placement
│
├── Cargo.toml                   # Workspace root
├── package.json                 # Frontend deps
├── rust-toolchain.toml
└── mise.toml / justfile         # Dev commands
```

---

## Соглашения

| Область | Соглашение |
|---------|------------|
| Rust | `edition 2021`, `clippy -D warnings` в CI |
| API core | Все публичные типы — `serde` для JSON bridge |
| Ошибки | `thiserror` в crates, коды `E_IMPORT_001` для UI |
| Frontend | Strict TS, ESLint, компоненты по feature |
| Коммиты | Conventional Commits (`feat(placement): ...`) |

---

## Зависимости workspace (ключевые)

```toml
# Cargo.toml (фрагмент)
[workspace.members]
members = ["crates/*", "app/src-tauri"]

[workspace.dependencies]
geo = "0.28"
clipper2 = "0.4"
usvg = "0.44"
dxf = "0.6"
rayon = "1.10"
rusqlite = "0.32"
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
```
