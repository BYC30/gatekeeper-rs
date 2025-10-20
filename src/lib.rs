/**
A high-performance, fault-tolerant proxy for AI models like Codex and Claude, targeting Cloudflare Workers.

Refer to docs/ for architecture, milestones, and tasks.
*/

pub mod config;
pub mod errors;
pub mod keypool;
pub mod lb;

#[cfg(target_arch = "wasm32")]
pub mod cf;

use config::Config;
use errors::Result;
use lb::LoadBalancer;

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub lb: LoadBalancer,
}

impl App {
    pub fn from_env() -> Result<Self> {
        let cfg = Config::from_env()?;
        let lb = LoadBalancer::new(cfg.lb_policy.clone(), &cfg.keys);
        Ok(Self { config: cfg, lb })
    }

    pub fn redacted_config(&self) -> Config {
        self.config.redacted()
    }
}
