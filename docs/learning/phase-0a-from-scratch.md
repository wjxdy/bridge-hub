# BridgeHub Phase 0A：从零手敲 Codex 握手程序

> 这是你应该照着操作的正式教程。`docs/superpowers/specs` 是设计说明，`docs/superpowers/plans` 是 Codex 的内部执行计划，都不是手敲教程。

## 使用规则

1. 所有命令都由你在终端亲自输入。
2. 所有项目代码都由你在 Neovim 亲自输入，不要复制粘贴。
3. 一次只做一个带编号的步骤。
4. 命令结果与“预期结果”不一致时立即停止，把完整命令和完整输出发给我。
5. 标记为“预期失败”的测试必须先看到失败，才能继续输入实现。

## 你最终会实现什么

最终运行：

```bash
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
```

你会看到类似：

```text
handshake=ok
user_agent=...
codex_home=/...
platform_family=unix
platform_os=macos
```

这里的 `...` 是你电脑上的真实值，不要求逐字相同。

当前只实现：

```text
Rust 启动 Codex
→ Rust 发送 initialize
→ Codex 返回初始化结果
→ Rust 发送 initialized
→ Rust 打印结果
→ Rust 安全关闭 Codex
```

当前不实现聊天、Thread、Turn、Tool Calling、Session、Relay、Connector 或 Web。

---

# 第 0 章：先认识两个程序

硬盘上的 `/opt/homebrew/bin/codex` 是一个可执行文件。执行它以后，操作系统会创建一个正在运行的 Codex 进程。

我们要写的 Rust CLI 也是一个程序。运行时存在两个独立进程：

```text
bridgehub-codex-spike       ← Rust 父进程
└── codex app-server        ← Codex 子进程
```

这和 Web 服务启动 `ffmpeg` 处理视频相似：Web 服务没有实现视频编码，只是启动并控制 ffmpeg。BridgeHub 没有实现 Agent Loop，只是启动并控制 Codex。

- [ ] 我知道 Rust 和 Codex 是两个独立进程。
- [ ] 我知道 Codex 是子进程，不是 Rust 子线程。

---

# 第 1 章：确认目录和 Git

## 1.1 进入项目目录

在终端亲自输入：

```bash
# cd 是 change directory，表示切换目录。
cd /Users/xulei/.dev/bridge-hub

# pwd 是 print working directory，打印当前目录。
pwd

# ls 列出文件；-l 显示详细信息；-a 显示隐藏文件。
ls -la
```

`pwd` 必须输出：

```text
/Users/xulei/.dev/bridge-hub
```

你已经执行过 `git init`，不要重复初始化。检查当前分支：

```bash
# --show-current 只打印当前分支名。
git branch --show-current
```

预期：

```text
main
```

## 1.2 `.gitignore` 的位置

```text
相对路径：.gitignore
绝对路径：/Users/xulei/.dev/bridge-hub/.gitignore
所在位置：项目根目录
```

打开：

```bash
nvim .gitignore
```

完整内容：

```gitignore
# Cargo 把编译产物放在项目根目录的 target 中。
# 编译产物可以重新生成，所以不提交到 Git。
/target/
```

保存退出：

```vim
:wq
```

---

# 第 2 章：建立 Cargo Workspace

完成本章后的目录结构：

```text
/Users/xulei/.dev/bridge-hub/
├── .gitignore
├── Cargo.toml
├── rust-toolchain.toml
└── tools/
    └── codex-spike/
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

## 2.1 修正工具链文件名

你现在创建的是 `toolchain.toml`，正确名称必须是 `rust-toolchain.toml`。

在终端亲自输入：

```bash
# mv 可以移动文件，也可以给文件改名。
mv toolchain.toml rust-toolchain.toml
```

正确位置：

```text
相对路径：rust-toolchain.toml
绝对路径：/Users/xulei/.dev/bridge-hub/rust-toolchain.toml
所在位置：项目根目录
```

打开：

```bash
nvim rust-toolchain.toml
```

在 Neovim 中输入 `ggdG` 清空旧内容，然后按 `i` 进入插入模式，亲手输入：

```toml
# [toolchain] 表示下面是 Rust 工具链配置。
[toolchain]

# 固定使用 Rust 1.96.0。
channel = "1.96.0"

# 安装 Clippy 静态检查器和 rustfmt 格式化工具。
components = ["clippy", "rustfmt"]

