# Phase 0A Codex app-server Handshake Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local Rust diagnostic CLI that starts the installed Codex app-server over stdio, completes the required `initialize`/`initialized` handshake, prints negotiated server information, and shuts the child down cleanly.

**Architecture:** Create a minimal Cargo workspace containing one retained diagnostic tool under `tools/codex-spike`. Keep the JSONL handshake generic over async readers/writers so it is deterministically tested with an in-memory fake server before touching the real Codex process. Do not add Relay, Connector, SQLite, Web, Thread, Turn, or Approval behavior in this plan.

**Tech Stack:** Rust 1.96, Tokio, Serde/serde_json, Clap, thiserror, tracing/tracing-subscriber, Codex app-server 0.144.1 stdio protocol.

---

## Scope and evidence

This is the first independently testable slice of Phase 0. Phase 0B adds Thread, Turn, streaming events, approvals, interrupt, and a local REPL after this handshake is proven.

Protocol evidence:

- Installed CLI: `codex-cli 0.144.1` at `/opt/homebrew/bin/codex`.
- `../codex/codex-rs/app-server/README.md:20`: stdio is newline-delimited JSON.
- `../codex/codex-rs/app-server/README.md:83`: one `initialize` request followed by one `initialized` notification is mandatory.
- Installed schema generated with `codex app-server generate-ts`: `InitializeParams`, `InitializeResponse`, and `ClientNotification` were checked against the installed binary.

## File map

- `Cargo.toml`: Workspace membership, shared metadata, dependencies, and lints.
- `rust-toolchain.toml`: Reproducible Rust toolchain.
- `.gitignore`: Cargo build output.
- `tools/codex-spike/Cargo.toml`: Diagnostic package.
- `tools/codex-spike/src/lib.rs`: Typed generic handshake.
- `tools/codex-spike/src/main.rs`: Real child-process lifecycle.
- `docs/development/codex-spike.md`: Operator runbook.

### Task 1: Bootstrap the Rust workspace

**Files:**
- Create: `.gitignore`
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `tools/codex-spike/Cargo.toml`
- Create: `tools/codex-spike/src/lib.rs`

- [ ] **Step 1: Create manifests and a failing identity test**

Create `.gitignore`:

```gitignore
/target/
```

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.96.0"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

Create root `Cargo.toml`:

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

Create `tools/codex-spike/Cargo.toml`:

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

Create `tools/codex-spike/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }
}
```

- [ ] **Step 2: Verify the intended failure**

Run:

```bash
cargo test -p bridgehub-codex-spike client_identity_is_stable
```

Expected: compilation fails with `cannot find function client_name in module super`.

- [ ] **Step 3: Add the smallest implementation**

Insert before the test module:

```rust
pub const fn client_name() -> &'static str {
    "bridgehub"
}
```

- [ ] **Step 4: Run the workspace gate**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands succeed; one test passes.

- [ ] **Step 5: Commit**

```bash
git add .gitignore Cargo.toml Cargo.lock rust-toolchain.toml tools/codex-spike
git commit -m "build: initialize Rust workspace"
```

### Task 2: Implement the typed JSONL handshake

**Files:**
- Modify: `tools/codex-spike/src/lib.rs`

- [ ] **Step 1: Add a failing in-memory handshake test**

Add these imports and the test inside `tests`. It intentionally references `InitializeResponse` and `handshake` before implementation:

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

- [ ] **Step 2: Verify the test fails**

```bash
cargo test -p bridgehub-codex-spike handshake_sends_initialize_then_initialized
```

Expected: compilation fails because `InitializeResponse` and `handshake` are undefined.

- [ ] **Step 3: Implement the complete handshake**

Replace all non-test content before `#[cfg(test)]` in `tools/codex-spike/src/lib.rs` with:

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

Keep `client_identity_is_stable` and `handshake_sends_initialize_then_initialized` unchanged in the test module.

- [ ] **Step 4: Run the gate**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands succeed; two tests pass.

- [ ] **Step 5: Commit**

```bash
git add tools/codex-spike/src/lib.rs
git commit -m "feat: implement Codex initialize handshake"
```

### Task 3: Spawn and stop the real Codex process

**Files:**
- Modify: `tools/codex-spike/src/lib.rs`
- Create: `tools/codex-spike/src/main.rs`

- [ ] **Step 1: Add the failing process-command test**

