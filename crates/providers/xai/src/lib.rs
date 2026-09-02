#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct XaiAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for XaiAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Xai,
                model: "grok-imagine-image-2.0".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:xai:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit],
                    max_references: 5,
                    supports_masks: false,
                    supports_vector: false,
                    supports_transparency: false,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["api.x.ai".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for XaiAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        let endpoint = if intent.operation == Operation::Edit {
            "https://api.x.ai/v1/images/edits"
        } else {
            "https://api.x.ai/v1/images/generations"
        };
        compiled_request(
            endpoint,
            if intent.operation == Operation::Edit {
                RequestEncoding::Multipart
            } else {
                RequestEncoding::Json
            },
            json!({"model": self.descriptor.model, "prompt": intent.prompt.value, "n": 1, "response_format": "url"}),
            None,
        )
    }
}
