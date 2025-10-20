//! A high-performance, fault-tolerant proxy for AI models like Codex and Claude, targeting Cloudflare Workers.
//!
//! Refer to the docs in the `docs/` directory for architecture, roadmap, and task breakdown.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use worker::*;

        #[event(fetch)]
        pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
            Router::new()
                .get("/healthz", |_, _| Response::ok("ok"))
                .get("/readyz", |_, _| Response::ok("ok"))
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
