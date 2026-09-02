#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::{
    CompiledProviderRequest, PrivacyClass, ProviderAdapter, ProviderCapabilities,
    ProviderDescriptor, ProviderError, ProviderId, ProviderLifecycle, RequestEncoding,
    compiled_request, ensure_operation,
};
use x402edit_workflow::VisualIntent;

pub struct OpenAiAdapter {
    descriptor: ProviderDescriptor,
}

impl Default for OpenAiAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::OpenAi,
                model: "gpt-image-2-2026-04-21".into(),
                lifecycle: ProviderLifecycle::FixedSnapshot,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:openai:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit],
                    max_references: 14,
                    supports_masks: true,
                    supports_vector: false,
                    supports_transparency: true,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png, OutputFormat::Jpeg, OutputFormat::Webp],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["api.openai.com".into()],
            },
        }
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for OpenAiAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }

    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        let endpoint = match intent.operation {
            Operation::Generate | Operation::Design => {
                "https://api.openai.com/v1/images/generations"
            }
            Operation::Edit => "https://api.openai.com/v1/images/edits",
        };
        let encoding = if intent.operation == Operation::Edit {
            RequestEncoding::Multipart
        } else {
            RequestEncoding::Json
        };
        compiled_request(
            endpoint,
            encoding,
            json!({
                "model": self.descriptor.model,
                "prompt": intent.prompt.value,
                "n": 1,
                "size": format!("{}x{}", intent.canvas.width, intent.canvas.height),
                "output_format": "png"
            }),
            Some("Idempotency-Key"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pins_documented_snapshot() {
        assert_eq!(
            OpenAiAdapter::default().descriptor().model,
            "gpt-image-2-2026-04-21"
        );
    }
}
