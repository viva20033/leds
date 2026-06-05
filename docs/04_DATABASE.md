# Структура базы данных LEDS

Локальная **SQLite 3** для метаданных, истории проектов, настроек и кэша; тяжёлая геометрия — в файле `.leds` (FlatBuffers).

---

## 1. ER-диаграмма

```mermaid
erDiagram
    projects ||--o{ project_layers : has
    projects ||--o{ project_contours : has
    projects ||--o{ placements : has
    projects ||--o{ simulation_runs : has
    projects ||--o{ power_groups : has
    projects ||--o{ documents : has
    projects }o--|| catalog_versions : uses

    catalog_versions ||--o{ led_modules : contains
    catalog_versions ||--o{ power_supplies : contains

    placements }o--|| led_modules : type
    power_groups ||--o{ placements : includes
    power_groups }o--o| power_supplies : assigned

    projects {
        text id PK
        text name
        text client_name
        real depth_mm
        real rim_width_mm
        text diffuser_type
        real diffuser_transmission
        text catalog_version_id FK
        text file_path
        text status
        datetime created_at
        datetime updated_at
    }

    project_layers {
        integer id PK
        text project_id FK
        text source_name
        integer z_order
        integer visible
        integer locked
    }

    project_contours {
        integer id PK
        text project_id FK
        integer layer_id FK
        text kind
        blob geometry_wkb
        real area_mm2
        real min_width_mm
    }

    placements {
        text id PK
        text project_id FK
        text module_catalog_id FK
        real x_mm
        real y_mm
        real angle_deg
        integer fixed
        integer chain_id
        integer user_placed
    }

    simulation_runs {
        text id PK
        text project_id FK
        integer grid_width
        integer grid_height
        real uniformity_index
        real min_illuminance
        real max_illuminance
        blob heatmap_path
        datetime created_at
    }

    simulation_alerts {
        integer id PK
        text simulation_id FK
        text alert_type
        real x_mm
        real y_mm
        real severity
        text message
    }

    power_groups {
        integer id PK
        text project_id FK
        text psu_catalog_id FK
        real total_current_a
        real total_power_w
    }

    led_modules {
        text id PK
        text catalog_version_id FK
        text vendor
        text model_name
        real width_mm
        real height_mm
        real power_w
        real lumens
        real angle_deg
        real voltage
        real min_pitch_mm
        real min_depth_mm
        real unit_price
        text light_model_json
        text plugin_id
    }

    power_supplies {
        text id PK
        text catalog_version_id FK
        real voltage
        real max_power_w
        real max_current_a
        real unit_price
        real derating_factor
    }

    catalog_versions {
        text id PK
        text version
        text signature
        datetime installed_at
    }

    documents {
        text id PK
        text project_id FK
        text doc_type
        text file_path
        datetime generated_at
    }

    app_settings {
        text key PK
        text value_json
    }

    recent_files {
        text path PK
        datetime opened_at
    }
```

---

## 2. DDL (основные таблицы)