# 只安装这个项目需要的最小工具链组件。
profile = "minimal"
```

注意是 `rustfmt`，不是 `rustfmtk`。

保存退出：

```vim
:wq
```

## 2.2 创建目录

在项目根目录执行：

```bash
# mkdir 创建目录。
# -p 表示父目录不存在时一起创建；已经存在时不报错。
mkdir -p tools/codex-spike/src
```

## 2.3 根 `Cargo.toml`

```text
相对路径：Cargo.toml
绝对路径：/Users/xulei/.dev/bridge-hub/Cargo.toml
所在位置：项目根目录
```

打开现有文件：

```bash
nvim Cargo.toml
```

输入 `ggdG` 清空当前不完整内容，然后亲手输入完整内容：

```toml
# [workspace] 表示这个 Cargo.toml 管理整个 Workspace。
[workspace]

# members 列出属于当前 Workspace 的 Package 路径。
# 必须是 tools，不是 tool。
members = ["tools/codex-spike"]

# resolver = "3" 使用适配 Rust 2024 Edition 的依赖解析规则。
resolver = "3"

# [workspace.package] 定义所有成员可以继承的软件包元数据。
[workspace.package]

# 当前项目版本。
version = "0.1.0"

# 使用 Rust 2024 Edition 的语法和规则。
edition = "2024"

# 声明项目支持的最低 Rust 版本。
rust-version = "1.96"

# [workspace.dependencies] 集中声明所有成员可以继承的依赖。
[workspace.dependencies]

# Clap 负责解析命令行参数；derive 允许通过宏生成解析代码。
clap = { version = "4", features = ["derive"] }

# Serde 负责类型与数据之间的转换；derive 生成转换实现。
serde = { version = "1", features = ["derive"] }

# serde_json 负责 JSON 编码和解码。
serde_json = "1"

# thiserror 帮助我们定义带类型的错误。
thiserror = "2"

# Tokio 是异步运行时。
# io-util 提供异步读写扩展。
# macros 提供 #[tokio::main] 和 #[tokio::test]。
# process 提供异步子进程管理。
# rt-multi-thread 提供多线程运行时。
# time 提供超时功能。
tokio = { version = "1", features = ["io-util", "macros", "process", "rt-multi-thread", "time"] }

# tracing 提供结构化日志 API。
tracing = "0.1"

# tracing-subscriber 负责收集和输出日志。
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# [workspace.lints.rust] 定义 Rust 编译器级别的统一检查规则。
[workspace.lints.rust]

# 禁止项目使用 unsafe 代码。
unsafe_code = "forbid"

# [workspace.lints.clippy] 定义统一的 Clippy 规则。
[workspace.lints.clippy]

# 禁止使用可能直接导致程序崩溃的 unwrap()。
unwrap_used = "deny"

# 禁止使用可能直接导致程序崩溃的 expect()。
expect_used = "deny"
```

保存退出：

```vim
:wq
```

## 2.4 Crate `Cargo.toml`

```text
相对路径：tools/codex-spike/Cargo.toml
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/Cargo.toml
所在位置：tools/codex-spike 目录
```

打开：

```bash
nvim tools/codex-spike/Cargo.toml
```

亲手输入：

```toml
# [package] 开始定义这个 Package。
[package]

# Cargo Package 名称允许使用连字符。
name = "bridgehub-codex-spike"

# 从根 Workspace 继承版本。
version.workspace = true

# 从根 Workspace 继承 Rust Edition。
edition.workspace = true

# 从根 Workspace 继承最低 Rust 版本。
rust-version.workspace = true

# [dependencies] 开始声明这个 Package 使用的依赖。
[dependencies]

# 每个 .workspace = true 都表示继承根 Workspace 中的同名依赖。
clap.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

# [lints] 配置这个 Package 的检查规则。
[lints]

