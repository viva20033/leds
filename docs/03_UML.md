# UML-модель системы LEDS

Диаграммы в формате Mermaid (совместимы с GitHub, VS Code, Cursor).

---

## 1. Диаграмма вариантов использования

```mermaid
flowchart LR
    Manager((Менеджер))
    Tech((Технолог))
    Admin((Администратор))

    subgraph System["LEDS"]
        UC1[Импорт SVG/DXF]
        UC2[Настройка параметров изделия]
        UC3[Авторазмещение]
        UC4[Полуавтоматическое редактирование]
        UC5[Экспертное размещение]
        UC6[Симуляция засветки]
        UC7[Расчёт БП и мощности]
        UC8[Формирование документов]
        UC9[Управление каталогом модулей]
    end

    Manager --> UC1
    Manager --> UC3
    Manager --> UC8
    Tech --> UC2
    Tech --> UC4
    Tech --> UC5
    Tech --> UC6
    Tech --> UC7
    Admin --> UC9
```

---

## 2. Диаграмма классов (доменная модель)

```mermaid
classDiagram
    class Project {
        +UUID id
        +String name
        +ProductParams params
        +DateTime updatedAt
    }

    class ProductParams {
        +float depthMm
        +float rimWidthMm
        +DiffuserType diffuser
        +float diffuserTransmission
    }

    class Layer {
        +String id
        +String name
        +bool visible
        +bool locked
    }

    class ContourRing {
        +RingKind kind
        +Vec~Point~ vertices
        +bool closed
    }

    class LedModule {
        +String catalogId
        +Footprint footprint
        +ElectricalSpec electrical
        +LightModel lightModel
        +PlacementHints hints
    }

    class Placement {
        +UUID id
        +String moduleId
        +float x
        +float y
        +float angleDeg
        +bool fixed
        +int chainId
    }

    class LightModel {
        +ModelType type
        +float angleDeg
        +float sigmaMm
        +float peakIntensity
    }

    class SimulationResult {
        +float minIlluminance
        +float maxIlluminance
        +float uniformityIndex
        +List~Alert~ alerts
    }

    class PowerGroup {
        +int chainId
        +List~UUID~ placementIds
        +float currentA
    }

    class PowerSupplyUnit {
        +String catalogId
        +float voltage
        +float maxPowerW
    }

    class DocumentArtifact {
        +DocumentType type
        +String filePath
        +DateTime generatedAt
    }

    Project "1" --> "*" Layer
    Project "1" --> "*" ContourRing
    Project "1" --> "*" Placement
    Project "1" --> ProductParams
    Project "1" --> "0..1" SimulationResult
    Project "1" --> "*" PowerGroup
    Placement --> LedModule : references
    LedModule --> LightModel
    PowerGroup --> PowerSupplyUnit : assigned
    Project "1" --> "*" DocumentArtifact
```

---

## 3. Диаграмма компонентов

```mermaid
flowchart TB
    subgraph UI_Comp["ui-desktop"]
        Viewport[ViewportController]
        PropertyPanel[PropertyPanel]
        LayerList[LayerList]
        ModuleLibrary[ModuleLibrary]
        HeatmapView[HeatmapOverlay]
    end

    subgraph App_Comp["application"]
        ProjectService[ProjectService]
        CommandBus[CommandBus]
        UndoStack[UndoStack]
    end

    subgraph Core_Comp["leds-core"]
        Importer[Importer]
        Topology[TopologyBuilder]
        OffsetEngine[OffsetEngine]
        MedialAxis[MedialAxisExtractor]
        Placer[HybridPlacer]
        Optimizer[PlacementOptimizer]
        Simulator[LightingSimulator]
        PowerPlanner[PowerPlanner]
        PdfGen[PdfGenerator]
    end

    subgraph Data_Comp["leds-storage"]
        SqlRepo[SqliteRepository]
        CatalogRepo[CatalogRepository]
    end

    Viewport --> CommandBus
    PropertyPanel --> CommandBus
    CommandBus --> ProjectService
    ProjectService --> Importer
    ProjectService --> Placer
    ProjectService --> Simulator
    Placer --> Topology
    Placer --> MedialAxis
    Placer --> OffsetEngine
    Placer --> Optimizer
    Simulator --> Placer
    ProjectService --> SqlRepo
    ProjectService --> CatalogRepo
    ProjectService --> PdfGen
```

