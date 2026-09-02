#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use x402edit_domain::{ArtifactId, VersionId};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditabilityLevel {
    ExactStructured,
    SemanticRaster,
    FlatRasterWithRegions,
    OpaqueFlatRaster,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Editability {
    pub level: EditabilityLevel,
    pub supported_operations: Vec<String>,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub schema_version: String,
    pub artifact_id: ArtifactId,
    pub version_id: VersionId,
    pub parent_version_id: Option<VersionId>,
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    Group {
        id: String,
        children: Vec<String>,
        editability: Editability,
    },
    Raster {
        id: String,
        asset_id: String,
        editability: Editability,
    },
    Text {
        id: String,
        exact_utf8: String,
        editability: Editability,
    },
    Shape {
        id: String,
        path: String,
        editability: Editability,
    },
    Mask {
        id: String,
        asset_id: String,
        editability: Editability,
    },
}
pub fn canonical_coordinate(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