# 继承根 Workspace 的 Rust 和 Clippy Lint。
workspace = true
```

保存退出：

```vim
:wq
```

Package 名叫 `bridgehub-codex-spike`，但 Rust 代码导入时连字符会变成下划线：

```rust
use bridgehub_codex_spike::handshake;
```

- [ ] `rust-toolchain.toml` 位于项目根目录。
- [ ] 根 `Cargo.toml` 包含 `[workspace]`。
- [ ] 成员路径是 `tools/codex-spike`。
- [ ] Crate `Cargo.toml` 已创建。

---

# 第 3 章：第一个测试和第一个实现

## 3.1 先写测试

```text
相对路径：tools/codex-spike/src/lib.rs
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/src/lib.rs
所在位置：tools/codex-spike/src 目录
```

打开你当前不完整的文件：

```bash
nvim tools/codex-spike/src/lib.rs
```

输入 `ggdG` 清空内容，然后亲手输入：

```rust
// #[cfg(test)] 表示下面的模块只在运行测试时参与编译。
#[cfg(test)]
// mod tests 创建一个名为 tests 的模块。
// 模块必须有名字，所以不能只写 mod。
mod tests {
    // #[test] 把下面的函数注册成单元测试。
    #[test]
    // 函数名描述我们要保护的行为。
    fn client_identity_is_stable() {
        // super 指向 tests 的上一层模块。
        // 注意正确拼写是 super，不是 supper。
        // 这里故意调用还不存在的 client_name，所以第一次必须失败。
        assert_eq!(super::client_name(), "bridgehub");
    }
}
```

保存退出：

```vim
:wq
```

运行测试：

```bash
cargo test -p bridgehub-codex-spike client_identity_is_stable
```

正确的第一次结果是编译失败，关键错误类似：

```text
cannot find function `client_name` in module `super`
```

这个失败证明测试确实在检查尚未实现的能力。

## 3.2 紧接着写实现

重新打开：

```bash
nvim tools/codex-spike/src/lib.rs
```

把光标放到文件第一行，在测试模块前面亲手加入：

```rust
// pub 表示其他模块可以调用这个函数。
// const fn 表示它可以在允许的场景中参与编译期求值。
// &'static str 表示返回的字符串在整个程序运行期间都有效。
pub const fn client_name() -> &'static str {
    // Rust 会把最后一个没有分号的表达式作为函数返回值。
    "bridgehub"
}

```

此时完整 `lib.rs` 必须是：

```rust
// pub 表示其他模块可以调用这个函数。
// const fn 表示它可以在允许的场景中参与编译期求值。
// &'static str 表示返回的字符串在整个程序运行期间都有效。
pub const fn client_name() -> &'static str {
    // 最后一个没有分号的表达式是函数返回值。
    "bridgehub"
}

// 下面的模块只在测试时编译。
#[cfg(test)]
// 创建 tests 模块。
mod tests {
    // 注册单元测试。
    #[test]
    // 测试客户端名称保持不变。
    fn client_identity_is_stable() {
        // 调用上一层的 client_name，并比较结果。
        assert_eq!(super::client_name(), "bridgehub");
    }
}
```

保存退出，然后运行：

```bash
cargo fmt --all
cargo test -p bridgehub-codex-spike client_identity_is_stable
```

预期：

```text
1 passed; 0 failed
```

现在测试后面已经有真实实现：`client_name()`。

- [ ] 我先看到了缺少 `client_name` 的预期失败。
- [ ] 我加入了 `client_name` 实现。
- [ ] 我重新运行后看到 1 个测试通过。

---

# 第 4 章：Rust 和 Codex 怎样交换消息

普通终端程序默认这样工作：

```text
键盘 → stdin → 程序 → stdout → 屏幕
                       stderr → 屏幕
```

Rust 启动 Codex 时，让操作系统把这些通道连接到 Rust：

```text
Rust 写入 → Codex stdin
Rust 读取 ← Codex stdout
Rust 读取 ← Codex stderr
```

管道只是本机字节通道，不理解 JSON。Rust 和 Codex 约定“一行 JSON 是一条消息”，这叫 JSONL。

```text
{"method":"initialize","id":0,"params":{...}}\n
{"id":0,"result":{...}}\n
{"method":"initialized"}\n
```

- `initialize` 是请求：有 `id`，需要响应。
- `InitializeResponse` 是响应：使用同一个 `id`。
- `initialized` 是通知：没有 `id`，不等待响应。
- `\n` 是换行符，告诉接收方一条消息已经结束。

---

# 第 5 章：实现完整握手

这一章先写会失败的握手测试，再输入完整实现。

## 5.1 先把 `lib.rs` 替换成测试阶段

打开：

```bash
nvim tools/codex-spike/src/lib.rs
```

保留已有 `client_name`，把测试模块替换为下面内容。这里暂时引用不存在的 `InitializeResponse` 和 `handshake`，所以必须失败。

```rust
// 返回固定客户端名称。
pub const fn client_name() -> &'static str {
    // 返回字符串字面量。
    "bridgehub"
}

