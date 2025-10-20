# gatekeeper-rs

[![crates.io](https://img.shields.io/crates/v/gatekeeper-rs.svg)](https://crates.io/crates/gatekeeper-rs)
[![docs.rs](https://docs.rs/gatekeeper-rs/badge.svg)](https://docs.rs/gatekeeper-rs)
[![Build Status](https://github.com/BYC30/gatekeeper-rs/workflows/CI/badge.svg)](https://github.com/BYC30/gatekeeper-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, fault-tolerant proxy for AI models like Codex and Claude, built with Rust and running on Cloudflare Workers. It provides load balancing across multiple API keys to maximize usage and ensure reliability.

一个基于 Rust 和 Cloudflare Worker 的高性能、高容错 AI 模型（如 CodeX/Claude）代理服务。它能通过对多个 API Key 进行负载均衡，以实现更高的用量和可靠性。

## ✨ 功能特性 (Features)

*   **多 Key 负载均衡 (Multi-Key Load Balancing)**: 将请求轮流或按权重分配给配置好的多个 API Key，有效绕开单一 Key 的速率限制。
*   **故障自动切换 (Automatic Failover)**: 当某个 Key 失效或达到用量上限时，能自动切换到下一个可用的 Key，保证服务的高可用性。
*   **高性能与低延迟 (High Performance & Low Latency)**: 基于 Rust 编写，并部署在 Cloudflare 的全球边缘网络上，为用户提供极速的响应体验。
*   **轻量级部署 (Lightweight Deployment)**: 无需管理自己的服务器，一行命令即可部署到 serverless 平台 Cloudflare Workers。
*   **易于配置 (Easy to Configure)**: 通过简单的环境变量或配置文件即可完成 API Keys 和路由策略的设置。
*   **可观测性 (Observability)**: (可选) 支持集成日志和监控，方便追踪每个 Key 的使用情况和请求状态。

## 🏗️ 架构 (Architecture)

`gatekeeper-rs` 作为一个 Cloudflare Worker 运行在 Cloudflare 的边缘节点上。当一个请求到达 Worker 时，它会执行以下操作：

1.  **拦截请求**: 捕获发送至指定路由的 API 请求。
2.  **选择密钥**: 从预先配置的密钥池中，根据负载均衡策略（例如：轮询）选择一个可用的 API Key。
3.  **转发请求**: 使用选定的 Key，将原始请求转发到真正的 Codex/Claude API 服务端点。
4.  **返回响应**: 将上游服务的响应直接返回给客户端。
5.  **处理异常**: 如果某个 Key 返回错误（如 `429 Too Many Requests`），`gatekeeper-rs` 会将其标记为临时不可用，并自动使用下一个 Key 重试请求。
