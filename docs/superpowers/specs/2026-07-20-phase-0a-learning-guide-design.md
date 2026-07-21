# BridgeHub Phase 0A 手敲学习教程设计

> [!WARNING]
> 这是教程设计说明，不是学习者的操作教程。正式教程请打开 `docs/learning/phase-0a-from-scratch.md`。

## 1. 背景

学习者有软件开发和 Web 应用开发经验，但暂时不熟悉操作系统进程、标准输入输出、管道、异步运行时和 Codex app-server 协议。

学习者的时间成本较高，不希望通过开放式练习自行探索实现方案，而是希望直接采用经过验证的最佳实践代码，通过亲手输入、运行和观察结果建立理解。

## 2. 学习环境

- 手敲项目目录：`/Users/xulei/.dev/bridge-hub`
- 已验证参考项目：`/Users/xulei/.dev/bridgehub`
- 学习者在手敲项目目录中亲自执行 `git init`、Cargo 命令、Git 命令、测试和真实 Codex 验证。
- Codex 不代替学习者执行教程中的项目命令，不代替学习者创建 Cargo/Rust 项目文件，也不代替学习者输入项目代码。
- Codex 只负责在手敲目录中维护教程和相关学习文档。

两个目录必须保持独立。`bridgehub` 作为已经验证的标准答案保留，`bridge-hub` 作为学习者从零实现的练习项目。

## 3. 教程目标

学习者按照教程完成后，应当能够亲手实现并解释以下闭环：

1. 使用 Cargo 建立 Rust Workspace 和独立 Crate。
2. 理解程序文件、进程、父进程和子进程的区别。
3. 理解 stdin、stdout、stderr 和本机管道的作用。
4. 理解 JSON、JSONL、请求、响应和通知的区别。
5. 使用 Tokio 启动 Codex app-server 子进程。
6. 发送 `initialize` 请求并解析类型化响应。
7. 发送 `initialized` 通知，完成协议握手。
8. 异步读取 Codex stderr，避免子进程因缓冲区写满而阻塞。
9. 正常关闭 stdin，并在宽限期后安全终止、等待和回收子进程。
10. 使用测试、格式检查和 Clippy 验证实现。

## 4. 范围

教程只覆盖已经验证的 Phase 0A：Codex app-server 初始化握手。

教程不包含：

- Thread 创建与恢复。
- Turn 启动与多轮对话。
- 流式 Item 和 Tool Calling。
- Approval 和 Interrupt。
- Session 持久化。
- Local Connector、Public Relay 和 Web 页面。
- 录音、文件或图片上传。

这些内容必须在 Phase 0A 被学习者独立完成和理解后，再通过后续教程逐步加入。

## 5. 交付物

主教程文件：

```text
docs/learning/phase-0a-from-scratch.md
```

教程使用中文说明。Rust 标识符、Cargo 命令、文件路径、JSON 字段和 Codex API 名称保持英文，避免与真实技术接口不一致。

## 6. 教学方式

教程采用“完整答案、分段手敲、立即验证”的方式。

每个学习单元固定包含：

1. 本单元要实现的一个小目标。
2. 开始前必须理解的基础概念。
3. 学习者亲自输入的完整命令。
4. 命令中每个参数的中文解释。
5. 学习者亲自输入的完整代码。
6. 代码内部的逐行或逐语句中文注释。
7. 执行后应该看到的关键输出。
8. 该结果出现的原因。
9. 常见错误、检查位置和修复方向。
10. 完成检查框，方便中断后继续。

每次创建或修改文件之前，教程必须先单独写明：

- 文件相对于项目根目录的位置。
- 文件的绝对路径。
- 该文件在目录树中的层级。
- 使用 Neovim 打开或创建它的完整命令。

不能只写“创建某文件”，也不能假设学习者知道“根目录”具体指哪里。

教程不设置需要学习者自行设计答案的开放式作业。可以加入简短的理解检查，但答案必须紧随其后，不增加不必要的思考等待。

## 7. 代码呈现规则

### 7.1 完整可运行

每个代码阶段都必须能直接编译，或者被明确标记为 TDD 中“预期编译失败”的阶段。不能使用省略号、伪代码、`TODO` 或要求学习者自行补全的空白。

### 7.2 注释放在代码内部

Rust 使用 `//`，TOML 和 Shell 使用 `#`。每一行新代码都必须在代码块内部获得解释；只包含括号或符号的结构行，也要在它附近解释其作用。注释直接解释下一行代码或当前参数，使学习者可以一边手敲一边理解。

示例：

```rust
// pub 表示其他模块可以调用这个函数。
// const fn 表示这个函数可以在编译期求值。
// -> &'static str 表示返回值在整个程序运行期间都有效。
pub const fn client_name() -> &'static str {
    // Rust 会把最后一个没有分号的表达式作为函数返回值。
    "bridgehub"
}
```

### 7.3 不用术语解释术语

第一次出现专业词汇时，必须先用应用开发者熟悉的概念解释，再给出正式定义。例如先将 stdin/stdout 类比为本机程序之间的请求和响应方向，再解释它们是操作系统提供的标准流。

### 7.4 保留工程最佳实践

为降低理解难度，教程可以拆小步骤，但最终实现不得故意使用错误或过时的工程方式。需要保留：

