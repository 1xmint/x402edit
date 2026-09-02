#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct StabilityAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for StabilityAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Stability,
                model: "stable-image-ultra".into(),
                lifecycle: ProviderLifecycle::MutableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:stability:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit],
                    max_references: 1,
                    supports_masks: true,
                    supports_vector: false,
                    supports_transparency: true,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg, OutputFormat::Webp],
                    resolutions: vec![Resolution::OneK],
                },
                egress_hosts: vec!["api.stability.ai".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for StabilityAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        let endpoint = if intent.operation == Operation::Edit {
            "https://api.stability.ai/v2beta/stable-image/edit/inpaint"
        } else {
            "https://api.stability.ai/v2beta/stable-image/generate/ultra"
        };
        compiled_request(
            endpoint,
            RequestEncoding::Multipart,
            json!({"prompt": intent.prompt.value, "output_format": "png"}),
            None,
        )
    }
}
