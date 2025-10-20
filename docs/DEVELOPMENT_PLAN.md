# 开发计划与里程碑（Development Plan & Milestones）

本文档将整体规划 gatekeeper-rs 的开发阶段、优先级与交付物，便于团队并行推进。

## 阶段划分

- M0 项目初始化（本分支）
  - 目标：建立基本骨架、文档与质量基线。
  - 交付物：
    - Rust crate 基础结构（cargo fmt/clippy 通过）
    - 架构文档与任务清单
    - 标准 .gitignore

- M1 MVP：基础代理能力
  - 目标：支持单 Provider 的非流式请求代理，轮询多 Key 与基础熔断。
  - 交付物：
    - Cloudflare Worker 路由与请求转发
    - Key 管理（内存 Key 池）
    - 轮询 LB + 简单熔断（基于 429/5xx）
    - 基础日志与错误处理

- M2 稳定性与可观测性
  - 目标：更健壮的重试/超时、结构化日志、健康检查。
  - 交付物：
    - 重试策略（指数退避 + 抖动）与超时
    - /healthz、/readyz 探针
    - 结构化日志（请求 ID、重试计数、耗时）

- M3 Streaming 支持与多 Provider
  - 目标：端到端 SSE 转发，抽象 Provider 层，支持第二个上游。
  - 交付物：
    - SSE 流式转发与中断恢复策略
    - Provider trait 与 OpenAI/Anthropic 适配器

- M4 配置与动态管理
  - 目标：Secrets/Env 配置完善，KV/DO 动态配置（可选）。
  - 交付物：
    - 统一配置结构与加载
    - 管理 API：查看 Key 状态/策略（需鉴权）
    - （可选）从 KV 拉取动态配置

- M5 速率与并发控制
  - 目标：全局/Key 级并发限制与限流形成闭环。
  - 交付物：
    - 并发计数器与最少并发策略
    - 简易令牌桶或队列

- M6 监控与告警
  - 目标：可视化与告警闭环。
  - 交付物：
    - Analytics Engine 指标
    - 告警阈值定义（错误率、延迟、可用度）

## 验收标准（每阶段通用）
- 所有变更通过 cargo fmt、cargo clippy -- -D warnings、测试。
- 文档更新：README 链接与 docs/* 相关章节。
- 部署说明更新：wrangler.toml 示例与 secrets 清单。

## 风险与缓解
- 上游 API 变更：通过 Provider 抽象隔离差异，版本化配置。
- Cloudflare 限制：评估 KV/DO 成本与配额，必要时降级为内存策略。
- 流式转发复杂度：先 MVP 后增强，限定重试场景与协议边界。
