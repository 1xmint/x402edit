#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct IdeogramAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for IdeogramAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Ideogram,
                model: "ideogram-v4".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:ideogram:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit, Operation::Design],
                    max_references: 4,
                    supports_masks: true,
                    supports_vector: false,
                    supports_transparency: true,
                    supports_structured_prompt: true,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["api.ideogram.ai".into(), "storage.googleapis.com".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for IdeogramAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://api.ideogram.ai/v1/ideogram-v4/generate",
            RequestEncoding::Multipart,
            json!({"text_prompt": intent.prompt.value, "rendering_speed": "DEFAULT", "enable_copyright_detection": true}),
            None,
        )
    }
}
