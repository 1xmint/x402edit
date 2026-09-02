#![forbid(unsafe_code)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use x402edit_domain::{
    MAX_PROVIDER_ATTEMPTS, MAX_REPAIRS, MAX_WALL_TIME_SECONDS, Operation, PrivacyMode,
};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueSource {
    Explicit,
    Inferred,
    Default,
    Provider,
    Derived,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sourced<T> {
    pub value: T,
    pub source: ValueSource,
    pub confidence: f32,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

impl<T> Sourced<T> {
    pub fn explicit(value: T) -> Self {
        Self {
            value,
            source: ValueSource::Explicit,
            confidence: 1.0,
            evidence_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub units: String,
    pub origin: String,
    pub y_axis: String,
    pub color_space: String,
    pub background: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRole {
    SubjectIdentity,
    Product,
    Style,
    Composition,
    Palette,
    Source,
    Mask,
    Artifact,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntentReference {
    pub id: String,
    pub asset_id: String,
    pub role: ReferenceRole,
    pub region: Option<[f64; 4]>,
    pub strength: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtectedText {
    pub id: String,
    pub exact_utf8: String,
    pub role: String,
    pub language: String,
    pub must_be_exact: bool,
    #[serde(default)]
    pub invariants: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEditOperation {
    Replace,
    Remove,
    Recolor,
    Relight,
    Move,
    Resize,
    Retouch,
    BackgroundReplace,
    CanvasExtend,
    TextSet,
    TextStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityPolicy {
    RequireInput,
    Fail,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditTarget {
    pub node_id: Option<String>,
    pub semantic_query: Option<String>,
    pub region: Option<[f64; 4]>,
    pub expected_cardinality: u8,
    pub minimum_confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SemanticEdit {
    pub op: SemanticEditOperation,
    pub target: EditTarget,
    pub value: Value,
    #[serde(default)]
    pub invariants: Vec<String>,
    pub on_ambiguity: AmbiguityPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VisualIntent {
    pub schema_version: String,
    pub operation: Operation,
    pub canvas: Canvas,
    pub prompt: Sourced<String>,
    #[serde(default)]
    pub subjects: Vec<Sourced<String>>,
    pub composition: Sourced<String>,
    pub style: Sourced<String>,
    #[serde(default)]
    pub literal_text: Vec<ProtectedText>,
    #[serde(default)]
    pub references: Vec<IntentReference>,
    #[serde(default)]
    pub edits: Vec<SemanticEdit>,
    #[serde(default)]
    pub negative_constraints: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<String>,
    pub privacy_mode: PrivacyMode,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub unresolved: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowOperation {
    NormalizeInput,
    InterpretIntent,
    ResolveTarget,
    GenerateRaster,
    EditRaster,
    Segment,
    Compose,
    RenderDocument,
    Validate,
    Repair,
    EncryptResult,
    Purge,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStep {
    pub id: String,
    pub op: WorkflowOperation,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    pub budget_cost_atomic: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBudgets {
    pub max_provider_attempts: u8,
    pub max_repairs: u8,
    pub max_wall_time_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub schema_version: String,
    pub steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub invariants: Vec<String>,
    pub budgets: WorkflowBudgets,
    pub output_contract: Value,
}

impl Workflow {
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.schema_version != "1" {
            return Err(WorkflowError::UnsupportedVersion(
                self.schema_version.clone(),
            ));
        }
        if self.budgets.max_provider_attempts > MAX_PROVIDER_ATTEMPTS {
            return Err(WorkflowError::ProviderAttemptBudget);
        }
        if self.budgets.max_repairs > MAX_REPAIRS {
            return Err(WorkflowError::RepairBudget);
        }
        if self.budgets.max_wall_time_seconds > MAX_WALL_TIME_SECONDS {
            return Err(WorkflowError::WallTimeBudget);
        }
        let mut ids = std::collections::HashSet::new();
        for step in &self.steps {
            if step.id.is_empty() || !ids.insert(&step.id) {
                return Err(WorkflowError::DuplicateOrEmptyStep(step.id.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("unsupported workflow version {0}")]
    UnsupportedVersion(String),
    #[error("provider attempt budget exceeds the hard limit")]
    ProviderAttemptBudget,
    #[error("repair budget exceeds the hard limit")]
    RepairBudget,
    #[error("wall time budget exceeds the hard limit")]
    WallTimeBudget,
    #[error("workflow step id is empty or duplicated: {0}")]
    DuplicateOrEmptyStep(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excessive_provider_budget_is_rejected() {
        let workflow = Workflow {
            schema_version: "1".into(),
            steps: vec![],
            invariants: vec![],
            budgets: WorkflowBudgets {
                max_provider_attempts: 3,
                max_repairs: 0,
                max_wall_time_seconds: 10,
            },
            output_contract: Value::Null,
        };
        assert!(matches!(
            workflow.validate(),
            Err(WorkflowError::ProviderAttemptBudget)
        ));
    }
}
