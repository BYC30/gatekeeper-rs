# gatekeeper-rs 架构设计

目标：在 Cloudflare Workers 上实现一个高性能、可扩展、可观测、具备多 Key 负载均衡和容错能力的 AI API 代理。

## 核心设计原则
- 简单优先：保持核心路径精简、可维护。
- 可观测性：关键路径埋点与日志默认开启，可定位问题。
- 无状态边缘：在边缘节点以无状态为主；需要共享状态时优先 KV/Durable Objects。
- 快速失败与重试：对可恢复错误（429/5xx/网络抖动）快速切换 Key 并重试。
- 供应商无关：通过 Provider 抽象屏蔽上游差异（OpenAI/Anthropic 等）。

## 组件与模块划分

1. HTTP 层（Cloudflare Worker + 路由）
   - 负责接收请求、校验合法性、解析请求体、统一响应格式。
   - 支持 Streaming（SSE）和非流式响应。

2. 认证与访问控制
   - 可选：对外暴露 Basic Auth/签名鉴权，或基于 IP/令牌白名单。

3. 配置管理（Config）
   - 从环境变量/Worker Secrets 读取：Provider、上游 Base URL、Key 列表、权重、路由策略等。
   - 后续扩展：从 KV/DO 拉取动态配置，支持热更新。

4. Key 池与健康状态（KeyPool）
   - Key 元信息：provider、权重、失效标记、熔断窗口、上次错误时间、并发计数等。
   - 熔断策略：基于错误比例/连续错误计数，进入半开与恢复。

5. 负载均衡器（LoadBalancer）
   - 支持策略：RR（轮询）、WRR（加权轮询）、最少并发、随机。
   - 选择可用 Key（过滤熔断/过载 Key），返回一个候选。

6. 上游客户端（Upstream Client）
   - Provider 抽象：统一接口 send(request, key) -> response。
   - 处理上游差异：路径、头、流式/非流式、错误码映射。

7. 重试与故障切换（Retry/Failover）
   - 支持幂等请求的透明重试；对非幂等或流式请求按策略降级。
   - 指数退避 + 抖动；限次重试（例如 2-3 次）。

8. 可观测性（Observability）
   - 结构化日志：请求 ID、上游耗时、所用 Key、重试次数、错误原因。
   - 指标：请求量、成功率、P99、429/5xx 比例、Key 可用度。
   - Trace：可选集成 Workers Analytics Engine / 3rd 方。

9. 限流与并发控制
   - 全局/Key 级并发上限；排队或快速失败。
   - 简单令牌桶/计数器实现；后续可搭配 DO 做分布式协调。

10. 管理与运维
   - Admin API（受保护）：查看 Key 状态、切换策略、健康检查。
   - 健康探针：/healthz（存活）、/readyz（就绪）。

## 请求处理流程

1. 入口：Worker 捕获请求，根据路由匹配到 Provider 适配器。
2. 认证与配额校验（可选）。
3. 从 KeyPool 通过 LB 选择一个可用 Key。
4. 构造上游请求（复制 method/headers/body，注入 API Key/命名空间路径）。
5. 发起请求，捕获响应；
   - 流式：边读边转发；遇错按策略切换 Key 并尝试重放（仅幂等/可恢复场景）。
   - 非流式：失败则按策略重试并切换 Key。
6. 更新 Key 健康状态与指标。
7. 返回响应至客户端。

## 数据结构示意

- Config
  - providers: [ProviderConfig]
  - keys: [KeyConfig]
  - lb_policy: RR|WRR|LeastPending
  - retry: max_retries, base_delay_ms, jitter
  - concurrency: global_max, per_key_max

- Key
  - id, value, provider
  - weight, fail_count, open_until, in_flight
  - last_error, last_used_at

- Provider traits
  - trait Provider { fn send(&self, req, key) -> Result<Resp>; fn map_error(&self, resp) -> UpstreamError; }

## Cloudflare 特性利用
- Secrets：API Keys/内部令牌管理。
- KV：缓存 Key 健康状态或动态配置（可选）。
- Durable Objects：跨实例的协调（高级特性，后续迭代）。
- Analytics Engine：埋点与指标聚合。

## 错误分类与处理
- 速率限制（429）：标记 Key 暂不可用，短期熔断并切换。
- 上游错误（5xx/网络）：指数退避重试，逐步切换 Key。
- 客户端错误（4xx 非 429）：直接返回，不重试。
- 超时：可配置超时时间，视为可重试错误。

## 安全
- 日志脱敏：绝不记录完整 API Key/敏感参数。
- 访问控制：Admin API 需要 Basic Auth/Token。
- CORS：仅允许受信来源（或可配置）。

## 最小可行版本（MVP）能力
- 单 Provider（OpenAI/或 Anthropic）
- 轮询 LB + 简单熔断
- 非流式代理 + 基础日志
- Worker Secrets 配置

## 后续增强
- 多 Provider 支持与自动路由
- Streaming（SSE）端到端转发
- 指标与可视化（Analytics Engine）
- Key 动态配置（KV/DO）
- 更智能的 LB（最少并发/权重/健康评分）
