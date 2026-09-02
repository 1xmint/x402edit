#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct RecraftAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for RecraftAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Recraft,
                model: "recraftv4_1".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:recraft:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit, Operation::Design],
                    max_references: 3,
                    supports_masks: true,
                    supports_vector: true,
                    supports_transparency: true,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg, OutputFormat::Webp],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["external.api.recraft.ai".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for RecraftAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://external.api.recraft.ai/v1/images/generations",
            RequestEncoding::Json,
            json!({"model": self.descriptor.model, "prompt": intent.prompt.value, "n": 1, "response_format": "url"}),
            None,
        )
    }
}