// 只在测试时编译下面模块。
#[cfg(test)]
// 定义 tests 模块。
mod tests {
    // 导入标准错误 Trait，供测试返回不同错误。
    use std::error::Error;

    // Value 表示任意 JSON；json! 宏用于构造 JSON。
    use serde_json::{Value, json};
    // 导入异步逐行读取、异步写入和缓冲 Reader。
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // 从上一层导入尚未实现的响应类型和握手函数。
    use super::{InitializeResponse, handshake};

    // 注册同步测试。
    #[test]
    // 测试客户端名称。
    fn client_identity_is_stable() {
        // 预期名称为 bridgehub。
        assert_eq!(super::client_name(), "bridgehub");
    }

    // 注册由 Tokio 运行的异步测试。
    #[tokio::test]
    // async 表示函数内部可以使用 await。
    async fn handshake_sends_initialize_then_initialized(
    // 成功返回 ()；失败返回可跨线程传递的任意错误。
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // 在内存中建立容量为 8 KiB 的双向通道。
        let (client_io, server_io) = tokio::io::duplex(8 * 1024);
        // 把客户端一端拆成 Reader 和 Writer。
        let (client_reader, mut client_writer) = tokio::io::split(client_io);
        // 把伪服务端一端拆成 Reader 和 Writer。
        let (server_reader, mut server_writer) = tokio::io::split(server_io);

        // 并发启动伪 Codex 服务端异步任务。
        let fake_server = tokio::spawn(async move {
            // 用 BufReader 包装服务端 Reader，支持按行读取。
            let mut reader = BufReader::new(server_reader);
            // 创建字符串保存 initialize 消息。
            let mut initialize_line = String::new();
            // 等待客户端发送一行；? 把错误直接返回。
            reader.read_line(&mut initialize_line).await?;
            // 把字符串解析成通用 JSON Value。
            let initialize: Value = serde_json::from_str(&initialize_line)?;
            // 检查方法名。
            assert_eq!(initialize["method"], "initialize");
            // 检查请求 ID。
            assert_eq!(initialize["id"], 0);
            // 检查客户端名称。
            assert_eq!(initialize["params"]["clientInfo"]["name"], "bridgehub");
            // 检查实验 API 默认关闭。
            assert_eq!(
                initialize["params"]["capabilities"]["experimentalApi"].as_bool(),
                Some(false)
            );

            // 构造伪 Codex 的成功响应。
            let response = json!({
                "id": 0,
                "result": {
                    "userAgent": "codex_cli_rs/test",
                    "codexHome": "/tmp/codex-home",
                    "platformFamily": "unix",
                    "platformOs": "macos"
                }
            });
            // 把响应 JSON 转成字符串和字节，再写给客户端。
            server_writer.write_all(response.to_string().as_bytes()).await?;
            // 写入换行符，结束这条 JSONL 消息。
            server_writer.write_all(b"\n").await?;
            // 立即把缓冲内容送出。
            server_writer.flush().await?;

            // 创建字符串保存 initialized 通知。
            let mut initialized_line = String::new();
            // 等待客户端的下一行。
            reader.read_line(&mut initialized_line).await?;
            // 解析通知。
            let initialized: Value = serde_json::from_str(&initialized_line)?;
            // 检查通知内容准确。
            assert_eq!(initialized, json!({ "method": "initialized" }));
            // 显式说明任务成功时的返回类型。
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });

        // 包装客户端 Reader。
        let mut reader = BufReader::new(client_reader);
        // 调用尚未实现的握手函数。
        let result = handshake(&mut reader, &mut client_writer).await?;
        // 检查 JSON 已转换成准确的 Rust 类型。
        assert_eq!(
            result,
            InitializeResponse {
                user_agent: "codex_cli_rs/test".to_owned(),
                codex_home: "/tmp/codex-home".into(),
                platform_family: "unix".to_owned(),
                platform_os: "macos".to_owned(),
            }
        );
        // 第一个 ? 处理 Join 错误，第二个处理任务内部错误。
        fake_server.await??;
        // 整个测试成功。
        Ok(())
    }
}
```

保存后运行：

```bash
cargo test -p bridgehub-codex-spike handshake_sends_initialize_then_initialized
```

预期失败：找不到 `InitializeResponse` 和 `handshake`。

## 5.2 输入完整实现

现在重新打开 `lib.rs`，把 `#[cfg(test)]` 之前的内容整体替换成下面代码。不要删除或修改刚才的测试模块。

