# Phase 0A：Codex app-server 握手实施计划

> **供执行本计划的 Agent 使用：** 必须使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 技能，逐项执行本计划。所有步骤使用复选框（`- [ ]`）跟踪进度。

**目标：** 构建一个本地 Rust 诊断 CLI。它通过 stdio 启动已安装的 Codex app-server，完成必需的 `initialize`/`initialized` 握手，打印协商得到的服务端信息，并干净地关闭子进程。

**架构：** 创建一个最小 Cargo Workspace，并在 `tools/codex-spike` 下保留一个诊断工具。JSONL 握手逻辑面向通用异步 Reader/Writer 编写，以便先通过内存中的伪服务端完成确定性测试，再连接真实 Codex 进程。本计划不加入 Relay、Connector、SQLite、Web、Thread、Turn 或 Approval 行为。

**技术栈：** Rust 1.96、Tokio、Serde/serde_json、Clap、thiserror、tracing/tracing-subscriber，以及 Codex app-server 0.144.1 stdio 协议。

---

## 范围与依据

这是 Phase 0 中第一个可以独立测试的工作切片。完成并证明该握手后，Phase 0B 再加入 Thread、Turn、流式事件、审批、Interrupt 和本地 REPL。

协议依据：

- 已安装的 CLI：`codex-cli 0.144.1`，路径为 `/opt/homebrew/bin/codex`。
- `../codex/codex-rs/app-server/README.md:20`：stdio 使用以换行符分隔的 JSON。
- `../codex/codex-rs/app-server/README.md:83`：必须先发送一次 `initialize` 请求，再发送一次 `initialized` 通知。
- 使用 `codex app-server generate-ts` 从已安装版本生成 Schema，并已对照当前二进制检查 `InitializeParams`、`InitializeResponse` 和 `ClientNotification`。

## 文件职责

- `Cargo.toml`：定义 Workspace 成员、共享元数据、依赖和 Lint 规则。
- `rust-toolchain.toml`：固定可复现的 Rust 工具链。
- `.gitignore`：忽略 Cargo 构建产物。
- `tools/codex-spike/Cargo.toml`：诊断工具的软件包清单。
- `tools/codex-spike/src/lib.rs`：带类型的通用握手逻辑。
- `tools/codex-spike/src/main.rs`：真实子进程的生命周期管理。
- `docs/development/codex-spike.md`：操作与排障手册。

### 任务 1：初始化 Rust Workspace

**文件：**
- 新建：`.gitignore`
- 新建：`Cargo.toml`
- 新建：`rust-toolchain.toml`
- 新建：`tools/codex-spike/Cargo.toml`
- 新建：`tools/codex-spike/src/lib.rs`

- [ ] **步骤 1：创建清单文件和一个必定失败的身份标识测试**

创建 `.gitignore`：

```gitignore
/target/
```

创建 `rust-toolchain.toml`：