- Rust 2024 Edition 和固定工具链。
- Workspace 依赖集中管理。
- 禁止 `unsafe`、`unwrap` 和 `expect`。
- Serde 类型化序列化与反序列化。
- thiserror 类型化错误。
- Tokio 异步进程与 I/O。
- tracing 日志。
- `kill_on_drop`、退出宽限期和 `wait` 回收。
- 测试先失败、最小实现后通过的 TDD 过程。

## 8. 教程章节

### 第 0 章：建立学习地图

- 说明当前只实现什么。
- 解释最终可以看到的运行结果。
- 区分 Rust 程序和 Codex 程序。

### 第 1 章：准备目录和 Git

- 检查当前位置。
- 亲自执行 `git init`。
- 创建第一份 `.gitignore`。
- 解释工作区、暂存区和提交。

### 第 2 章：认识 Cargo 与 Rust Workspace

- 在项目根目录创建 `/Users/xulei/.dev/bridge-hub/rust-toolchain.toml`。
- 在项目根目录创建 `/Users/xulei/.dev/bridge-hub/Cargo.toml`。
- 创建 `/Users/xulei/.dev/bridge-hub/tools/codex-spike/Cargo.toml`。
- 解释 Workspace、Package、Crate 和依赖。

### 第 3 章：第一个 TDD 循环

- 创建 `lib.rs`。
- 先写 `client_identity_is_stable` 测试。
- 观察缺少函数导致的预期编译失败。
- 添加最小 `client_name` 实现。
- 观察测试变为通过。

### 第 4 章：理解本机程序通信

- 程序文件与进程。
- 父进程与子进程。
- stdin、stdout、stderr。
- 管道只传输字节，不理解 JSON。
- JSONL 如何划分消息边界。

### 第 5 章：定义 Codex 握手类型

- 请求、响应和通知。
- Serde 的 `Serialize`、`Deserialize` 和字段重命名。
- `InitializeParams`、`InitializeResponse` 和错误类型。

### 第 6 章：使用内存伪 Codex 测试握手

- `tokio::io::duplex` 的作用。
- 为什么先测试而不是直接调用真实 Codex。
- 验证 `initialize` 和 `initialized` 的严格顺序。
- 验证返回值被转换为 Rust 类型。

### 第 7 章：实现真实 Codex 子进程

- Clap 命令行参数。
- `Command::new` 与 `Stdio::piped`。
- 获取 stdin、stdout 和 stderr 句柄。
- 创建 stderr 异步读取任务。
- 调用通用 `handshake`。

### 第 8 章：安全退出

- 为什么关闭 stdin。
- 为什么必须设置超时。
- `kill` 和 `wait` 的区别。
- `kill_on_drop` 解决的异常路径。

### 第 9 章：质量检查与真实验证

- `cargo fmt --all --check`。
- `cargo clippy --workspace --all-targets -- -D warnings`。
- `cargo test --workspace`。
- 运行已安装的 Codex。
- 比较运行前后的进程集合，确认没有遗留子进程。

### 第 10 章：从代码回看完整流程

- 按执行顺序重新串联所有文件。
- 说明每个模块的职责。
- 列出 Phase 0A 已具备和仍未具备的能力。

## 9. 错误处理

教程覆盖最常见的学习阻塞：

- 当前目录错误。
- Rust 工具链未安装或版本不一致。
- Cargo 找不到 Workspace 成员。
- Crate 名称与 `use` 路径不一致。
- 测试在预期失败阶段没有失败。
- JSON 字段命名不匹配。
- Codex 路径错误。
- Codex 未安装或未完成认证。
- app-server 提前退出。
- 子进程退出超时。
- Clippy 因 `unwrap`、`expect` 或未使用内容失败。

错误说明先帮助学习者识别“这是教程预期的失败”还是“输入错误导致的意外失败”，再给出最短排查路径。

## 10. 验证策略

教程以 `/Users/xulei/.dev/bridgehub` 中已经通过验证的 Phase 0A 代码作为技术来源。

编写教程时需要确认：

- 所有路径在从零创建的目录结构中一致。
- 所有代码块完整，没有省略内容。
- 带中文注释的代码仍是合法 Rust、TOML 或 Shell。
- 红色阶段的失败原因与教程描述一致。
- 绿色阶段的测试数量与预期一致。
- 最终真实命令与 `bridgehub-codex-spike` CLI 参数一致。
- 文档内部链接和章节引用有效。

Codex 不在 `/Users/xulei/.dev/bridge-hub` 中执行项目命令。学习者根据教程亲自执行，并把实际输出或错误发回当前对话，由 Codex继续解释和指导。

## 11. 完成标准

教程完成必须同时满足：

- 学习者不需要自行补代码或猜命令。
- 每个新概念都在第一次使用前解释。
- 每一段代码都说明其输入、输出和运行位置。
- 每个文件在创建前都提供相对路径、绝对路径、目录层级和 Neovim 命令。
- 所有关键代码行都能在代码块内部找到中文解释。
- 每个步骤都有明确的成功或预期失败信号。
- 教程完整覆盖当前 Phase 0A 实现。
- 教程没有提前混入 Phase 0B 或完整 BridgeHub 架构。
- 所有项目命令与代码都由学习者亲自输入和执行。
