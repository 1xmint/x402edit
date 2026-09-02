#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct BflAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for BflAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Bfl,
                model: "flux-2-pro".into(),
                lifecycle: ProviderLifecycle::FixedSnapshot,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:bfl:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit],
                    max_references: 8,
                    supports_masks: false,
                    supports_vector: false,
                    supports_transparency: false,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK, Resolution::FourK],
                },
                egress_hosts: vec!["api.bfl.ai".into(), "delivery-eu1.bfl.ai".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for BflAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://api.bfl.ai/v1/flux-2-pro",
            RequestEncoding::Json,
            json!({"prompt": intent.prompt.value, "width": intent.canvas.width, "height": intent.canvas.height, "output_format": "png"}),
            None,
        )
    }
}