```rust
// PathBuf 保存由操作系统路径组成的数据。
use std::path::PathBuf;

// Deserialize 把数据转换为 Rust 类型；Serialize 做反向转换。
use serde::{Deserialize, Serialize};
// Value 可以暂存结构尚未确定的 JSON。
use serde_json::Value;
// Error 派生宏帮助生成标准错误实现。
use thiserror::Error;
// 导入异步 Reader、Writer 及其扩展方法。
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

// 返回 BridgeHub 的协议客户端名称。
pub const fn client_name() -> &'static str {
    // 返回固定字符串。
    "bridgehub"
}

// Debug 用于调试输出；Error 生成 std::error::Error 实现。
#[derive(Debug, Error)]
// pub 让 CLI 和测试可以看到这个错误类型。
pub enum HandshakeError {
    // error 属性定义这个变体的显示文本。
    #[error("app-server closed stdout before initialize completed")]
    // Codex 在响应前关闭 stdout。
    UnexpectedEof,
    // {actual} 会替换成变体字段值。
    #[error("initialize response id was {actual}, expected 0")]
    // Codex 返回了错误的请求 ID。
    UnexpectedResponseId { actual: u64 },
    // 响应没有成功结果也没有错误。
    #[error("initialize response did not contain result or error")]
    MissingResult,
    // Codex 主动返回 RPC 错误。
    #[error("app-server rejected initialize with {code}: {message}")]
    Server { code: i64, message: String },
    // transparent conversion：? 可以把 std::io::Error 转为这个变体。
    #[error("stdio I/O failed: {0}")]
    Io(#[from] std::io::Error),
    // ? 可以把 serde_json::Error 转为这个变体。
    #[error("invalid JSON from app-server: {0}")]
    Json(#[from] serde_json::Error),
}

// 生成调试、克隆、比较和反序列化能力。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
// 把 Rust snake_case 字段映射到 JSON camelCase 字段。
#[serde(rename_all = "camelCase")]
// Codex initialize 成功响应中的业务字段。
pub struct InitializeResponse {
    pub user_agent: String,
    pub codex_home: PathBuf,
    pub platform_family: String,
    pub platform_os: String,
}

// T 是 params 的具体类型。
#[derive(Debug, Serialize)]
struct Request<T> {
    method: &'static str,
    id: u64,
    params: T,
}

// 通知没有请求 ID，也不等待响应。
#[derive(Debug, Serialize)]
struct Notification {
    method: &'static str,
}

// initialize 请求参数。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    client_info: ClientInfo,
    capabilities: InitializeCapabilities,
}

// 告诉 Codex 是谁连接它。
#[derive(Debug, Serialize)]
struct ClientInfo {
    name: &'static str,
    title: &'static str,
    version: &'static str,
}

// 声明客户端能力。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeCapabilities {
    experimental_api: bool,
    request_attestation: bool,
}

// 包住 JSON-RPC 风格响应。
#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    id: u64,
    // 字段缺失时使用 Option 的默认值 None。
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

// Codex 返回的 RPC 错误字段。
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

// R 和 W 让同一个握手函数既能连接内存测试，也能连接真实进程。
pub async fn handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
) -> Result<InitializeResponse, HandshakeError>
where
    // Reader 必须支持异步缓冲读取并可安全移动。
    R: AsyncBufRead + Unpin,
    // Writer 必须支持异步写入并可安全移动。
    W: AsyncWrite + Unpin,
{
    // 构造 initialize 请求。
    let request = Request {
        method: "initialize",
        id: 0,
        params: InitializeParams {
            client_info: ClientInfo {
                name: client_name(),
                title: "BridgeHub",
                // env! 在编译时读取当前 Package 版本。
                version: env!("CARGO_PKG_VERSION"),
            },
            capabilities: InitializeCapabilities {
                experimental_api: false,
                request_attestation: false,
            },
        },
    };
    // 序列化、写入换行并 flush。
    write_json_line(writer, &request).await?;

    // 准备保存 Codex 的一行响应。
    let mut line = String::new();
    // read_line 返回 0 表示到达 EOF。
    if reader.read_line(&mut line).await? == 0 {
        return Err(HandshakeError::UnexpectedEof);
    }
    // 解析响应外层结构。
    let envelope: ResponseEnvelope = serde_json::from_str(&line)?;
    // 响应必须对应请求 ID 0。
    if envelope.id != 0 {
        return Err(HandshakeError::UnexpectedResponseId {
            actual: envelope.id,
        });
    }
    // 如果 Codex 返回 error，转换成我们的错误。
    if let Some(error) = envelope.error {
        return Err(HandshakeError::Server {
            code: error.code,
            message: error.message,
        });
    }
    // 成功响应必须有 result。
    let result = envelope.result.ok_or(HandshakeError::MissingResult)?;
    // 把通用 JSON Value 转成 InitializeResponse。
    let response = serde_json::from_value(result)?;

    // 按协议发送 initialized 通知。
    write_json_line(
        writer,
        &Notification {
            method: "initialized",
        },
    )
    .await?;
    // 返回类型化初始化结果。
    Ok(response)
}

// 把任意可序列化值写成一条 JSONL 消息。
async fn write_json_line<W, T>(writer: &mut W, value: &T) -> Result<(), HandshakeError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    // 序列化为 UTF-8 JSON 字节。
    let mut json = serde_json::to_vec(value)?;
    // 添加 JSONL 消息结束换行符。
    json.push(b'\n');
    // 写入全部字节。
    writer.write_all(&json).await?;
    // 把缓冲内容立即送给接收方。
    writer.flush().await?;
    // 写入成功。
    Ok(())
}
```

