use crate::errors::{GatekeeperError, Result};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum LbPolicy {
    RoundRobin,
    WeightedRoundRobin,
    LeastPending,
}

impl Default for LbPolicy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 100,
            jitter: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct KeyConfig {
    pub id: String,
    pub value: String,
    pub provider: String,
    pub weight: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Default)]
pub struct Config {
    pub provider: Option<String>,
    pub keys: Vec<KeyConfig>,
    #[serde(default)]
    pub lb_policy: LbPolicy,
    #[serde(default)]
    pub retry: RetryConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let provider = std::env::var("GATEKEEPER_PROVIDER").ok();

        let lb_policy = match std::env::var("GATEKEEPER_LB_POLICY")
            .unwrap_or_else(|_| "RR".to_string())
            .to_uppercase()
            .as_str()
        {
            "RR" | "ROUND_ROBIN" => LbPolicy::RoundRobin,
            "WRR" | "WEIGHTED_ROUND_ROBIN" => LbPolicy::WeightedRoundRobin,
            "LEAST_PENDING" | "LP" => LbPolicy::LeastPending,
            other => {
                return Err(GatekeeperError::InvalidConfig(format!(
                    "unknown lb policy: {}",
                    other
                )))
            }
        };

        let retry = RetryConfig {
            max_retries: std::env::var("GATEKEEPER_RETRY_MAX")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            base_delay_ms: std::env::var("GATEKEEPER_RETRY_BASE_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            jitter: std::env::var("GATEKEEPER_RETRY_JITTER")
                .ok()
                .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
        };

        let keys = match std::env::var("GATEKEEPER_KEYS_JSON") {
            Ok(json) if !json.trim().is_empty() => {
                let parsed: Vec<KeyConfig> = serde_json::from_str(&json).map_err(|e| {
                    GatekeeperError::InvalidConfig(format!(
                        "invalid GATEKEEPER_KEYS_JSON: {}",
                        e
                    ))
                })?;
                parsed
            }
            _ => {
                let raw = std::env::var("GATEKEEPER_KEYS").unwrap_or_default();
                let default_provider = provider.clone().unwrap_or_else(|| "default".to_string());
                let mut keys = Vec::new();
                for (i, part) in raw
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .enumerate()
                {
                    let value = part.to_string();
                    let id = format!("k{}", i + 1);
                    keys.push(KeyConfig {
                        id,
                        value,
                        provider: default_provider.clone(),
                        weight: 1,
                    });
                }
                keys
            }
        };

        if keys.is_empty() {
            return Err(GatekeeperError::InvalidConfig(
                "no API keys configured (GATEKEEPER_KEYS or GATEKEEPER_KEYS_JSON)".to_string(),
            ));
        }

        if let Some(p) = &provider {
            // Ensure keys providers are consistent if provider is globally specified
            if keys.iter().any(|k| k.provider != *p) {
                return Err(GatekeeperError::InvalidConfig(
                    "provider mismatch between global provider and per-key settings".to_string(),
                ));
            }
        }

        Ok(Self {
            provider,
            keys,
            lb_policy,
            retry,
        })
    }

    pub fn redacted(&self) -> Self {
        let mut cloned = self.clone();
        for k in &mut cloned.keys {
            k.value = "***".into();
        }
        cloned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_env() {
        for (k, _) in std::env::vars() {
            if k.starts_with("GATEKEEPER_") {
                std::env::remove_var(k);
            }
        }
    }

    #[test]
    fn parse_keys_from_csv() {
        reset_env();
        std::env::set_var("GATEKEEPER_KEYS", "a,b,c");
        std::env::set_var("GATEKEEPER_PROVIDER", "openai");
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.keys.len(), 3);
        assert_eq!(cfg.keys[0].provider, "openai");
        assert_eq!(cfg.lb_policy, LbPolicy::RoundRobin);
        assert_eq!(cfg.retry.max_retries, 2);
        assert!(cfg.retry.jitter);
    }

    #[test]
    fn parse_keys_from_json() {
        reset_env();
        let json = serde_json::json!([
            {"id": "k1", "value": "abc", "provider": "anthropic", "weight": 2},
            {"id": "k2", "value": "def", "provider": "anthropic", "weight": 1}
        ]);
        std::env::set_var("GATEKEEPER_KEYS_JSON", json.to_string());
        std::env::set_var("GATEKEEPER_LB_POLICY", "WRR");
        std::env::set_var("GATEKEEPER_RETRY_MAX", "3");
        std::env::set_var("GATEKEEPER_RETRY_BASE_MS", "250");
        std::env::set_var("GATEKEEPER_RETRY_JITTER", "false");
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.keys.len(), 2);
        assert_eq!(cfg.lb_policy, LbPolicy::WeightedRoundRobin);
        assert_eq!(cfg.retry.max_retries, 3);
        assert_eq!(cfg.retry.base_delay_ms, 250);
        assert!(!cfg.retry.jitter);
    }

    #[test]
    fn invalid_lb_policy() {
        reset_env();
        std::env::set_var("GATEKEEPER_KEYS", "a");
        std::env::set_var("GATEKEEPER_LB_POLICY", "unknown");
        let err = Config::from_env().unwrap_err();
        assert!(matches!(
            err,
            crate::errors::GatekeeperError::InvalidConfig(_)
        ));
    }
}
