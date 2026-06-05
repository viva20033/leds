use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub defaults: Option<CatalogDefaults>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogDefaults {
    #[serde(rename = "preferredModuleByDepth")]
    pub preferred_module_by_depth: Option<Vec<DepthRule>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DepthRule {
    #[serde(rename = "depthMaxMm")]
    pub depth_max_mm: f64,
    #[serde(rename = "moduleId")]
    pub module_id: String,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub manifest: CatalogManifest,
    pub modules: Vec<LedModule>,
}

impl Catalog {
    pub fn get(&self, id: &str) -> Option<&LedModule> {
        self.modules.iter().find(|m| m.id == id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LedModule {
    pub id: String,
    pub vendor: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub footprint: Footprint,
    pub electrical: Electrical,
    #[serde(default)]
    pub color: Option<Color>,
    pub optics: Optics,
    pub placement: PlacementHints,
    pub chain: ChainLimits,
    #[serde(default)]
    pub pricing: Pricing,
    #[serde(rename = "lightModel")]
    pub light_model: LightModel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Footprint {
    #[serde(rename = "lengthMm")]
    pub length_mm: f64,
    #[serde(rename = "widthMm")]
    pub width_mm: f64,
    #[serde(rename = "heightMm")]
    pub height_mm: f64,
    #[serde(default)]
    pub orientation: Option<String>,
}

impl Footprint {
    pub fn max_extent_mm(&self) -> f64 {
        self.length_mm.max(self.width_mm)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Electrical {
    #[serde(rename = "voltageV")]
    pub voltage_v: f64,
    #[serde(rename = "powerW")]
    pub power_w: f64,
    pub lumens: f64,
    #[serde(default)]
    pub current_a: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Color {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BeamAngle {
    Symmetric {
        degrees: f64,
    },
    Asymmetric {
        #[serde(rename = "horizontalDeg")]
        horizontal_deg: f64,
        #[serde(rename = "verticalDeg")]
        vertical_deg: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Optics {
    #[serde(rename = "beamAngle")]
    pub beam_angle: BeamAngle,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlacementHints {
    #[serde(rename = "depthMinMm")]
    pub depth_min_mm: f64,
    #[serde(rename = "depthMaxMm")]
    pub depth_max_mm: f64,
    #[serde(rename = "recommendedPitchMm")]
    pub recommended_pitch_mm: f64,
    #[serde(rename = "minPitchMm")]
    pub min_pitch_mm: f64,
    #[serde(rename = "maxPitchMm", default)]
    pub max_pitch_mm: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainLimits {
    #[serde(rename = "maxCenterDistanceMm")]
    pub max_center_distance_mm: f64,
    #[serde(rename = "maxModulesInSeries")]
    pub max_modules_in_series: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Pricing {
    #[serde(rename = "unitPrice")]
    pub unit_price: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightModel {
    #[serde(rename = "type")]
    pub model_type: String,
    pub params: serde_json::Value,
}