注意：文件最上面的 `use` 必须位于 `client_name` 之前；不要保留旧的第二份 `client_name`。

保存后运行：

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

预期：2 个测试通过。

- [ ] 我先看到了缺少握手类型和函数的预期失败。
- [ ] 我输入了完整握手实现。
- [ ] 我看到 2 个测试通过。

---

# 第 6 章：启动真实 Codex 子进程

## 6.1 先测试启动参数

在 `lib.rs` 的 `tests` 模块中，放在异步握手测试前面，加入：

```rust
// 注册同步测试。
#[test]
// 名称说明测试 stdio 运输参数。
fn app_server_arguments_use_the_supported_stdio_transport() {
    // app_server_args 尚未实现，所以第一次运行必须失败。
    assert_eq!(
        super::app_server_args(),
        ["app-server", "--listen", "stdio://"]
    );
}
```

运行：

```bash
cargo test -p bridgehub-codex-spike app_server_arguments_use_the_supported_stdio_transport
```

预期失败：找不到 `app_server_args`。

然后在 `lib.rs` 的 `client_name` 后加入：

```rust
// 返回启动 Codex app-server 所需的三个固定参数。
pub const fn app_server_args() -> [&'static str; 3] {
    // stdio:// 表示通过标准输入输出通信，不监听网络端口。
    ["app-server", "--listen", "stdio://"]
}
```

再次运行相同测试，预期通过。

## 6.2 创建 `main.rs`

```text
相对路径：tools/codex-spike/src/main.rs
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/src/main.rs
所在位置：tools/codex-spike/src，与 lib.rs 同级
```

打开：

```bash
nvim tools/codex-spike/src/main.rs
```

亲手输入完整代码：

