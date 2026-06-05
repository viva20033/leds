mod error;
mod types;

pub use error::{CatalogError, Result};
pub use types::*;

use std::path::{Path, PathBuf};

/// Load catalog manifest and all referenced modules from a directory.
pub fn load_catalog(dir: impl AsRef<Path>) -> Result<Catalog> {
    let dir = dir.as_ref();
    let manifest_path = dir.join("manifest.json");
    let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|e| {
        CatalogError::Io {
            path: manifest_path.display().to_string(),
            source: e,
        }
    })?;
    let manifest: CatalogManifest =
        serde_json::from_str(&manifest_text).map_err(CatalogError::Json)?;

    let mut modules = Vec::with_capacity(manifest.modules.len());
    for rel in &manifest.modules {
        let path = dir.join(rel);
        let text = std::fs::read_to_string(&path).map_err(|e| CatalogError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        let module: LedModule =
            serde_json::from_str(&text).map_err(CatalogError::Json)?;
        modules.push(module);
    }

    Ok(Catalog { manifest, modules })
}

/// Resolve default catalog path relative to workspace / executable.
pub fn default_catalog_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../catalog")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("catalog"))
}

/// Pick module id by sign depth using manifest defaults.
pub fn module_for_depth(catalog: &Catalog, depth_mm: f64) -> Option<&LedModule> {
    let rules = catalog.manifest.defaults.as_ref()?.preferred_module_by_depth.as_ref()?;
    let mut chosen = None;
    for rule in rules {
        if depth_mm <= rule.depth_max_mm {
            chosen = Some(rule.module_id.as_str());
            break;
        }
    }
    let id = chosen?;
    catalog.modules.iter().find(|m| m.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_elf_catalog() {
        let catalog = load_catalog(default_catalog_path()).expect("catalog");
        assert_eq!(catalog.modules.len(), 3);
        assert!(catalog.modules.iter().any(|m| m.pricing.unit_price == Some(43.0)));
    }

    #[test]
    fn picks_module_by_depth() {
        let catalog = load_catalog(default_catalog_path()).unwrap();
        assert_eq!(
            module_for_depth(&catalog, 55.0).unwrap().id,
            "elf.sol-plus-dot.1smd-2835.warm"
        );
        assert_eq!(
            module_for_depth(&catalog, 100.0).unwrap().id,
            "elf.sol-plus.2smd-2835.white"
        );
        assert_eq!(
            module_for_depth(&catalog, 150.0).unwrap().id,
            "elf.sol-plus.3smd-2835.cold"
        );
    }
}
