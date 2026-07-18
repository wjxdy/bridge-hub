# Codex app-server 协议验证工具

这个诊断工具用于证明 BridgeHub 能够启动已经安装的 Codex app-server，完成必需的 stdio 握手，并在退出时不留下孤儿进程。

## 前置条件

- 使用 `rust-toolchain.toml` 中指定的 Rust 工具链。
- 本机已经安装 Codex CLI，并已完成登录认证。
- Codex app-server 生成的 Schema 必须与当前实现相匹配。

## 运行

```bash
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
```

成功时会打印 `handshake=ok`、Codex User Agent、Codex Home 和平台信息，随后返回 Shell。

## 排查问题

- 出现 `No such file or directory`：通过 `--codex-bin` 传入正确路径。
- 出现 `Not initialized`：使用 `codex app-server generate-ts` 生成的结果对照 Initialize JSON。
- JSON 解码失败：从已经安装的二进制文件重新生成 Schema，并对照 `InitializeResponse`。
- 退出超时：确认程序已经关闭 stdin，并在 2 秒宽限期结束后终止子进程。

仅在本地排查问题时设置 `RUST_LOG=codex_app_server=debug`。绝不能把该调试输出转发到公网 Relay 日志。

## 下一阶段

Phase 0B 将加入 Thread 的创建与恢复、Turn 启动、流式 Item、审批、Interrupt 和本地 REPL。该工具会继续作为最小依赖健康检查工具保留。
