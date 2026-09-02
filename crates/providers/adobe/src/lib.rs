#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct AdobeAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for AdobeAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Adobe,
                model: "firefly_image".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:adobe:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit],
                    max_references: 4,
                    supports_masks: false,
                    supports_vector: false,
                    supports_transparency: false,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["firefly-api.adobe.io".into(), "adobe.io".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for AdobeAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://firefly-api.adobe.io/v4/images/generate-async",
            RequestEncoding::Json,
            json!({"prompt": intent.prompt.value, "modelId": self.descriptor.model, "numVariations": 1,
                "referenceBlobs": [], "modelSpecificPayload": {"localeCode": "en-US", "prompt_reasoner": "quality"}}),
            None,
        )
    }
}
