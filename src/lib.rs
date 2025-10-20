//! A high-performance, fault-tolerant proxy for AI models like Codex and Claude, targeting Cloudflare Workers.
//!
//! Refer to the docs in the `docs/` directory for architecture, roadmap, and task breakdown.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use worker::*;

        #[derive(Clone, Copy, Debug)]
        enum LbPolicy {
            Rr,
        }

        impl core::fmt::Display for LbPolicy {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    LbPolicy::Rr => write!(f, "RR"),
                }
            }
        }

        #[derive(Clone, Debug)]
        struct Config {
            provider: String,
            upstream_base: String,
            keys: Vec<String>,
            lb_policy: LbPolicy,
        }

        impl Config {
            fn from_ctx(ctx: &RouteContext<()>) -> Result<Self> {
                let provider = ctx
                    .var("GK_PROVIDER")
                    .ok()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "openai".to_string());

                let upstream_base = ctx
                    .var("GK_UPSTREAM_BASE")
                    .ok()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "https://api.openai.com".to_string());

                let lb_policy = ctx
                    .var("GK_LB_POLICY")
                    .ok()
                    .map(|v| match v.to_string().to_uppercase().as_str() {
                        "RR" => LbPolicy::Rr,
                        _ => LbPolicy::Rr,
                    })
                    .unwrap_or(LbPolicy::Rr);

                let keys_secret = ctx.secret("GK_API_KEYS")?;
                let keys = keys_secret
                    .to_string()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();

                if keys.is_empty() {
                    return Err(Error::RustError("GK_API_KEYS is empty".into()));
                }

                console_log!(
                    "config loaded: provider={}, upstream_base={}, keys={}, lb_policy={}",
                    provider,
                    upstream_base,
                    keys.len(),
                    lb_policy
                );

                Ok(Self { provider, upstream_base, keys, lb_policy })
            }
        }

        #[event(fetch)]
        pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
            Router::new()
                .get("/healthz", |_, _| Response::ok("ok"))
                .get("/readyz", |_, ctx| {
                    let _cfg = Config::from_ctx(&ctx)?;
                    Response::ok("ok")
                })
                .get("/", |_, _| Response::ok("gatekeeper-rs"))
                .get_async("/v1/*path", |_req, _| async move { Response::ok("ok") })
                .run(req, env)
                .await
        }
    } else {
        // Non-wasm targets intentionally have no runtime entrypoint.
        // This allows cargo check/clippy on host while the Worker entrypoint
        // is compiled only for wasm32 by Cloudflare's build.
    }
}
