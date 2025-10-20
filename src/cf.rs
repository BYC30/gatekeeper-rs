#![cfg(target_arch = "wasm32")]

use worker::*;

fn log_request(req: &Request) {
    let path = req
        .url()
        .ok()
        .map(|u| u.pathname().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
    console_log!("{} {}", req.method().to_string(), path);
}

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    log_request(&req);

    Router::new()
        .get("/healthz", |_req, _| Response::ok("ok"))
        .get("/readyz", |_req, _| Response::ok("ok"))
        .get("/", |_req, _| Response::ok("gatekeeper-rs"))
        .run(req, env)
        .await
}