Append inside the `tests` module:

```rust
#[test]
fn app_server_arguments_use_the_supported_stdio_transport() {
    assert_eq!(
        super::app_server_args(),
        ["app-server", "--listen", "stdio://"]
    );
}
```

- [ ] **Step 2: Verify the test fails**

```bash
cargo test -p bridgehub-codex-spike app_server_arguments_use_the_supported_stdio_transport
```

Expected: compilation fails because `app_server_args` is undefined.

- [ ] **Step 3: Restore the minimal command contract**

Insert after `client_name`:

```rust
pub const fn app_server_args() -> [&'static str; 3] {
    ["app-server", "--listen", "stdio://"]
}
```

- [ ] **Step 4: Implement the real binary**

Create `tools/codex-spike/src/main.rs`:

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

- [ ] **Step 5: Run static checks**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p bridgehub-codex-spike -- --help
```

Expected: all commands succeed; three tests pass; help shows `--codex-bin <CODEX_BIN>`.

- [ ] **Step 6: Commit**

```bash
git add tools/codex-spike/src/lib.rs tools/codex-spike/src/main.rs
git commit -m "feat: spawn Codex app-server process"
```

### Task 4: Prove the live handshake and document it

**Files:**
- Create: `docs/development/codex-spike.md`
- Modify: `PROJECT_PROGRESS.md`
- Modify: `PROJECT_TODO.md`

- [ ] **Step 1: Run the installed Codex binary**

```bash
before=$(pgrep -f "codex app-server.*stdio" | sort || true)
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
after=$(pgrep -f "codex app-server.*stdio" | sort || true)
test "$after" = "$before"
```

Expected output contains:

```text
handshake=ok
user_agent=...
codex_home=/...
platform_family=unix
platform_os=macos
```

Expected: the five output fields are present and the before/after process sets are identical, proving this run left no child.

- [ ] **Step 2: Create the runbook**

Create `docs/development/codex-spike.md`:

````markdown
# Codex app-server Spike

This diagnostic proves that BridgeHub can start the installed Codex app-server, complete its required stdio handshake, and shut it down without an orphan process.

## Prerequisites

- Rust toolchain from `rust-toolchain.toml`.
- An installed and authenticated Codex CLI.
- A Codex app-server version whose generated schema matches the implementation.

## Run

```bash
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
```

Success prints `handshake=ok`, the Codex user agent/home, and platform information, then returns to the shell.

## Diagnose

- `No such file or directory`: pass the correct path with `--codex-bin`.
- `Not initialized`: compare initialize JSON with `codex app-server generate-ts`.
- JSON decode failure: regenerate the schema from the installed binary and compare `InitializeResponse`.
- Timeout on exit: confirm stdin is dropped and the child is killed after the two-second grace period.

Set `RUST_LOG=codex_app_server=debug` only for local diagnosis. Never forward this debug stream to public Relay logs.

## Next slice

Phase 0B adds Thread start/resume, Turn start, streaming items, approvals, interrupt, and a local REPL. This tool remains the smallest dependency-health check.
````

- [ ] **Step 3: Update project memory**

In `PROJECT_PROGRESS.md`:

- Change `当前阶段` from `计划` to `开发`.
- Add under `已完成`: `Phase 0A：Rust CLI 已完成真实 Codex app-server initialize/initialized handshake，并验证无遗留子进程。`
- Replace `最近一次进展` with the live command and successful unit/live proof dated 2026-07-18.

In `PROJECT_TODO.md`:

- Remove the Phase 0A execution and in-progress entries.
- Make `编写 Phase 0B Thread/Turn/Approval 实施计划` the first `下一步` item.

- [ ] **Step 4: Run the final gate**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Expected: all commands succeed with three tests passing and no whitespace errors.

- [ ] **Step 5: Commit**

```bash
git add docs/development/codex-spike.md PROJECT_PROGRESS.md PROJECT_TODO.md
git commit -m "docs: record Codex handshake proof"
```

## Completion criteria

- In-memory fake server proves exact `initialize` then `initialized` ordering.
- EOF, wrong response ID, server error, invalid JSON, and missing result have typed errors.
- The installed Codex app-server returns a typed `InitializeResponse`.
- The CLI exits within the grace period and leaves no child process.
- No Relay, Connector, Web, SQLite, Thread, Turn, or Approval code is introduced.
- Project memory points to Phase 0B.