---

## 4. Диаграмма последовательности — автоматическое размещение

```mermaid
sequenceDiagram
    actor User
    participant UI as React UI
    participant App as ProjectService
    participant Geo as Geometry Engine
    participant Pl as Placement Engine
    participant Sim as Lighting Simulator

    User->>UI: Выбрать модуль + Авторазмещение
    UI->>App: run_placement(Auto, moduleId)
    App->>Geo: buildTopology(contours)
    Geo-->>App: rings, safeZone
    App->>Geo: extractMedialAxis(safeZone)
    Geo-->>App: skeletonGraph
    App->>Pl: placeHybrid(skeleton, module, params)
    Pl->>Pl: seedFromAxis + fillGaps + optimize
    Pl-->>App: placements[], metrics
    App->>Sim: simulate(placements, lightModel)
    Sim-->>App: heatmap, alerts[]
    App-->>UI: result + warnings
    UI-->>User: Отображение модулей и карты
```

---

## 5. Диаграмма состояний — режим размещения

```mermaid
stateDiagram-v2
    [*] --> Empty
    Empty --> AutoPlaced : run_auto
    AutoPlaced --> SemiAuto : user_edit
    SemiAuto --> SemiAuto : move/add/delete/fix
    SemiAuto --> Recalculating : simulate
    Recalculating --> SemiAuto : done
    AutoPlaced --> Expert : switch_expert
    Expert --> Expert : manual_only
    Expert --> Recalculating : simulate
    SemiAuto --> AutoPlaced : rerun_auto_keep_fixed
    Expert --> SemiAuto : import_from_auto
```

---

## 6. Диаграмма активности — гибридный pipeline размещения

```mermaid
flowchart TD
    Start([Старт]) --> Import[Импорт и топология]
    Import --> Offset[Offset: борт + габарит]
    Offset --> Mask[Растеризация маски]
    Mask --> DT[Distance Transform]
    Mask --> MAT[Medial Axis / Voronoi]
    DT --> Classify[Классификация зон: тонкие / широкие / углы]
    MAT --> Seed[Начальные позиции вдоль скелета]
    Classify --> Seed
    Seed --> Poisson[Poisson-подобное уплотнение с min pitch]
    Poisson --> Check{Порог равномерности?}
    Check -->|Нет| Fill[Добавить в зоны недосвета DT]
    Fill --> Local[Локальная оптимизация: удалить пересвет]
    Local --> Check
    Check -->|Да| Validate[Проверка границ и коллизий]
    Validate --> End([Готово])
```

---

## 7. Диаграмма развёртывания

```mermaid
flowchart LR
    subgraph Workstation["ПК пользователя Windows"]
        App[LEDS.exe / Tauri]
        CoreLib[leds_core.dll]
        SQLite[(SQLite local)]
        Files[(.leds projects)]
        Catalog[(catalog packages)]
    end

    subgraph OptionalCloud["Опционально"]
        CDN[Catalog CDN]
        License[License Server]
    end

    App --> CoreLib
    App --> SQLite
    App --> Files
    App --> Catalog
    App -.-> CDN
    App -.-> License
```

---

## 8. Пакетная диаграмма (Rust crates)

```mermaid
flowchart TB
    leds_app["leds-app (Tauri binary)"]
    leds_core["leds-core"]
    leds_geom["leds-geometry"]
    leds_place["leds-placement"]
    leds_light["leds-lighting"]
    leds_import["leds-import"]
    leds_elec["leds-electrical"]
    leds_docs["leds-documents"]
    leds_storage["leds-storage"]
    leds_catalog["leds-catalog"]
    leds_plugin["leds-plugin-api"]

    leds_app --> leds_core
    leds_core --> leds_geom
    leds_core --> leds_place
    leds_core --> leds_light
    leds_core --> leds_import
    leds_core --> leds_elec
    leds_core --> leds_docs
    leds_core --> leds_storage
    leds_place --> leds_geom
    leds_place --> leds_catalog
    leds_light --> leds_catalog
    leds_light --> leds_geom
    leds_import --> leds_geom
    leds_core --> leds_plugin
```