```toml
[toolchain]
channel = "1.96.0"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

创建根目录的 `Cargo.toml`：

```toml
[workspace]
members = ["tools/codex-spike"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[workspace.dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["io-util", "macros", "process", "rt-multi-thread", "time"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
```

创建 `tools/codex-spike/Cargo.toml`：

```toml
[package]
name = "bridgehub-codex-spike"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
clap.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[lints]
workspace = true
```

创建 `tools/codex-spike/src/lib.rs`：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }
}
```

- [ ] **步骤 2：确认测试按预期失败**

运行：

```bash
cargo test -p bridgehub-codex-spike client_identity_is_stable
```

预期：编译失败，并出现 `cannot find function client_name in module super`。

- [ ] **步骤 3：加入最小实现**

在测试模块之前插入：

```rust
pub const fn client_name() -> &'static str {
    "bridgehub"
}
```

- [ ] **步骤 4：执行 Workspace 质量检查**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

预期：所有命令成功，并通过 1 个测试。

- [ ] **步骤 5：提交改动**

```bash
git add .gitignore Cargo.toml Cargo.lock rust-toolchain.toml tools/codex-spike
git commit -m "build: initialize Rust workspace"
```

### 任务 2：实现带类型的 JSONL 握手

**文件：**
- 修改：`tools/codex-spike/src/lib.rs`

- [ ] **步骤 1：加入一个必定失败的内存握手测试**

在 `tests` 模块中加入以下导入和测试。这里会故意在实现之前引用 `InitializeResponse` 和 `handshake`：

```rust
use std::error::Error;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::{InitializeResponse, handshake};

#[tokio::test]
async fn handshake_sends_initialize_then_initialized() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (client_io, server_io) = tokio::io::duplex(8 * 1024);
    let (client_reader, mut client_writer) = tokio::io::split(client_io);
    let (server_reader, mut server_writer) = tokio::io::split(server_io);

    let fake_server = tokio::spawn(async move {
        let mut reader = BufReader::new(server_reader);
        let mut initialize_line = String::new();
        reader.read_line(&mut initialize_line).await?;
        let initialize: Value = serde_json::from_str(&initialize_line)?;
        assert_eq!(initialize["method"], "initialize");
        assert_eq!(initialize["id"], 0);
        assert_eq!(initialize["params"]["clientInfo"]["name"], "bridgehub");
        assert_eq!(
            initialize["params"]["capabilities"]["experimentalApi"].as_bool(),
            Some(false)
        );

        let response = json!({
            "id": 0,
            "result": {
                "userAgent": "codex_cli_rs/test",
                "codexHome": "/tmp/codex-home",
                "platformFamily": "unix",
                "platformOs": "macos"
            }
        });
        server_writer.write_all(response.to_string().as_bytes()).await?;
        server_writer.write_all(b"\n").await?;
        server_writer.flush().await?;

        let mut initialized_line = String::new();
        reader.read_line(&mut initialized_line).await?;
        let initialized: Value = serde_json::from_str(&initialized_line)?;
        assert_eq!(initialized, json!({ "method": "initialized" }));
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    let mut reader = BufReader::new(client_reader);
    let result = handshake(&mut reader, &mut client_writer).await?;
    assert_eq!(
        result,
        InitializeResponse {
            user_agent: "codex_cli_rs/test".to_owned(),
            codex_home: "/tmp/codex-home".into(),
            platform_family: "unix".to_owned(),
            platform_os: "macos".to_owned(),
        }
    );
    fake_server.await??;
    Ok(())
}
```

- [ ] **步骤 2：确认测试失败**

```bash
cargo test -p bridgehub-codex-spike handshake_sends_initialize_then_initialized
```

预期：由于尚未定义 `InitializeResponse` 和 `handshake`，编译失败。

- [ ] **步骤 3：实现完整握手流程**

将 `tools/codex-spike/src/lib.rs` 中 `#[cfg(test)]` 之前的所有非测试内容替换为：

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub const fn client_name() -> &'static str {
    "bridgehub"
}

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("app-server closed stdout before initialize completed")]
    UnexpectedEof,
    #[error("initialize response id was {actual}, expected 0")]
    UnexpectedResponseId { actual: u64 },
    #[error("initialize response did not contain result or error")]
    MissingResult,
    #[error("app-server rejected initialize with {code}: {message}")]
    Server { code: i64, message: String },
    #[error("stdio I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON from app-server: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub user_agent: String,
    pub codex_home: PathBuf,
    pub platform_family: String,
    pub platform_os: String,
}

#[derive(Debug, Serialize)]
struct Request<T> {
    method: &'static str,
    id: u64,
    params: T,
}

#[derive(Debug, Serialize)]
struct Notification {
    method: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    client_info: ClientInfo,
    capabilities: InitializeCapabilities,
}

#[derive(Debug, Serialize)]
struct ClientInfo {
    name: &'static str,
    title: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeCapabilities {
    experimental_api: bool,
    request_attestation: bool,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

pub async fn handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
) -> Result<InitializeResponse, HandshakeError>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let request = Request {
        method: "initialize",
        id: 0,
        params: InitializeParams {
            client_info: ClientInfo {
                name: client_name(),
                title: "BridgeHub",
                version: env!("CARGO_PKG_VERSION"),
            },
            capabilities: InitializeCapabilities {
                experimental_api: false,
                request_attestation: false,
            },
        },
    };
    write_json_line(writer, &request).await?;

    let mut line = String::new();
    if reader.read_line(&mut line).await? == 0 {
        return Err(HandshakeError::UnexpectedEof);
    }
    let envelope: ResponseEnvelope = serde_json::from_str(&line)?;
    if envelope.id != 0 {
        return Err(HandshakeError::UnexpectedResponseId {
            actual: envelope.id,
        });
    }
    if let Some(error) = envelope.error {
        return Err(HandshakeError::Server {
            code: error.code,
            message: error.message,
        });
    }
    let result = envelope.result.ok_or(HandshakeError::MissingResult)?;
    let response = serde_json::from_value(result)?;

    write_json_line(
        writer,
        &Notification {
            method: "initialized",
        },
    )
    .await?;
    Ok(response)
}

async fn write_json_line<W, T>(writer: &mut W, value: &T) -> Result<(), HandshakeError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut json = serde_json::to_vec(value)?;
    json.push(b'\n');
    writer.write_all(&json).await?;
    writer.flush().await?;
    Ok(())
}
```

保留测试模块中的 `client_identity_is_stable` 和 `handshake_sends_initialize_then_initialized`，不要修改。

- [ ] **步骤 4：执行质量检查**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

预期：所有命令成功，并通过 2 个测试。

- [ ] **步骤 5：提交改动**

```bash
git add tools/codex-spike/src/lib.rs
git commit -m "feat: implement Codex initialize handshake"
```

### 任务 3：启动并停止真实 Codex 进程

**文件：**
- 修改：`tools/codex-spike/src/lib.rs`
- 新建：`tools/codex-spike/src/main.rs`

- [ ] **步骤 1：加入一个必定失败的进程命令测试**

在 `tests` 模块中追加：

```rust
#[test]
fn app_server_arguments_use_the_supported_stdio_transport() {
    assert_eq!(
        super::app_server_args(),
        ["app-server", "--listen", "stdio://"]
    );
}
```

- [ ] **步骤 2：确认测试失败**

```bash
cargo test -p bridgehub-codex-spike app_server_arguments_use_the_supported_stdio_transport
```

预期：由于尚未定义 `app_server_args`，编译失败。

- [ ] **步骤 3：补齐最小命令约定**

在 `client_name` 之后插入：

```rust
pub const fn app_server_args() -> [&'static str; 3] {
    ["app-server", "--listen", "stdio://"]
}
```

- [ ] **步骤 4：实现真实可执行程序**

创建 `tools/codex-spike/src/main.rs`：

```rust
use std::{error::Error, io, path::PathBuf, process::Stdio, time::Duration};