```rust
// 从标准库导入错误、I/O、路径、进程标准流和时间长度类型。
use std::{error::Error, io, path::PathBuf, process::Stdio, time::Duration};

// 从我们自己的 Library 导入启动参数和握手函数。
use bridgehub_codex_spike::{app_server_args, handshake};
// 导入 Clap 的命令行解析 Trait。
use clap::Parser;
// 从 Tokio 导入异步逐行读取、缓冲 Reader、子进程命令和超时。
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};
// 导入 debug 日志宏。
use tracing::debug;
// 导入通过环境变量配置日志级别的 Filter。
use tracing_subscriber::EnvFilter;

// Debug 便于调试；Parser 让 Clap 为结构体生成参数解析代码。
#[derive(Debug, Parser)]
// 设置 --help 中的程序说明。
#[command(about = "Verify the Codex app-server stdio initialize handshake")]
// Args 保存命令行参数。
struct Args {
    // 定义 --codex-bin；不提供时默认执行 PATH 中的 codex。
    #[arg(long, default_value = "codex")]
    // PathBuf 保存 Codex 可执行文件路径。
    codex_bin: PathBuf,
}

// Tokio 创建异步运行时，并在其中执行 main。
#[tokio::main]
// 成功返回 ()；失败返回任意标准错误。
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志输出。
    init_tracing();
    // 从当前进程的命令行解析参数。
    let args = Args::parse();
    // 准备启动用户指定的 Codex 可执行文件。
    let mut child = Command::new(&args.codex_bin)
        // 添加 app-server --listen stdio:// 参数。
        .args(app_server_args())
        // 把 Codex stdin 连接成 Rust 可写管道。
        .stdin(Stdio::piped())
        // 把 Codex stdout 连接成 Rust 可读管道。
        .stdout(Stdio::piped())
        // 把 Codex stderr 连接成 Rust 可读管道。
        .stderr(Stdio::piped())
        // Rust 意外丢弃 Child 时，请求终止 Codex。
        .kill_on_drop(true)
        // 真正创建 Codex 操作系统子进程；失败时 ? 返回错误。
        .spawn()?;

    // Child 中取出 stdin 所有权，Rust 用它向 Codex 写消息。
    let mut stdin = child
        .stdin
        .take()
        // 理论上已经 piped；若不存在则构造明确 I/O 错误。
        .ok_or_else(|| io::Error::other("Codex child stdin was not piped"))?;
    // 取出 stdout 所有权，Rust 用它读取协议消息。
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdout was not piped"))?;
    // 取出 stderr 所有权，Rust 用它排空日志。
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Codex child stderr was not piped"))?;

    // 创建 Tokio 异步任务持续读取 stderr；它不是 Codex 子线程。
    let stderr_task = tokio::spawn(async move {
        // 把 stderr 包装成可以逐行读取的 Reader。
        let mut lines = BufReader::new(stderr).lines();
        // 每次异步等待一行；None 表示 stderr 已关闭。
        while let Some(line) = lines.next_line().await? {
            // 只有启用对应日志级别时才输出 Codex 日志。
            debug!(target: "codex_app_server", message = %line);
        }
        // 明确异步任务的成功类型和错误类型。
        Ok::<(), io::Error>(())
    });

    // 用 BufReader 包装 stdout，满足 handshake 的 AsyncBufRead 要求。
    let mut reader = BufReader::new(stdout);
    // 使用真实 stdout/stdin 完成和 Codex 的握手。
    let initialized = handshake(&mut reader, &mut stdin).await?;
    // 打印握手成功标记。
    println!("handshake=ok");
    // 打印 Codex 用户代理和版本信息。
    println!("user_agent={}", initialized.user_agent);
    // display() 把 PathBuf 转成适合显示的路径。
    println!("codex_home={}", initialized.codex_home.display());
    // 打印平台家族。
    println!("platform_family={}", initialized.platform_family);
    // 打印操作系统。
    println!("platform_os={}", initialized.platform_os);

    // 关闭 Codex stdin，告诉它不会再收到消息。
    drop(stdin);
    // 最多等待 Codex 两秒正常退出。
    let status = match timeout(Duration::from_secs(2), child.wait()).await {
        // 两秒内退出，传播 wait 可能产生的 I/O 错误。
        Ok(wait_result) => wait_result?,
        // 超时后进入强制清理路径。
        Err(_) => {
            // 请求操作系统终止 Codex。
            child.kill().await?;
            // kill 后仍然必须 wait，真正回收子进程。
            child.wait().await?
        }
    };
    // 等待 stderr 异步任务结束，并记录非致命日志错误。
    match stderr_task.await {
        // Task 正常结束且没有 I/O 错误。
        Ok(Ok(())) => {}
        // Task 运行了，但读取 stderr 发生错误。
        Ok(Err(error)) => debug!(%error, "failed while draining Codex stderr"),
        // Task 自身被取消或 Panic。
        Err(error) => debug!(%error, "Codex stderr task failed"),
    }
    // Codex 非成功退出时，不把整个诊断报告成成功。
    if !status.success() {
        // 把退出状态转换成上层可返回的错误。
        return Err(format!("Codex app-server exited with {status}").into());
    }
    // Rust CLI 正常结束。
    Ok(())
}

// 初始化 tracing 日志订阅器。
fn init_tracing() {
    // 优先读取 RUST_LOG；没有设置时使用安全默认值。
    let filter = EnvFilter::try_from_default_env()
        // 默认显示本程序 info，关闭 Codex app-server debug 噪音。
        .unwrap_or_else(|_| EnvFilter::new("bridgehub_codex_spike=info,codex_app_server=off"));
    // 创建文本日志 Subscriber 并注册为全局 Subscriber。
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
```

