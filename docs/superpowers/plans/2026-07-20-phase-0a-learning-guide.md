# BridgeHub Phase 0A 手敲学习教程实施计划

> [!WARNING]
> 这是供 Codex 编写教程使用的内部实施计划，不是学习者的操作教程，请不要照着本文件手敲。项目内正文是 `docs/learning/phase-0a-from-scratch.md`；学习者实际阅读 Obsidian 中的《BridgeHub Phase 0A 从零手敲教程》。

> **供执行本计划的 Agent 使用：** 必须使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans`，逐项编写教程。所有步骤使用复选框（`- [ ]`）跟踪。执行者只能编辑学习文档，不能替学习者执行教程中的项目命令或创建项目代码。

> [!NOTE]
> 初版教程已经生成，下面的任务清单现在同时作为维护验收清单使用。仓库初始化等已经发生的历史步骤必须按当前 `learn-develop` 状态解释，不能再次要求学习者执行。

**目标：** 编写一份从零开始的中文手敲教程，让有应用开发经验但不了解进程通信和 Codex app-server 的学习者，通过亲手输入全部命令与完整代码，实现 Phase 0A 初始化握手。

**架构：** 教程以 `/Users/xulei/.dev/bridgehub` 中已经验证的实现作为只读标准答案。正文按“问题 → 约束 → 候选方案 → Codex 的选择及理由 → 代价 → 可观察行为 → 完整命令与代码 → 验证 → 设计回看”递进，严格停在 Phase 0A。项目内版本保存在 `docs/learning/phase-0a-from-scratch.md`；学习者实际阅读的中文版本保存在 Obsidian，并用 WikiLink 连接相关基础笔记。

**技术栈：** Markdown、Rust 1.96、Cargo Workspace、Tokio、process-wrap 9.1、Serde/serde_json、Clap、thiserror、tracing/tracing-subscriber、Codex app-server 0.144.1 stdio JSONL 协议。

---

## 执行边界

- 允许创建或修改：`docs/learning/phase-0a-from-scratch.md`、本设计/计划文档，以及 Obsidian 中对应的 BridgeHub 学习文档。
- 允许读取：本目录中的设计和计划，以及 `/Users/xulei/.dev/bridgehub` 中已验证的 Phase 0A 文件。
- 禁止执行：`git init`、`cargo new`、`cargo test`、`cargo run`、`cargo fmt`、`cargo clippy`、真实 Codex 命令以及教程中展示的任何 Git 命令。
- 禁止创建：`Cargo.toml`、`rust-toolchain.toml`、`.gitignore`、`tools/**`、Rust 源文件和 `Cargo.lock`。
- 学习项目已经由学习者初始化为 Git 仓库。不得切换或重写学习者的分支，不得提交，不得修改其正在手敲的 Rust/TOML 文件。
- 所有示例均使用 `/Users/xulei/.dev/bridge-hub`，不能误写成只读参考目录 `/Users/xulei/.dev/bridgehub`。
- 每次让学习者创建或编辑文件前，必须先给出相对路径、绝对路径、目录树位置和完整 Neovim 命令。
- 不向学习者提出“你觉得应该选哪个架构”之类的开放式工程题。Codex 必须直接选择，并在代码前解释依据；只有产品目标、成本或安全授权无法由技术事实决定时才询问学习者。

## 文件职责

- `docs/learning/phase-0a-from-scratch.md`：项目内可版本化教程，包含第 0–10 章、工程推理、完整命令、全部阶段代码、预期输出、原因和排错方法。
- `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/BridgeHub Phase 0A 从零手敲教程.md`：学习者实际阅读版本；与项目内教程保持技术内容一致，并保留 Obsidian WikiLink。
- `docs/superpowers/specs/2026-07-20-phase-0a-learning-guide-design.md`：教程需求、教学模型和完成标准；只有教学设计经用户确认后才修改。
- `/Users/xulei/.dev/bridgehub/Cargo.toml`：根 Workspace 清单的只读起始基线；最终教程额外包含 `process-wrap`。
- `/Users/xulei/.dev/bridgehub/rust-toolchain.toml`：工具链配置的只读起始基线。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/Cargo.toml`：Crate 清单的只读起始基线；最终教程额外继承 `process-wrap`。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/lib.rs`：握手实现与成功测试的只读起始基线；最终教程额外包含三条失败契约测试。
- `/Users/xulei/.dev/bridgehub/tools/codex-spike/src/main.rs`：真实进程 CLI 的只读起始基线，不是最终权威。教程最终实现还包含已验证修正：显式把 tracing 写入父进程 stderr；使用 `process-wrap` 管理 Node 启动器与原生 Codex 组成的进程树；给握手单独设置期限；先保存握手结果再统一清理；自然退出宽限期使用 `try_wait + sleep`；期限后使用组感知 `start_kill() + wait().await`；stderr Task 限时收拢。
- 本次维护曾在系统临时目录创建验证 Crate；最终代码通过格式、Clippy、6 个测试和真实 Codex Smoke Test。临时目录不是持久依据，可能被系统删除；后续维护应从教程代码块重新构造隔离验证副本，并重新运行证据。学习者仍需亲自在 `bridge-hub` 执行教程命令。

## 设计先于代码的强制规则

初版教程已经包含完整代码和逐行解释，但缺少代码出现前的工程推理。本计划的后续维护必须补齐并长期保留这一层，不能用更多行内注释代替。

每个有意义的实现单元在第一段代码之前，至少回答：

1. 当前缺少什么能力，不做会发生什么。
2. 当前阶段有哪些不能破坏的约束或协议不变量。
3. 有哪些真实可行的候选方案。
4. Codex 选择哪个方案，以及为什么它最适合当前阶段和学习者。
5. 这个选择有什么代价，哪些能力明确推迟。
6. 准备通过什么测试、日志、退出状态或真实输出来证明设计成立。

代码之后必须增加“设计落点”，指出上述选择分别落在哪个文件、类型、函数或测试里。核心理由必须写在主教程中；语法和操作系统原理可以通过 Obsidian WikiLink 延伸。

各阶段必须覆盖的决策问题：

| 阶段 | 必须讲清的设计问题 |
|---|---|
| 项目边界 | 为什么 BridgeHub 调用 Codex app-server，而不是重写 Agent Loop 或解析交互式终端文本 |
| Workspace | 为什么从虚拟 Workspace + 独立 Spike Crate 起步，而不是把全部代码放进根 Package；为什么固定工具链、集中依赖和 Lint |
| 第一个 TDD 循环 | 为什么先保护稳定客户端身份；为什么这是教学切入点而不是完整协议测试 |
| 传输协议 | 为什么本机阶段选择 stdio JSONL；stdout、stderr 和换行分别承担什么职责 |
| 握手 | 为什么使用类型化参数/结果、响应信封、严格 ID 和固定顺序；为什么暂不实现通用 JSON-RPC 路由器 |
| 测试 | 为什么先用内存伪 Codex，再连接真实 Codex；两层测试分别能证明和不能证明什么 |
| 子进程 | 为什么选择 Tokio 异步进程；谁拥有 ChildWrapper、进程树和三条标准流；为什么 stderr 要并发排空 |
| 退出 | 为什么握手与自然退出各有期限；为何先保存结果再统一清理；为何管理进程树而不是直接 Child；为什么关闭 stdin、用 `try_wait + sleep` 给宽限期、超时后组感知 `start_kill` 并最终 `wait`；`KillOnDrop` 为什么只能是直接 Child 的意外析构兜底 |
| 质量门禁 | rustfmt、Clippy、单元测试和 Live Smoke 各自防止哪类问题，为什么不能互相替代 |

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

预期说明：`pwd` 必须输出 `/Users/xulei/.dev/bridge-hub`；目录中已经存在 Cargo 配置、教程和学习者当前手敲的源码，不得假设它是空目录。

- [ ] **步骤 2：确认现有 Git 学习分支**

```bash
git branch --show-current
git status -sb
```

预期当前分支为 `learn-develop`。解释 `.git`、工作区、暂存区和分支，但不得再次运行 `git init`、强制改名或在存在未提交修改时切换分支。`main` 保留标准答案，`learn-develop` 保留手敲过程，两条分支不整体合并。

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

- [ ] **步骤 4：说明阶段提交边界**

```bash
git add .gitignore docs
git status
git commit -m "docs: add Phase 0A learning guide"
```

这组命令只作为最初教程提交的历史示例。当前已有提交历史和未完成源码，不能原样重复执行。后续应在一个学习阶段通过检查后，由学习者选择准确文件并亲自提交；Codex 不替学习者暂存或提交。

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
# [toolchain] 开始声明这个项目使用的 Rust 工具链配置。
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

必须逐行覆盖最终设计中的 `[workspace]`、`members`、Resolver、共享 Package 元数据、8 个依赖及 Feature、Rust Lint 和 Clippy Lint。8 个依赖中包含用于跨平台进程树管理的 `process-wrap`。注释不得改变：

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

### 任务 6：编写第 5 章握手类型与内存测试

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

- [ ] **步骤 5：加入关键失败契约测试**

在成功测试以外，必须使用内存 Reader/Writer 加入并解释三条行为测试：

- 响应前 EOF 返回 `UnexpectedEof`。
- 请求 0 收到 ID 7 的响应时返回 `UnexpectedResponseId { actual: 7 }`。
- 服务端 error 保留 `code` 和 `message`。

断言错误变体，不比较显示字符串；不使用被 Lint 禁止的 `unwrap` 或 `expect`。解释为什么当前不为每个 Serde/I/O 内部分支堆测试，以及什么时候应增加测试。

- [ ] **步骤 6：加入绿色验证**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

加入握手实现后先看到 2 个测试通过；加入三条失败契约后预期 5 个测试通过。分别解释格式化、格式检查、静态检查和测试。

### 任务 7：编写第 6–7 章真实进程树与安全退出审计

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

必须完整覆盖经过临时 Crate 验证的最终 `main.rs`，不能机械复制旧基线。按下列顺序解释：

1. 标准库与第三方导入。
2. `Args` 与 `--codex-bin`。
3. `#[tokio::main]`。
4. 初始化 tracing。
5. `CommandWrap`、参数和三条 `Stdio::piped`。
6. Unix `ProcessGroup::leader()`、Windows `JobObject`、`KillOnDrop` 的不同职责与 `spawn()`。
7. 使用 `take_child_pipes` 一次性取得 stdin/stdout/stderr 所有权；不变量失败也先整组清理。
8. `tokio::spawn` 创建始终排空 stderr 的 Task；`RUST_LOG` 只控制显示。
9. 用 `BufReader` 包装 stdout，并用 10 秒 `timeout` 调用 `handshake`。
10. 保存 `handshake_result`，不在清理前使用 `?` 提前返回。
11. `drop(stdin)` 和 `drop(reader)` 关闭父进程持有的协议管道。
12. `wait_or_kill` 使用 `try_wait + sleep` 给两秒自然退出宽限期。
13. 期限后使用进程组 / Job Object 的 `start_kill`，再用 `wait` 回收；解释为何不取消组感知 `wait` Future。
14. `finish_stderr_task` 限时等待，必要时只取消日志 Task。
15. 先按错误优先级传播握手与清理错误，再检查非成功退出状态。
16. 只有全部成功后才打印 5 个结果字段。
17. `init_tracing` 显式使用 `.with_writer(io::stderr)` 和默认 Filter。

不得将 Tokio Task 错称为 Codex 子线程。必须明确：Codex 是操作系统子进程，stderr Reader 是 Tokio 异步任务。

- [ ] **步骤 3：加入 CLI 静态验证**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p bridgehub-codex-spike -- --help
```

预期：6 个测试通过；帮助信息包含 `--codex-bin <CODEX_BIN>`。

- [ ] **步骤 4：解释统一生命周期主干**

- 正常响应、协议错误和 10 秒握手超时都先保存结果，再关闭管道并进入同一清理路径。
- 正常退出路径：关闭 stdin，Codex 自己退出，`try_wait` 取得状态。
- 卡住路径：2 秒后整组 `start_kill`，随后 `wait` 回收并取得退出状态。
- 意外析构：`KillOnDrop` 只尝试终止直接 Child，不能替代进程组 / Job Object 的显式清理。
- stderr 始终持续读取；进程结束后最多再等待 1 秒，必要时取消这个日志 Task。
- 握手错误优先于同时发生的清理错误；握手成功时清理错误才成为最终错误。

### 任务 8：编写第 8–10 章质量检查、真实验证、排错与复习

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

必须覆盖：当前目录错误、Rust 工具链问题、Workspace 成员错误、Crate 名称与 `use` 不匹配、预期失败没有出现、JSON 字段不匹配、Codex 路径错误、Codex 未认证、app-server 提前退出、握手超时、超过自然退出宽限期后进入强制终止路径，以及 Clippy 的 `unwrap`/`expect`/未使用内容错误。

每项都写：识别信号、最可能原因、第一条检查命令、何时停止并发回完整输出。不得建议删除 `Cargo.lock`、强制终止无关 Codex 进程或通用使用 `sudo`。

- [ ] **步骤 4：加入完整执行顺序复习**

```text
用户启动 Rust CLI
→ Rust 用 ProcessGroup / JobObject 启动并拥有 Codex 进程树
→ Rust 取得 stdin/stdout/stderr 管道并排空 stderr
→ Rust 发送 initialize
→ Codex 返回 InitializeResponse
→ Rust 发送 initialized
→ Rust 保存握手结果并关闭协议管道
→ Codex 正常退出，或在宽限期后被整组终止
→ Rust wait 回收进程并收拢 stderr Task
→ 全部成功后打印协商信息，否则按错误优先级退出
```

- [ ] **步骤 5：加入能力边界清单**

“已经实现”只能包含 Workspace、类型化握手、内存测试、真实进程、日志排空和安全退出。“尚未实现”必须包含聊天、Thread、Turn、Tool Calling、Session 持久化、Relay、Connector 和 Web。

### 任务 9：教程自检与交付

**文件：**
- 修改：`docs/learning/phase-0a-from-scratch.md`

- [ ] **步骤 1：检查设计覆盖**

逐项对照 `docs/superpowers/specs/2026-07-20-phase-0a-learning-guide-design.md` 第 3、6、7、8、9、10、11 节，确认教程有对应章节；特别检查每个实现单元是否在代码前完成工程推理，发现缺口时直接补入。

- [ ] **步骤 2：检查最终代码完整性**

人工逐行对照起始基线、教程最终代码块和当前依赖/Codex 源码依据：

```text
/Users/xulei/.dev/bridgehub/Cargo.toml
/Users/xulei/.dev/bridgehub/rust-toolchain.toml
/Users/xulei/.dev/bridgehub/tools/codex-spike/Cargo.toml
/Users/xulei/.dev/bridgehub/tools/codex-spike/src/lib.rs
/Users/xulei/.dev/bridgehub/tools/codex-spike/src/main.rs
```

从教程代码块重新构造一次隔离验证副本，确认最终阶段没有少任何有效行；协议标识符、JSON 字段和命令参数与直接核对过的 Codex 契约一致；`process-wrap`、超时和统一清理这些已记录修正通过重新验证。不要依赖可能消失的 `/tmp` 路径，也不要为了“与旧标准答案逐字相同”撤销有源码依据的修正。

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

- [ ] **步骤 6：同步 Obsidian 阅读版本**

把技术正文同步到 `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/BridgeHub Phase 0A 从零手敲教程.md`，保留其中已有的 WikiLink、Neovim 补充说明和学习者更正。检查每个新增 WikiLink 的目标存在；重要关联必须有反向链接。

## 完成标准

- 项目内教程 `docs/learning/phase-0a-from-scratch.md` 与 Obsidian 阅读版本的技术正文保持一致；Obsidian 可以额外保留 WikiLink 和个性化操作说明。
- 第 0–10 章齐全，并符合固定学习单元结构。
- 每个有意义的配置或代码阶段都在代码之前说明问题、约束、候选方案、Codex 的最终选择、理由、代价和验证方式。
- 代码之后能把设计决定映射到具体文件、类型、函数和测试。
- 所有命令完整、可手敲、参数有中文解释。
- 所有最终代码与直接核对过的 Codex/依赖契约及隔离验证结果一致；相对旧起始基线的差异必须像 `process-wrap`、握手期限和统一清理一样被明确记录，且不扩大阶段范围。
- 每一行新代码在代码块内部有中文注释或紧邻的结构解释。
- 每个文件在创建前都有相对路径、绝对路径、目录树位置和 Neovim 命令。
- TDD 红色和绿色阶段顺序正确，预期错误与测试数量明确。
- 技术决策由 Codex 直接完成并解释，没有要求学习者自行设计或补齐代码。
- Codex 协议和依赖行为的关键结论有当前官方文档或源码依据，并标出需要重新核对的升级边界。
- 没有混入 Phase 0B 或完整 BridgeHub 架构。
- 文档执行者没有替学习者运行任何项目命令或创建项目代码。
- 文档执行者没有切换、重写或提交学习者当前分支，也没有修改学习者正在手敲的项目代码。
