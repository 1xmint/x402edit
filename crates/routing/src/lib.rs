#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use x402edit_domain::{Operation, PrivacyMode, QualityProfile};
use x402edit_provider_core::{PrivacyClass, ProviderDescriptor, ProviderId};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub quality: f64,
    pub reliability: f64,
    pub latency: f64,
    pub price: f64,
}

#[derive(Clone, Debug)]
pub struct RouteRequest {
    pub operation: Operation,
    pub reference_count: u8,
    pub requires_vector: bool,
    pub quality_profile: QualityProfile,
    pub privacy_mode: PrivacyMode,
    pub accepted_policy_digests: HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct RouteDecision {
    pub provider: ProviderId,
    pub model: String,
    pub policy_digest: String,
    pub score: f64,
}

pub fn select_provider(
    request: &RouteRequest,
    providers: &[ProviderDescriptor],
    metrics: &HashMap<ProviderId, ProviderMetrics>,
) -> Result<RouteDecision, RoutingError> {
    providers
        .iter()
        .filter(|provider| privacy_allows(request, provider))
        .filter(|provider| {
            provider
                .capabilities
                .operations
                .contains(&request.operation)
        })
        .filter(|provider| provider.capabilities.max_references >= request.reference_count)
        .filter(|provider| !request.requires_vector || provider.capabilities.supports_vector)
        .filter_map(|provider| {
            metrics.get(&provider.id).map(|m| RouteDecision {
                provider: provider.id,
                model: provider.model.clone(),
                policy_digest: provider.policy_digest.clone(),
                score: score(request.quality_profile, *m),
            })
        })
        .max_by(|left, right| left.score.total_cmp(&right.score))
        .ok_or(RoutingError::NoEligibleProvider)
}

fn privacy_allows(request: &RouteRequest, provider: &ProviderDescriptor) -> bool {
    match (request.privacy_mode, provider.privacy_class) {
        (_, PrivacyClass::Disabled) => false,
        (PrivacyMode::StrictEphemeral, PrivacyClass::StrictEligible) => true,
        (PrivacyMode::StrictEphemeral, PrivacyClass::ConsentOnly) => false,
        (PrivacyMode::ProviderConsent, PrivacyClass::StrictEligible) => true,
        (PrivacyMode::ProviderConsent, PrivacyClass::ConsentOnly) => request
            .accepted_policy_digests
            .contains(&provider.policy_digest),
    }
}

fn score(profile: QualityProfile, metrics: ProviderMetrics) -> f64 {
    let (quality, reliability, latency, price) = match profile {
        QualityProfile::Quality => (0.75, 0.15, 0.05, 0.05),
        QualityProfile::Balanced => (0.60, 0.15, 0.15, 0.10),
        QualityProfile::Economy => (0.40, 0.15, 0.15, 0.30),
    };
    quality * metrics.quality
        + reliability * metrics.reliability
        + latency * metrics.latency
        + price * metrics.price
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("no provider satisfies the capability and privacy gates")]
    NoEligibleProvider,
}

#[cfg(test)]
mod tests {
    use super::*;
    use x402edit_domain::{OutputFormat, Resolution};
    use x402edit_provider_core::{ProviderCapabilities, ProviderLifecycle};

    fn provider(id: ProviderId, privacy_class: PrivacyClass, digest: &str) -> ProviderDescriptor {
        ProviderDescriptor {
            id,
            model: "test".into(),
            lifecycle: ProviderLifecycle::FixedSnapshot,
            privacy_class,
            policy_digest: digest.into(),
            capabilities: ProviderCapabilities {
                operations: vec![Operation::Generate],
                max_references: 2,
                supports_masks: false,
                supports_vector: false,
                supports_transparency: false,
                supports_structured_prompt: false,
                output_formats: vec![OutputFormat::Png],
                resolutions: vec![Resolution::OneK],
            },
            egress_hosts: vec![],
        }
    }

    #[test]
    fn strict_mode_excludes_consent_only_provider() {
        let providers = vec![provider(
            ProviderId::OpenAi,
            PrivacyClass::ConsentOnly,
            "p1",
        )];
        let metrics = HashMap::from([(
            ProviderId::OpenAi,
            ProviderMetrics {
                quality: 1.0,
                reliability: 1.0,
                latency: 1.0,
                price: 1.0,
            },
        )]);
        let request = RouteRequest {
            operation: Operation::Generate,
            reference_count: 0,
            requires_vector: false,
            quality_profile: QualityProfile::Balanced,
            privacy_mode: PrivacyMode::StrictEphemeral,
            accepted_policy_digests: HashSet::new(),
        };
        assert_eq!(
            select_provider(&request, &providers, &metrics).unwrap_err(),
            RoutingError::NoEligibleProvider
        );
    }

    #[test]
    fn consent_is_bound_to_policy_digest() {
        let providers = vec![provider(
            ProviderId::Ideogram,
            PrivacyClass::ConsentOnly,
            "p1",
        )];
        let metrics = HashMap::from([(
            ProviderId::Ideogram,
            ProviderMetrics {
                quality: 1.0,
                reliability: 1.0,
                latency: 1.0,
                price: 1.0,
            },
        )]);
        let request = RouteRequest {
            operation: Operation::Generate,
            reference_count: 0,
            requires_vector: false,
            quality_profile: QualityProfile::Balanced,
            privacy_mode: PrivacyMode::ProviderConsent,
            accepted_policy_digests: HashSet::from(["p1".into()]),
        };
        assert_eq!(
            select_provider(&request, &providers, &metrics)
                .unwrap()
                .provider,
            ProviderId::Ideogram
        );
    }
}
