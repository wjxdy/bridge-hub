# BridgeHub

BridgeHub 是一个个人 Agent 网关：用户通过公网 Web 界面访问本机已有的编码 Agent，由本机 Connector 调用 Codex 等 Agent Harness 完成实际工作。

BridgeHub 不重新实现模型调用、Agent Loop 或 Tool Calling。V1 复用 Codex app-server，重点实现公网访问、本机连接、Session 管理、事件流、审批、恢复和安全边界。

## 当前阶段

项目设计已经批准。Phase 0A 已完成 Codex app-server 真实握手和子进程回收验证，下一步进入 Phase 0B Thread/Turn/Approval 计划。

- [V1 系统设计](docs/superpowers/specs/2026-07-18-bridgehub-design.md)
- [Phase 0A：Codex app-server 握手实施计划](docs/superpowers/plans/2026-07-18-phase-0a-codex-handshake.md)
- [Codex app-server 协议验证工具](docs/development/codex-spike.md)
- [项目进度](PROJECT_PROGRESS.md)
- [项目待办](PROJECT_TODO.md)

## V1 目标

- 从公网网页选择已授权的本机 Workspace。
- 创建、查看并恢复 Codex Session。
- 支持多轮对话和流式输出。
- 展示 Shell 与文件修改事件。
- 支持批准、拒绝和停止 Turn。
- 在浏览器或 Relay 断线后恢复状态。
- Relay 不持久化聊天正文、命令输出或 diff。

## V1 非目标

- 重写 Codex、Claude Code 或 OpenClaw 的 Agent Loop。
- 录音、文件和图片上传。
- Claude Code、OpenClaw 等多后端支持。
- 多用户、RBAC、完整终端、文件管理器、Git 面板和原生移动端。
