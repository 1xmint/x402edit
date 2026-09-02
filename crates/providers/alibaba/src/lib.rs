#![forbid(unsafe_code)]

use serde_json::json;
use x402edit_domain::{Operation, OutputFormat, Resolution};
use x402edit_provider_core::*;
use x402edit_workflow::VisualIntent;

pub struct AlibabaAdapter {
    descriptor: ProviderDescriptor,
}
impl Default for AlibabaAdapter {
    fn default() -> Self {
        Self {
            descriptor: ProviderDescriptor {
                id: ProviderId::Alibaba,
                model: "qwen-image-3.0-pro".into(),
                lifecycle: ProviderLifecycle::StableAlias,
                privacy_class: PrivacyClass::ConsentOnly,
                policy_digest: "unverified:alibaba:2026-09-01".into(),
                capabilities: ProviderCapabilities {
                    operations: vec![Operation::Generate, Operation::Edit, Operation::Design],
                    max_references: 3,
                    supports_masks: false,
                    supports_vector: false,
                    supports_transparency: false,
                    supports_structured_prompt: false,
                    output_formats: vec![OutputFormat::Png],
                    resolutions: vec![Resolution::OneK, Resolution::TwoK],
                },
                egress_hosts: vec!["dashscope-intl.aliyuncs.com".into()],
            },
        }
    }
}
#[async_trait::async_trait]
impl ProviderAdapter for AlibabaAdapter {
    fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }
    fn compile(&self, intent: &VisualIntent) -> Result<CompiledProviderRequest, ProviderError> {
        ensure_operation(&self.descriptor, intent)?;
        compiled_request(
            "https://dashscope-intl.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation",
            RequestEncoding::Json,
            json!({"model": self.descriptor.model, "input": {"messages": [{"role": "user", "content": [{"text": intent.prompt.value}]}]},
                "parameters": {"n": 1, "size": format!("{}*{}", intent.canvas.width, intent.canvas.height)}}),
            None,
        )
    }
}
