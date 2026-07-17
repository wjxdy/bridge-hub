# 项目进度

## 基本信息

- 项目名称：BridgeHub
- 当前阶段：设计
- 最后更新：2026-07-18

## 当前状态

- V1 产品范围、系统架构、通信协议、状态归属、安全边界、工程结构和验证策略已经完成讨论并获得确认。
- 独立 Git 仓库已初始化，完整设计已写入 `docs/superpowers/specs/2026-07-18-bridgehub-design.md`。
- 尚未开始业务代码实现。

## 已完成

- 调研 OpenClaw、ZeroClaw、Codex app-server、Claude Code 可编程接口和现有远程 Agent 项目。
- 确认 V1 采用 Web App → Public Relay → Local Connector → Codex app-server 架构。
- 确认 Relay 可信但不持久化聊天正文，不要求 V1 端到端加密。
- 确认 V1 只支持 Codex、文本输入和单用户部署。
- 确认 Rust 核心、TypeScript Web、Tokio Actor、SQLite 和 JSONL/JSON-RPC 技术路线。

## 关键决策

- 2026-07-18：项目定名为 BridgeHub，本地仓库目录为 `/Users/xulei/.dev/bridgehub`。
- 2026-07-18：复用 Codex app-server，不解析交互式终端输出，不自行实现 Agent Loop。
- 2026-07-18：本机 Connector 只建立出站 WSS，本机不开放公网入站端口。
- 2026-07-18：Relay 只保存设备和 Session 路由元数据；Codex 是会话内容的唯一持久化真相。
- 2026-07-18：公开协议通过 `AgentBackend` 与 Codex 私有协议隔离，为后续 Claude Code/OpenClaw 后端保留扩展点。
- 2026-07-18：V1 使用 Workspace Allowlist、Human/Device 分离凭证、Codex Sandbox 和显式审批。

## 技术 / 结构备注

- Rust Workspace 计划包含 protocol、agent-core、codex-backend、connector 和 relay。
- Web 计划使用 React、TypeScript 和 Vite。
- 运行状态使用有界 Tokio Channel 与 Actor 所有权建模，避免关键状态散落在全局锁中。
- Relay 与 Connector 使用独立 SQLite；不复制 Codex transcript。

## 最近一次进展

- 2026-07-18：初始化项目仓库，创建 README、项目记忆与完整 V1 设计文档。