use bridgehub_codex_spike::{app_server_args, handshake};
use clap::Parser;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(about = "Verify the Codex app-server stdio initialize handshake")]
struct Args {
    #[arg(long, default_value = "codex")]
    codex_bin: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    let args = Args::parse();
    let mut child = Command::new(&args.codex_bin)
        .args(app_server_args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdin was not piped"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdout was not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Codex child stderr was not piped"))?;

    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await? {
            debug!(target: "codex_app_server", message = %line);
        }
        Ok::<(), io::Error>(())
    });

    let mut reader = BufReader::new(stdout);
    let initialized = handshake(&mut reader, &mut stdin).await?;
    println!("handshake=ok");
    println!("user_agent={}", initialized.user_agent);
    println!("codex_home={}", initialized.codex_home.display());
    println!("platform_family={}", initialized.platform_family);
    println!("platform_os={}", initialized.platform_os);

    drop(stdin);
    let status = match timeout(Duration::from_secs(2), child.wait()).await {
        Ok(wait_result) => wait_result?,
        Err(_) => {
            child.kill().await?;
            child.wait().await?
        }
    };
    match stderr_task.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => debug!(%error, "failed while draining Codex stderr"),
        Err(error) => debug!(%error, "Codex stderr task failed"),
    }
    if !status.success() {
        return Err(format!("Codex app-server exited with {status}").into());
    }
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("bridgehub_codex_spike=info,codex_app_server=off"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
```

- [ ] **步骤 5：执行静态检查**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p bridgehub-codex-spike -- --help
```

预期：所有命令成功，通过 3 个测试，并且帮助信息中显示 `--codex-bin <CODEX_BIN>`。

- [ ] **步骤 6：提交改动**

```bash
git add tools/codex-spike/src/lib.rs tools/codex-spike/src/main.rs
git commit -m "feat: spawn Codex app-server process"
```

### 任务 4：验证真实握手并编写文档

**文件：**
- 新建：`docs/development/codex-spike.md`
- 修改：`PROJECT_PROGRESS.md`
- 修改：`PROJECT_TODO.md`

- [ ] **步骤 1：运行已经安装的 Codex 二进制文件**

```bash
before=$(pgrep -f "codex app-server.*stdio" | sort || true)
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
after=$(pgrep -f "codex app-server.*stdio" | sort || true)
test "$after" = "$before"
```

预期输出包含：

```text
handshake=ok
user_agent=...
codex_home=/...
platform_family=unix
platform_os=macos
```

预期：5 个输出字段全部存在，并且运行前后的进程集合完全一致，从而证明本次运行没有遗留子进程。

- [ ] **步骤 2：创建操作与排障手册**

创建 `docs/development/codex-spike.md`：

````markdown
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
````

- [ ] **步骤 3：更新项目记忆文档**

在 `PROJECT_PROGRESS.md` 中：

- 将 `当前阶段` 从 `计划` 改为 `开发`。
- 在 `已完成` 下加入：`Phase 0A：Rust CLI 已完成真实 Codex app-server initialize/initialized 握手，并验证无遗留子进程。`
- 将 `最近一次进展` 替换为 2026-07-18 执行的真实命令，以及单元测试和真实环境验证均成功的结果。

在 `PROJECT_TODO.md` 中：

- 删除 Phase 0A 的执行项和进行中事项。
- 将 `编写 Phase 0B Thread/Turn/Approval 实施计划` 设为 `下一步` 中的第一项。

- [ ] **步骤 4：执行最终质量检查**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

预期：所有命令成功，通过 3 个测试，并且不存在空白字符错误。

- [ ] **步骤 5：提交改动**

```bash
git add docs/development/codex-spike.md PROJECT_PROGRESS.md PROJECT_TODO.md
git commit -m "docs: record Codex handshake proof"
```

## 完成标准

- 内存伪服务端能够证明严格遵循先 `initialize`、后 `initialized` 的顺序。
- EOF、响应 ID 错误、服务端错误、无效 JSON 和缺少 Result 都有对应的类型化错误。
- 已安装的 Codex app-server 能够返回带类型的 `InitializeResponse`。
- CLI 在宽限期内退出，并且不遗留子进程。
- 不引入任何 Relay、Connector、Web、SQLite、Thread、Turn 或 Approval 代码。
- 项目记忆文档将下一阶段指向 Phase 0B。