保存退出后运行：

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p bridgehub-codex-spike -- --help
```

预期：3 个测试通过，帮助信息包含：

```text
--codex-bin <CODEX_BIN>
```

---

# 第 7 章：为什么必须安全回收子进程

当前代码处理三条退出路径：

1. 正常路径：`drop(stdin)` 后 Codex 自己退出，`wait()` 回收。
2. 超时路径：两秒后 `kill()`，再执行 `wait()` 回收。
3. Rust 异常提前退出：`kill_on_drop(true)` 请求终止 Codex。

`kill` 只负责发送终止请求，`wait` 才负责等待并回收进程资源。因此不能只写 `kill`。

stderr Task 持续读取错误输出，是为了避免 Codex 写满操作系统缓冲区后卡住。它是 Tokio Task，不是 Codex 子线程。

---

# 第 8 章：检查完整代码

运行下面命令前，确认你已经保存所有 Neovim Buffer：

```bash
# 检查所有文件是否符合 rustfmt 格式，但不修改。
cargo fmt --all --check

# 检查整个 Workspace 的 Library、Binary 和测试。
# -- 后的 -D warnings 表示把所有警告当成错误。
cargo clippy --workspace --all-targets -- -D warnings

# 运行 Workspace 全部测试。
cargo test --workspace
```

预期：3 个测试通过，没有 Clippy 警告。

---

# 第 9 章：连接真实 Codex

先检查环境：

```bash
rustc --version
cargo --version
which codex
codex --version
```

参考实现使用 Rust 1.96.0 和 Codex 0.144.1。Codex 补丁版本不同不一定是错误；如果协议解析失败，把版本和完整错误发给我。

运行真实握手并检查是否遗留子进程：

```bash
# 保存运行前所有匹配的 Codex app-server 进程 ID。
before=$(pgrep -f "codex app-server.*stdio" | sort || true)

# 启动我们亲手实现的 Rust CLI。
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex

# 保存运行后的匹配进程 ID。
after=$(pgrep -f "codex app-server.*stdio" | sort || true)

# 前后集合必须完全相同，表示本次没有留下新进程。
test "$after" = "$before"
```

已有 Codex app-server 可以同时出现在 `before` 和 `after` 中，这没有问题。我们只检查本次运行是否新增遗留进程。

---

# 第 10 章：回看整个程序

```text
用户启动 Rust CLI
→ Rust 启动 Codex app-server 子进程
→ Rust 取得 stdin、stdout、stderr 管道
→ Rust 发送 initialize 请求
→ Codex 返回 InitializeResponse
→ Rust 解析为 InitializeResponse 结构体
→ Rust 发送 initialized 通知
→ Rust 打印协商信息
→ Rust 关闭 stdin
→ Codex 正常退出或被超时终止
→ Rust wait 回收子进程并退出
```

已经实现：

- Rust Workspace 和独立 Crate。
- 类型化 JSONL 初始化握手。
- 内存伪 Codex 测试。
- 真实 Codex 子进程启动。
- stderr 异步排空。
- 超时、终止和回收。

尚未实现：

- 聊天和多轮对话。
- Thread、Turn 和流式 Item。
- Tool Calling 与 Approval。
- Session 持久化。
- Relay、Connector 和 Web。

## 常见错误先查哪里

| 现象 | 第一检查点 |
|---|---|
| 找不到工具链文件 | 文件名必须是根目录 `rust-toolchain.toml` |
| TOML 解析失败 | 检查 `[workspace]` 和每个 `=` |
| 找不到 Workspace 成员 | 必须是 `tools/codex-spike`，不是 `tool/codex-spike` |
| `mod` 语法错误 | 必须写 `mod tests {` |
| 找不到 `supper` | 正确拼写是 `super` |
| 找不到 `bridgehub_codex_spike` | 检查 Crate `Cargo.toml` 是否存在且名称正确 |
| 找不到 Codex | 运行 `which codex`，确认 `--codex-bin` 路径 |
| JSON 解码失败 | 发回 `codex --version` 和完整错误，不猜字段 |
| Clippy 失败 | 从第一条 Warning 开始修，不跳过 `-D warnings` |

遇到教程未覆盖的错误时，不继续改代码。把下面三项发给我：

1. 你执行的完整命令。
2. 从第一行到最后一行的完整输出。
3. 当前正在编辑的文件绝对路径。
