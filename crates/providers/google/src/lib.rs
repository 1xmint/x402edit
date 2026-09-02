#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct GoogleAdapter {
    descriptor: ProviderDescriptor,
}

impl Default for GoogleAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Google,
                model: "gemini-3.1-flash-image".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:google:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit, Operation::Design],
                    max_references: 14,
                    supports_masks: false,
                    supports_vector: false,
                    supports_transparency: true,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg],
                    resolutions: vec![
                        Resolution::HalfK,
                        Resolution::OneK,
                        Resolution::TwoK,
                        Resolution::FourK,
                    ],
                },
                egress_hosts: vec!["generativelanguage.googleapis.com".into()],
            },
        }
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for GoogleAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-image:generateContent",
            RequestEncoding::Json,
            json!({
                "contents": [{"role": "user", "parts": [{"text": intent.prompt.value}]}],
                "generationConfig": {"responseModalities": ["TEXT", "IMAGE"]},
                "store": false
            }),
            None,
        )
    }
}