```sql
-- Проекты
CREATE TABLE projects (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    client_name     TEXT,
    depth_mm        REAL NOT NULL DEFAULT 80,
    rim_width_mm    REAL NOT NULL DEFAULT 15,
    diffuser_type   TEXT NOT NULL DEFAULT 'opal_3mm',
    diffuser_transmission REAL NOT NULL DEFAULT 0.65,
    catalog_version_id TEXT REFERENCES catalog_versions(id),
    file_path       TEXT,
    status          TEXT NOT NULL DEFAULT 'draft',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Слои
CREATE TABLE project_layers (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    source_name     TEXT NOT NULL,
    z_order         INTEGER NOT NULL,
    visible         INTEGER NOT NULL DEFAULT 1,
    locked          INTEGER NOT NULL DEFAULT 0
);

-- Контуры (геометрия — WKB или ссылка на blob в .leds)
CREATE TABLE project_contours (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    layer_id        INTEGER REFERENCES project_layers(id),
    kind            TEXT NOT NULL CHECK (kind IN ('outer', 'hole', 'island')),
    geometry_wkb    BLOB,
    area_mm2        REAL,
    min_width_mm    REAL
);

CREATE INDEX idx_contours_project ON project_contours(project_id);

-- Размещение модулей
CREATE TABLE placements (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    module_catalog_id TEXT NOT NULL,
    x_mm            REAL NOT NULL,
    y_mm            REAL NOT NULL,
    angle_deg       REAL NOT NULL DEFAULT 0,
    fixed           INTEGER NOT NULL DEFAULT 0,
    chain_id        INTEGER,
    user_placed     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_placements_project ON placements(project_id);

-- Симуляции
CREATE TABLE simulation_runs (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    grid_width      INTEGER NOT NULL,
    grid_height      INTEGER NOT NULL,
    uniformity_index REAL,
    min_illuminance REAL,
    max_illuminance REAL,
    heatmap_path    TEXT,
    created_at      TEXT NOT NULL
);

CREATE TABLE simulation_alerts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    simulation_id   TEXT NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    alert_type      TEXT NOT NULL,
    x_mm            REAL,
    y_mm            REAL,
    severity        REAL NOT NULL,
    message         TEXT NOT NULL
);

-- Электрика
CREATE TABLE power_groups (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    psu_catalog_id  TEXT REFERENCES power_supplies(id),
    total_current_a REAL,
    total_power_w   REAL
);

-- Справочники
CREATE TABLE catalog_versions (
    id              TEXT PRIMARY KEY,
    version         TEXT NOT NULL,
    signature       TEXT,
    installed_at    TEXT NOT NULL
);

CREATE TABLE led_modules (
    id              TEXT NOT NULL,
    catalog_version_id TEXT NOT NULL REFERENCES catalog_versions(id),
    vendor          TEXT,
    model_name      TEXT NOT NULL,
    width_mm        REAL NOT NULL,
    height_mm       REAL NOT NULL,
    power_w         REAL NOT NULL,
    lumens          REAL NOT NULL,
    angle_deg       REAL NOT NULL,
    voltage         REAL NOT NULL,
    min_pitch_mm    REAL NOT NULL,
    min_depth_mm    REAL NOT NULL,
    unit_price      REAL,
    light_model_json TEXT NOT NULL,
    plugin_id       TEXT,
    PRIMARY KEY (id, catalog_version_id)
);

CREATE TABLE power_supplies (
    id              TEXT NOT NULL,
    catalog_version_id TEXT NOT NULL REFERENCES catalog_versions(id),
    voltage         REAL NOT NULL,
    max_power_w     REAL NOT NULL,
    max_current_a   REAL NOT NULL,
    unit_price      REAL,
    derating_factor REAL NOT NULL DEFAULT 0.8,
    PRIMARY KEY (id, catalog_version_id)
);

-- Документы
CREATE TABLE documents (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    doc_type        TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    generated_at    TEXT NOT NULL
);

-- Настройки приложения
CREATE TABLE app_settings (
    key             TEXT PRIMARY KEY,
    value_json      TEXT NOT NULL
);

CREATE TABLE recent_files (
    path            TEXT PRIMARY KEY,
    opened_at       TEXT NOT NULL
);
```

---

## 3. Индексы и миграции

- Версионирование схемы: `schema_migrations (version, applied_at)`.
- Миграции — SQL-файлы `migrations/00N_description.sql`.
- При открытии `.leds` — синхронизация SQLite ↔ ZIP (источник истины — файл проекта).

---

## 4. Кэш и производительность

| Данные | Хранение |
|--------|----------|
| Distance field | Временный файл / память, не в SQLite |
| Medial axis | Кэш в `project.json` optional debug |
| Heatmap PNG | `simulation_runs.heatmap_path` |
| Импорт hash | `projects` + dedup повторного импорта |

---

## 5. Резервное копирование

- Пользователь копирует `.leds` + папку `Documents/LEDS`.
- Опционально: экспорт «архив проекта» (zip всего связанного).
