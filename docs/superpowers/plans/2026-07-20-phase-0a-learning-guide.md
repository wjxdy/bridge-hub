# BridgeHub Phase 0A 手敲学习教程 Implementation Plan

> [!WARNING]
> 这是供 Codex 编写教程使用的内部实施计划，不是学习者的操作教程，请不要照着本文件手敲。正式教程请打开 `docs/learning/phase-0a-from-scratch.md`。

> **供执行本计划的 Agent 使用：** 必须使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans`，逐项编写教程。所有步骤使用复选框（`- [ ]`）跟踪。执行者只能编辑学习文档，不能替学习者执行教程中的项目命令或创建项目代码。

**目标：** 编写一份从零开始的中文手敲教程，让有应用开发经验但不了解进程通信和 Codex app-server 的学习者，通过亲手输入全部命令与完整代码，实现 Phase 0A 初始化握手。

**架构：** 教程以 `/Users/xulei/.dev/bridgehub` 中已经验证的实现作为只读标准答案，唯一交付文件为 `docs/learning/phase-0a-from-scratch.md`。正文按“基础概念 → 完整命令 → 逐行注释代码 → 预期结果 → 原因 → 排错”递进，严格停在 Phase 0A。

**技术栈：** Markdown、Rust 1.96、Cargo Workspace、Tokio、Serde/serde_json、Clap、thiserror、tracing/tracing-subscriber、Codex app-server 0.144.1 stdio JSONL 协议。

---

## 执行边界

- 允许创建或修改：`docs/learning/phase-0a-from-scratch.md`。
- 允许读取：本目录中的设计和计划，以及 `/Users/xulei/.dev/bridgehub` 中已验证的 Phase 0A 文件。
- 禁止执行：`git init`、`cargo new`、`cargo test`、`cargo run`、`cargo fmt`、`cargo clippy`、真实 Codex 命令以及教程中展示的任何 Git 命令。
- 禁止创建：`Cargo.toml`、`rust-toolchain.toml`、`.gitignore`、`tools/**`、Rust 源文件和 `Cargo.lock`。
- 目标目录不是 Git 仓库。不要初始化仓库，也不要提交文档；学习者会亲自执行 `git init`。
- 所有示例均使用 `/Users/xulei/.dev/bridge-hub`，不能误写成只读参考目录 `/Users/xulei/.dev/bridgehub`。
- 每次让学习者创建或编辑文件前，必须先给出相对路径、绝对路径、目录树位置和完整 Neovim 命令。

## 文件职责

- `docs/learning/phase-0a-from-scratch.md`：唯一教程正文，包含第 0–10 章、完整命令、全部阶段代码、预期输出、原因和排错方法。
- `docs/superpowers/specs/2026-07-20-phase-0a-learning-guide-design.md`：教程需求和完成标准，只读。
- `/Users/xulei/.dev/bridgehub/Cargo.toml`：最终根 Workspace 清单的只读标准答案。
- `/Users/xulei/.dev/bridgehub/rust-toolchain.toml`：最终工具链文件的只读标准答案。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/Cargo.toml`：最终 Crate 清单的只读标准答案。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/lib.rs`：最终握手实现和测试的只读标准答案。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/main.rs`：最终真实子进程 CLI 的只读标准答案。

### 任务 1：创建教程骨架和第 0 章学习地图

**文件：**
- 新建：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：创建教程标题、使用方式和边界声明**

正文开头必须明确：这是“理解后亲手输入”的教程，不是复制粘贴教程；学习者负责执行所有命令和创建所有项目文件；遇到错误时停止并发回完整命令和输出；每章有完成检查框。

当前能力只能写成：启动 Codex、完成握手、打印信息、安全退出。必须明确尚未实现聊天、Thread、Turn、Tool Calling、Session、Relay、Connector 或 Web。

- [ ] **步骤 2：写出最终可见结果**

教程必须展示最终命令：

```bash
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
```

展示输出形状：

```text
handshake=ok
user_agent=...
codex_home=/...
platform_family=unix
platform_os=macos
```

解释 `...` 是机器相关真实值，不要求逐字相同。

- [ ] **步骤 3：用应用开发类比解释两个进程**

依次解释：可执行文件是硬盘上的程序；进程是程序启动后的实例；Rust CLI 是父进程；Codex app-server 是独立子进程。使用 Web 服务启动 `ffmpeg` 作类比：BridgeHub 不实现 Agent，只启动并控制 Codex。

本章结尾必须加入：

```markdown
- [ ] 我知道 Rust 和 Codex 是两个独立进程。
- [ ] 我知道 Phase 0A 结束时程序会主动关闭 Codex。
```

### 任务 2：编写第 1 章目录与 Git 准备

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：解释命令提示符与当前目录**

展示并逐项解释，但不得执行：

```bash
cd /Users/xulei/.dev/bridge-hub
pwd
ls -la
```

预期说明：`pwd` 必须输出 `/Users/xulei/.dev/bridge-hub`；此时目录中只有教程文档。

- [ ] **步骤 2：指导学习者亲自初始化 Git**

```bash
git init
git branch -M main
git status
```

解释 `init`、`.git`、`branch -M main`、工作区和未跟踪文件。不得声称命令已经运行。

- [ ] **步骤 3：指导创建 `.gitignore`**

必须先给出：

```text
相对路径：.gitignore
绝对路径：/Users/xulei/.dev/bridge-hub/.gitignore
所在位置：项目根目录
Neovim 命令：nvim .gitignore
```

要求学习者亲手创建，完整内容为：

```gitignore
# Cargo 默认把所有编译产物放进 target 目录。
# 编译产物可以重新生成，所以不提交到 Git。
/target/
```

解释开头 `/` 只匹配项目根目录下的 `target`。

- [ ] **步骤 4：加入第一个提交步骤**

```bash
git add .gitignore docs
git status
git commit -m "docs: add Phase 0A learning guide"
```

说明教程文件由 Codex 准备，但提交动作由学习者完成。

### 任务 3：编写第 2 章 Cargo Workspace

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：解释 Cargo、Workspace、Package 和 Crate**

使用对应表：Cargo 类似 npm、构建器和测试入口的组合；Workspace 类似 monorepo 根；Package 是带 `Cargo.toml` 的构建单元；Crate 是编译器一次编译的库或程序；dependency 类似 npm dependency。明确类比不代表行为完全相同。

- [ ] **步骤 2：写出目录创建命令**

```bash
mkdir -p tools/codex-spike/src
```

逐项解释 `mkdir`、`-p` 和目录层级。不使用 `cargo new`，因为学习者要亲手创建每个文件。

在创建文件前展示本章目标目录树：

```text
/Users/xulei/.dev/bridge-hub/
├── Cargo.toml
├── rust-toolchain.toml
└── tools/
    └── codex-spike/
        ├── Cargo.toml
        └── src/
```

- [ ] **步骤 3：加入完整注释版 `rust-toolchain.toml`**

必须先明确：

```text
相对路径：rust-toolchain.toml
绝对路径：/Users/xulei/.dev/bridge-hub/rust-toolchain.toml
所在位置：项目根目录，与根 Cargo.toml 同级
```

学习者从项目根目录执行：

```bash
nvim rust-toolchain.toml
```

如果学习者已经打开了一个无文件名 Buffer，则使用：

```vim
:w /Users/xulei/.dev/bridge-hub/rust-toolchain.toml
```

解释 `rust-toolchain.toml` 放在 Workspace 根目录后，Rust 工具链管理器会让这个目录及其子目录中的 Cargo 命令使用同一工具链。

```toml
# [toolchain] 开始声明这个项目使用的 Rust 工具链配置。i
[toolchain]
# 固定 Rust 1.96.0，避免不同机器使用不同编译器版本。
channel = "1.96.0"
# 同时安装 Clippy 静态检查器和 rustfmt 格式化工具。
components = ["clippy", "rustfmt"]
# minimal 只安装完成本项目所需的最小组件。
profile = "minimal"
```

- [ ] **步骤 4：加入完整注释版根 `Cargo.toml`**

必须先给出：

```text
相对路径：Cargo.toml
绝对路径：/Users/xulei/.dev/bridge-hub/Cargo.toml
所在位置：项目根目录
Neovim 命令：nvim Cargo.toml
```

必须逐行覆盖标准答案中的 `[workspace]`、`members`、Resolver、共享 Package 元数据、7 个依赖及 Feature、Rust Lint 和 Clippy Lint。注释不得改变：

```toml
members = ["tools/codex-spike"]
resolver = "3"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
```

- [ ] **步骤 5：加入完整注释版 Crate `Cargo.toml`**

必须先给出：

```text
相对路径：tools/codex-spike/Cargo.toml
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/Cargo.toml
所在位置：tools/codex-spike 目录
Neovim 命令：nvim tools/codex-spike/Cargo.toml
```

逐行覆盖参考 Crate 清单，解释 `bridgehub-codex-spike` 为什么在 Rust `use` 中变为 `bridgehub_codex_spike`、`.workspace = true` 的继承语义和统一 Lint。

- [ ] **步骤 6：加入人工检查点**

```bash
find . -maxdepth 4 -type f | sort
```

预期至少看到 `.gitignore`、根 `Cargo.toml`、教程、`rust-toolchain.toml` 和 Crate `Cargo.toml`。

### 任务 4：编写第 3 章第一个 TDD 循环

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：解释测试先行的目的**

用核心规则开头：测试先描述“代码应该做什么”，观察它因能力不存在而失败，再加入最小实现。

- [ ] **步骤 2：加入预期失败的完整 `lib.rs`**

必须先给出：

```text
相对路径：tools/codex-spike/src/lib.rs
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/src/lib.rs
所在位置：tools/codex-spike/src 目录
Neovim 命令：nvim tools/codex-spike/src/lib.rs
```

```rust
// #[cfg(test)] 表示这个模块只在运行测试时参与编译。
#[cfg(test)]
// mod 创建一个名为 tests 的模块。
mod tests {
    // #[test] 把下方函数注册为同步单元测试。
    #[test]
    // 测试名称说明要保护的行为：客户端身份必须保持稳定。
    fn client_identity_is_stable() {
        // super 指向 tests 模块的上一层，也就是 Library 根模块。
        // 这里故意调用尚不存在的 client_name，让测试先失败。
        assert_eq!(super::client_name(), "bridgehub");
    }
}
```

- [ ] **步骤 3：展示红色验证命令**

```bash
cargo test -p bridgehub-codex-spike client_identity_is_stable
```

预期：编译失败，并包含 `cannot find function client_name in module super`。解释这是正确的红色阶段。

- [ ] **步骤 4：加入最小实现**

```rust
// pub 让其他模块可以调用这个函数。
// const fn 表示函数可以在允许的上下文中参与编译期求值。
// &'static str 表示字符串字面量在整个程序生命周期都有效。
pub const fn client_name() -> &'static str {
    // 最后一行没有分号，因此它是函数返回值。
    "bridgehub"
}
```

要求放在测试模块之前。

- [ ] **步骤 5：展示绿色验证和提交命令**

```bash
cargo fmt --all
cargo test -p bridgehub-codex-spike client_identity_is_stable
git add Cargo.toml Cargo.lock rust-toolchain.toml tools/codex-spike .gitignore
git commit -m "build: initialize Rust workspace"
```

预期：1 个测试通过。说明首次构建会下载依赖并生成 `Cargo.lock`，所以可能更慢。

### 任务 5：编写第 4 章本机程序通信基础

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：解释标准流**

先从键盘和屏幕解释：

```text
键盘 → stdin → 程序 → stdout → 屏幕
                       stderr → 屏幕
```

再转换为当前项目：

```text
Rust 写入 → Codex stdin
Rust 读取 ← Codex stdout
Rust 读取 ← Codex stderr
```

明确：Codex 是操作系统子进程，不是 Rust 子线程。

- [ ] **步骤 2：解释管道**

说明管道是操作系统提供的本机字节通道。它只负责运输字节，不理解 JSON、请求或响应；消息意义来自 Rust 与 Codex 共同遵守的协议。

- [ ] **步骤 3：解释 JSONL 消息边界**

```text
{"method":"initialize","id":0,"params":{...}}\n
{"id":0,"result":{...}}\n
{"method":"initialized"}\n
```

解释 `\n` 是换行符，一行是一条 JSON 消息；请求有 ID 并等待响应，通知没有 ID 且不等待响应。

- [ ] **步骤 4：解释同步等待与异步等待**

用 Web 请求等待数据库结果类比 `.await`：任务暂时让出执行机会，数据到达后继续；不能把 `.await` 解释为创建新线程，也不能说程序在持续占用 CPU 轮询。

### 任务 6：编写第 5–6 章握手类型与内存测试

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：先安排握手测试的红色阶段**

实际手敲顺序必须是：先加入测试和测试所需导入，再运行下面命令；此时 `InitializeResponse` 和 `handshake` 尚不存在。

```bash
cargo test -p bridgehub-codex-spike handshake_sends_initialize_then_initialized
```

预期：编译失败，错误指向尚未定义的 `InitializeResponse` 和 `handshake`。不能先展示实现导致测试第一次运行就通过。

- [ ] **步骤 2：逐行注释完整内存伪服务端测试**

测试代码必须与 `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/lib.rs` 中 `handshake_sends_initialize_then_initialized` 一致，解释：

- `tokio::io::duplex(8 * 1024)` 创建内存双向字节通道。
- `split` 把每一端拆成 Reader 和 Writer。
- `tokio::spawn` 并发运行伪 Codex。
- 伪 Codex 读取并断言 `initialize`。
- 伪 Codex 返回固定响应 JSON。
- 伪 Codex 读取并断言 `initialized`。
- 客户端将 JSON 转为 `InitializeResponse`。
- `fake_server.await??` 分别传播 Join 错误和任务内部错误。

不得省略测试中的导入、类型注解、断言、换行写入或 `flush`。

- [ ] **步骤 3：按依赖顺序加入完整握手实现**

第 5 章必须按以下顺序展示参考 `lib.rs` 的最终代码，每一行新代码在代码块内部有中文解释：

1. `use` 导入。
2. `client_name`。
3. `HandshakeError` 的 6 类错误。
4. `InitializeResponse`。
5. `Request<T>` 与 `Notification`。
6. `InitializeParams`、`ClientInfo`、`InitializeCapabilities`。
7. `ResponseEnvelope` 与 `RpcError`。
8. `handshake<R, W>` 和泛型边界。
9. 构造并发送 `initialize`。
10. 读取、解析和校验响应。
11. 发送 `initialized`。
12. `write_json_line`。

不得省略任何有效代码行。`serde(rename_all = "camelCase")`、`#[serde(default)]`、`Unpin`、`?`、`env!`、`Serialize` 和 `Deserialize` 必须在第一次出现处解释。

- [ ] **步骤 4：加入错误对应表**

| 错误 | 发生条件 |
|---|---|
| `UnexpectedEof` | Codex 在初始化完成前关闭 stdout |
| `UnexpectedResponseId` | 响应不是当前请求 ID 0 |
| `MissingResult` | 响应既没有 result，也没有 error |
| `Server` | Codex 明确返回 RPC 错误 |
| `Io` | stdin/stdout 读写失败 |
| `Json` | JSON 语法错误或字段类型不匹配 |

- [ ] **步骤 5：加入绿色验证**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

预期：2 个测试通过。分别解释格式化、格式检查、静态检查和测试。

### 任务 7：编写第 7–8 章真实子进程与安全退出

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：加入启动参数的同步测试**

先展示并逐行解释测试：

```rust
#[test]
fn app_server_arguments_use_the_supported_stdio_transport() {
    assert_eq!(
        super::app_server_args(),
        ["app-server", "--listen", "stdio://"]
    );
}
```

要求先运行并观察缺少 `app_server_args` 的预期失败，再加入并逐行解释：

```rust
pub const fn app_server_args() -> [&'static str; 3] {
    ["app-server", "--listen", "stdio://"]
}
```

- [ ] **步骤 2：逐段注释完整 `main.rs`**

必须先给出：

```text
相对路径：tools/codex-spike/src/main.rs
绝对路径：/Users/xulei/.dev/bridge-hub/tools/codex-spike/src/main.rs
所在位置：tools/codex-spike/src 目录，与 lib.rs 同级
Neovim 命令：nvim tools/codex-spike/src/main.rs
```

必须完整覆盖 `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/main.rs`，按下列顺序解释：

1. 标准库与第三方导入。
2. `Args` 与 `--codex-bin`。
3. `#[tokio::main]`。
4. 初始化 tracing。
5. `Command::new`、参数和三条 `Stdio::piped`。
6. `kill_on_drop(true)` 与 `spawn()`。
7. 使用 `take()` 取得 stdin/stdout/stderr 所有权。
8. `tokio::spawn` 创建 stderr 排空任务。
9. 用 `BufReader` 包装 stdout 并调用 `handshake`。
10. 打印 5 个结果字段。
11. `drop(stdin)` 表示不会再发送消息。
12. 两秒 `timeout`。
13. 超时后的 `kill` 与 `wait`。
14. 等待 stderr Task。
15. 检查非成功退出状态。
16. `init_tracing` 的默认 Filter。

不得将 Tokio Task 错称为 Codex 子线程。必须明确：Codex 是操作系统子进程，stderr Reader 是 Tokio 异步任务。

- [ ] **步骤 3：加入 CLI 静态验证**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p bridgehub-codex-spike -- --help
```

预期：3 个测试通过；帮助信息包含 `--codex-bin <CODEX_BIN>`。

- [ ] **步骤 4：解释四条生命周期路径**

- 正常路径：关闭 stdin，Codex 自己退出，`wait` 回收。
- 卡住路径：2 秒后 `kill`，随后仍然 `wait` 回收。
- Rust 提前退出：`kill_on_drop(true)` 请求终止 Codex。
- stderr 持续读取：避免错误输出缓冲区写满导致阻塞。

### 任务 8：编写第 9–10 章真实验证、排错与复习

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：加入前置条件检查**

```bash
rustc --version
cargo --version
which codex
codex --version
```

说明参考环境是 Rust 1.96.0 和 Codex 0.144.1，但实际 Codex 补丁版本可能不同。协议字段不一致时停止并提交完整版本与错误，不自行猜测协议。

- [ ] **步骤 2：加入真实握手和进程回收命令**

```bash
before=$(pgrep -f "codex app-server.*stdio" | sort || true)
cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
after=$(pgrep -f "codex app-server.*stdio" | sort || true)
test "$after" = "$before"
```

逐行解释命令替换、`pgrep -f`、`sort`、`|| true` 和字符串比较。说明已有 Codex app-server 进程可以同时出现在前后结果中；要求只是本次运行不新增遗留进程。

- [ ] **步骤 3：加入完整排错表**

必须覆盖：当前目录错误、Rust 工具链问题、Workspace 成员错误、Crate 名称与 `use` 不匹配、预期失败没有出现、JSON 字段不匹配、Codex 路径错误、Codex 未认证、app-server 提前退出、退出超时，以及 Clippy 的 `unwrap`/`expect`/未使用内容错误。

每项都写：识别信号、最可能原因、第一条检查命令、何时停止并发回完整输出。不得建议删除 `Cargo.lock`、强制终止无关 Codex 进程或通用使用 `sudo`。

- [ ] **步骤 4：加入完整执行顺序复习**

```text
用户启动 Rust CLI
→ Rust 启动 Codex app-server 子进程
→ Rust 建立 stdin/stdout/stderr 管道
→ Rust 发送 initialize
→ Codex 返回 InitializeResponse
→ Rust 发送 initialized
→ Rust 打印协商信息
→ Rust 关闭 stdin
→ Codex 正常退出或被超时终止
→ Rust wait 回收子进程并退出
```

- [ ] **步骤 5：加入能力边界清单**

“已经实现”只能包含 Workspace、类型化握手、内存测试、真实进程、日志排空和安全退出。“尚未实现”必须包含聊天、Thread、Turn、Tool Calling、Session 持久化、Relay、Connector 和 Web。

### 任务 9：教程自检与交付

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：检查设计覆盖**

逐项对照 `docs/superpowers/specs/2026-07-20-phase-0a-learning-guide-design.md` 第 3、7、8、9、10、11 节，确认教程有对应章节；发现缺口时直接补入。

- [ ] **步骤 2：检查最终代码完整性**

人工逐行对照：

```text
/Users/xulei/.dev/bridgehub/Cargo.toml
/Users/xulei/.dev/bridgehub/rust-toolchain.toml
/Users/xulei/.dev/bridgehub/tools/codex-spike/Cargo.toml
/Users/xulei/.dev/bridgehub/tools/codex-spike/src/lib.rs
/Users/xulei/.dev/bridgehub/tools/codex-spike/src/main.rs
```

确认最终阶段没有少任何有效行，也没有改动标识符、JSON 字段、依赖 Feature、错误文本或命令参数。

- [ ] **步骤 3：扫描不完整内容**

只扫描文档，不运行项目命令：

```bash
rg -n '\b(TBD|FIXME|XXX)\b|待补充|自行实现|省略代码|其余类似' docs/learning/phase-0a-from-scratch.md
```

预期：没有输出。

- [ ] **步骤 4：检查章节与代码围栏**

```bash
rg -n '^#{1,4} ' docs/learning/phase-0a-from-scratch.md
```

确认第 0–10 章均存在、代码围栏闭合、所有命令明确标为学习者执行。

- [ ] **步骤 5：检查路径与文件边界**

确认操作目录始终是 `/Users/xulei/.dev/bridge-hub`；`/Users/xulei/.dev/bridgehub` 只作为标准答案来源；除教程正文外没有创建项目文件；没有执行 `git init`、Cargo、Git、测试或 Codex 项目命令。

## 完成标准

- 生成唯一教程正文 `docs/learning/phase-0a-from-scratch.md`。
- 第 0–10 章齐全，并符合固定学习单元结构。
- 所有命令完整、可手敲、参数有中文解释。
- 所有最终代码与已经验证的 Phase 0A 标准答案一致。
- 每一行新代码在代码块内部有中文注释或紧邻的结构解释。
- 每个文件在创建前都有相对路径、绝对路径、目录树位置和 Neovim 命令。
- TDD 红色和绿色阶段顺序正确，预期错误与测试数量明确。
- 没有要求学习者自行设计或补齐代码。
- 没有混入 Phase 0B 或完整 BridgeHub 架构。
- 文档执行者没有替学习者运行任何项目命令或创建项目代码。
- 目标目录仍由学习者自行执行 `git init`。
