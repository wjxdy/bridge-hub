# BridgeHub Phase 0A：从零手敲 Codex 握手程序

> 这是你应该照着操作的正式教程。`docs/superpowers/specs` 是设计说明，`docs/superpowers/plans` 是 Codex 的内部执行计划，都不是手敲教程。

## 使用规则

1. 所有命令都由你在终端亲自输入。
2. 所有项目代码都由你在 Neovim 亲自输入，不要复制粘贴。
3. 一次只做一个带编号的步骤。
4. 命令结果与“预期结果”不一致时立即停止，把完整命令和完整输出发给我。
5. 标记为“预期失败”的测试必须先看到失败，才能继续输入实现。

## 这本教程怎样替你完成工程设计

你不需要先学会架构设计，才能继续写这个项目。这个教程中的技术选择由 Codex 负责完成；你负责理解设计依据、亲手输入代码、观察结果，并逐渐建立自己的工程判断。

以后每个重要实现阶段都按下面的顺序展开：

```text
当前状态
→ 要解决的问题
→ 约束和不能破坏的规则
→ 真实可行的候选方案
→ 本教程选定的方案及理由
→ 代价和暂不实现的内容
→ 准备用什么行为验证
→ 完整代码
→ 运行验证
→ 回看设计最终落在哪段代码里
```

这不是让你做开放式架构题。候选方案用于展示工程师怎样比较方案，最终结论会直接告诉你。只有产品目标、费用或安全授权无法由技术事实替你决定时，才需要你作选择。

本教程所说的“最佳实践”同时满足四件事：符合当前官方接口、能用于真实项目、只引入当前阶段需要的复杂度、能够被测试和观察。它不等于“代码越少越好”，也不等于“一开始就把最终系统的全部抽象写完”。

## 阅读总体设计前的最小词汇表

下面这些词会反复出现。先建立一个够用的直觉，后面遇到代码时再逐层加深：

| 词汇 | 现在先这样理解 |
|---|---|
| Agent Loop | Codex 内部反复执行“读取上下文 → 调模型 → 执行工具 → 把结果交回模型”的主循环；BridgeHub 不重写它 |
| app-server | Codex 提供给其他程序调用的机器接口模式，不是给人直接聊天的终端界面 |
| stdio | standard input/output 的合称，表示程序的标准输入和标准输出；本教程也单独使用标准错误 |
| stdin | standard input，某个进程接收字节的入口；这里由 Rust 写、Codex 读 |
| stdout | standard output，程序输出正常结果的出口；这里由 Codex 写协议消息、Rust 读 |
| stderr | standard error，程序输出诊断信息的独立出口；它与 stdout 分开，避免日志混入协议 |
| JSONL | JSON Lines，一行是一条完整 JSON 消息，换行符就是消息边界 |
| JSON-RPC | 用 JSON 表达请求、响应、通知和错误的一套远程调用约定；Codex app-server 使用其语义，但 stdio 消息省略标准 `jsonrpc` 字段 |
| pending request 表 | 用请求 ID 记录“哪些请求仍在等待响应”的映射；Phase 0A 只有请求 0，因此暂时不需要 |
| PTY | pseudo-terminal，模拟真实终端的接口；适合交互式界面，不适合本阶段的稳定机器协议 |
| Thread / Turn | Codex 中的会话和一次模型执行；它们必须在初始化成功以后才能使用 |
| 进程树 | 一个进程及其继续启动的后代；当前包含 Node 启动器和原生 Codex，不一定只有父子两层 |
| Spike | 为验证一个高风险技术问题而写的小型端到端程序；它应真实可运行，但不追求完整产品功能 |
| Smoke Test | 在真实环境走通最基础主路径的快速检查；能发现集成失败，但不能替代精确的单元测试 |

这些名字看起来像网络概念，但 Phase 0A 的数据只在本机两个进程之间流动，不开放网络端口。

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
→ Rust 安全关闭并回收 Codex 进程树
→ Rust 确认清理成功后打印结果
```

当前不实现聊天、Thread、Turn、Tool Calling、Session、Relay、Connector 或 Web。

## Phase 0A 的总体工程设计

> [!question] 要解决的问题
> BridgeHub 最终要通过网络操作本机编码 Agent，但第一步还不能同时处理公网认证、WebSocket、数据库、页面和 Agent 协议。我们需要先证明一个最靠近 Codex、失败原因最容易观察的本机闭环：Rust 能否正确启动 Codex app-server、完成初始化并安全退出。

### 为什么不自己重新实现 Codex

当前有四条可能路线：

| 方案 | 优点 | 主要问题 | 本阶段结论 |
|---|---|---|---|
| 自己调用模型 API 并实现 Agent Loop | 所有行为都由自己控制 | 需要重写上下文、工具调用、审批、沙箱和会话管理，偏离 BridgeHub 的学习目标 | 不选 |
| 解析交互式 `codex` 终端输出 | 看起来容易启动 | 终端文字是给人看的展示层，不是稳定机器协议；颜色、进度动画和版本变化都可能破坏解析 | 不选 |
| 连接 app-server WebSocket | 天然是双向连接 | 当前 WebSocket 传输仍是实验接口，而且本机第一步没有网络传输需求 | 暂不选 |
| 连接 app-server stdio JSONL | 是 Codex 提供的机器接口；本机通信简单；每条消息可以直接观察 | 需要正确处理子进程、三条标准流、消息边界和退出回收 | **选择** |

> [!success] 本教程选定的方案
> Rust 作为父进程启动 `codex app-server --listen stdio://`，使用子进程 stdin 发送请求、stdout 接收协议消息、stderr 接收诊断日志。Codex 继续拥有 Agent Loop、模型调用、工具、沙箱和会话；BridgeHub 只负责连接、控制和后续的远程网关能力。

> [!note] 当前协议依据
> 这项选择已按本机 `codex-cli 0.144.1` 和 Codex 源码提交 `315195492c` 重新核对：`codex-rs/app-server/README.md` 说明 stdio 使用 JSONL、WebSocket 仍是实验接口，并规定 `initialize → initialized` 顺序；`codex-rs/app-server-protocol/src/protocol/v1.rs` 定义初始化参数、capability 和响应字段；`codex-rs/app-server-transport/src/transport/stdio.rs` 展示 stdin 逐行读取和 stdout 追加换行。升级 Codex 后应重新核对这些依据，而不是默认协议永远不变。

Codex 当前协议要求每条连接先发送一次 `initialize` 请求，再发送 `initialized` 通知；握手前的其他请求会被拒绝。因此初始化不是随便挑选的演示功能，而是所有 Thread、Turn 和 Tool Calling 的共同前置条件。

### 为什么 Phase 0A 只做到握手

如果第一步直接加入 Thread、流式事件和 Tool Calling，那么失败时可能来自进程启动、JSONL 分帧、请求关联、协议顺序或业务逻辑，初学者很难判断是哪一层出错。Phase 0A 故意只保留一个请求和一个通知，使我们先验证最底层的六个事实：

1. Rust 能启动正确的 Codex 程序。
2. Rust 能取得并使用三条标准流。
3. 双方能按一行一条 JSON 消息通信。
4. 请求 ID 和响应 ID 能正确对应。
5. 初始化顺序符合 Codex 协议。
6. Rust 最终能回收子进程，不留下后台进程。

最小架构如下：

```text
用户
  │ 启动
  ▼
bridgehub-codex-spike（Rust 父进程）
  ├─ 写 Codex stdin  ─────── initialize / initialized
  ├─ 读 Codex stdout ◀────── initialize response
  ├─ 读 Codex stderr ◀────── 诊断日志
  └─ 管理 Codex 进程树 ───── 等待、超时、整组终止、回收
                    │
                    ▼
       Codex 启动器 + 原生 app-server 进程树
```

> [!warning] 代价与暂不实现
> 这个 Spike 只支持单连接、固定请求 ID `0` 和一次初始化。它还不是通用 JSON-RPC 客户端。等 Phase 0B 真正出现多个并发请求和服务端主动请求时，再引入 ID 分配、pending request 表和持续读取循环；现在提前实现只会增加无法验证的抽象。

> [!check] Phase 0A 的完成证据
> 内存伪 Codex 测试证明发出的消息形状和顺序；真实 Codex Smoke Test 证明当前安装版本能够完成握手；进程前后集合一致证明没有留下新的 app-server 进程。这三种证据缺一不可，也不能互相替代。

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

> [!question] 要解决的问题
> 你会一边学习一边反复修改代码。如果目录和分支不明确，教程文件、自己手敲的代码和可随时回退的标准版本就会混在一起。

> [!success] 本教程选定的方案
> `/Users/xulei/.dev/bridge-hub` 是你亲手实现的学习项目；Git 记录每个可工作的阶段。当前学习工作放在 `learn-develop`，`main` 保留已经验证的标准答案，两条分支不做整体合并。教程只指导命令，不替你切分支或提交。

> [!warning] 代价与暂不实现
> 学习分支会比直接在 `main` 上多维护一条历史，但它允许你放心试错。Phase 0A 不设计复杂的 Git Flow，也不增加 release、hotfix 等暂时用不到的分支。

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

# status 查看当前仓库状态；-s 使用简短格式；-b 同时显示分支。
git status -sb
```

预期：

```text
learn-develop
```

如果你看到的是 `main`，先不要自行切换，把输出发给我确认。教程不应该在不知道你当前改动状态的情况下让你切分支。

这里先建立四个 Git 概念：

| 概念 | 现在先这样理解 |
|---|---|
| 仓库 | 项目根目录下的隐藏目录 `.git`，保存提交历史、分支指针和暂存信息 |
| 工作区 | 你正在看和编辑的普通项目文件；修改后尚未提交的内容就在这里 |
| 暂存区 | 下一次提交准备包含哪些改动的清单；`git add` 会更新它，但不会提交 |
| 分支 | 指向某条提交历史末端的可移动名字；当前学习过程使用 `learn-develop` |
| 提交 | 暂存区内容的一份带说明文字的历史快照；提交后仍可继续产生新修改 |

`git status -sb` 的第一行应以 `## learn-develop` 开头。后面的 `M`、`A` 或 `??` 分别表示已修改、已暂存新增或尚未跟踪；它们不是报错，也不要在不理解内容时用命令清掉。

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

`target/` 是 Cargo 的编译产物目录。开头的 `/` 不是电脑文件系统根目录，而是表示“只匹配当前 Git 仓库根目录下的 `target`”；因此以后某个示例目录中同名的普通文件夹不会被这条规则误伤。

保存退出：

```vim
:wq
```

## 1.3 本教程怎样安排提交

本教程不会替你执行 `git add` 或 `git commit`，也不会让你在代码尚未通过检查时把整个目录盲目暂存。一个学习阶段完成后，先运行该章要求的格式检查、Clippy 和测试，再用 `git status -sb` 查看实际改动；只暂存你能解释且属于当前阶段的文件，然后创建一个含义明确的小提交。

例如 Phase 0A 全部验证完成后，提交动作的形状是：

```text
# 先再次确认分支和改动清单。
git status -sb

# 把 <本阶段文件...> 替换成你刚确认过的准确路径；这里故意不提供通配符。
git add <本阶段文件...>

# 再检查暂存区中将要提交的文件。
git status

# 用一句话说明这份历史快照完成了什么。
git commit -m "feat: complete Codex app-server handshake spike"
```

尖括号这一行是提交方法的说明，不是现在可以原样执行的命令。等你完成最终验证并把 `git status -sb` 发给我后，我会根据当时真实改动给出准确文件列表；这样不会把密钥、临时文件或尚未完成的代码一起提交。

- [ ] 我确认当前分支是 `learn-develop`，并能区分工作区、暂存区和提交。
- [ ] 我知道当前不能原样执行带 `<本阶段文件...>` 的示例命令。

---

# 第 2 章：建立 Cargo Workspace

Cargo 是 Rust 的构建和依赖管理工具。它管理的几个名字容易混淆：

| 名称 | 含义 | 本项目里的例子 |
|---|---|---|
| Package | 一份由 `Cargo.toml` 描述、可以被 Cargo 构建和发布的工程单元 | `bridgehub-codex-spike` |
| Crate | Rust 编译器一次编译的代码单元；一个 Package 可以含 Library Crate 和 Binary Crate | `src/lib.rs` 与 `src/main.rs` |
| Workspace | 由一个根 `Cargo.toml` 统一管理的多个 Package | BridgeHub 根目录 |
| Cargo | 读取这些配置、解析依赖并执行 build/test/clippy 等命令的工具 | 终端里的 `cargo ...` |

日常交流中大家有时会把 Package 也简称为 Crate。本教程在谈 `Cargo.toml` 成员时尽量使用 Package，在谈某次编译产生的库或程序时使用 Crate。

## 2.0 先理解这套工程结构为什么这样设计

> [!question] 要解决的问题
> Phase 0A 目前只有一个小程序，但完整 BridgeHub 将逐步出现协议库、Codex Backend、Local Connector 和 Public Relay。如果现在把所有内容都堆进根目录的一个 Package，后面拆分时会同时改变目录、依赖和模块边界，学习成本反而更高。

真实可行的起步方式有三种：

| 方案 | 优点 | 代价 | 结论 |
|---|---|---|---|
| 单个根 Package | 文件最少，适合一次性小工具 | 后续多个程序和库会挤在一起，依赖边界不清楚 | 不选 |
| 一开始创建完整 V1 的所有 Crate | 最终目录一步到位 | 大量空模块没有当前用途，会让你学习不存在的业务 | 不选 |
| 虚拟 Workspace + 一个独立 Spike Crate | 现在只有一个真实成员，同时保留以后增加成员的根边界 | 比单 Package 多一个 `Cargo.toml` | **选择** |

> [!success] 本教程选定的方案
> 根 `Cargo.toml` 只定义 Workspace，不定义根 Package；真正可编译的程序放在 `tools/codex-spike`。这是“现在只建需要的一个成员，但从第一天就明确多 Crate 项目边界”的折中。

这套配置还作出四个配套决定：

1. **固定 Rust 1.96.0**：所有人在这个项目中使用同一编译器、Edition 行为、rustfmt 和 Clippy。代价是升级需要显式修改版本并重新验证，而不是自动跟随最新 stable。
2. **在 Workspace 集中声明依赖**：成员 Crate 通过 `.workspace = true` 继承版本，避免将来不同成员悄悄使用不同版本。每个成员仍然必须显式列出自己真正使用的依赖，依赖方向不会被隐藏。
3. **只启用 Tokio 所需 Feature**：当前只启用异步 I/O、宏、进程、多线程运行时和时间；不使用 `features = ["full"]`，这样 `Cargo.toml` 会明确告诉我们项目依赖 Tokio 的哪些能力。
4. **从根目录统一 Lint**：`unsafe`、`unwrap` 和 `expect` 的规则对所有成员一致。它会让错误处理代码稍长，但能迫使我们把失败路径设计清楚。

当前依赖各自只承担一个明确职责：

| 依赖 | 当前职责 | 为什么现在需要 |
|---|---|---|
| `clap` | 解析 `--codex-bin` | 不把本机安装路径硬编码进程序 |
| `process-wrap` | 把 Codex 启动器及其后代放入可统一终止的进程组或 Job Object | 当前 `codex` 是包装器，还会启动真正的原生 Codex，不能只管理直接 Child |
| `serde`、`serde_json` | Rust 类型与 JSONL 消息互转 | 协议边界需要准确字段名和可测试的数据结构 |
| `thiserror` | 定义握手错误枚举 | 调用方能区分 EOF、错误 ID、服务端拒绝和解析失败 |
| `tokio` | 异步子进程、标准流、Task 和超时 | 同一程序需要同时等待协议、stderr 和进程退出 |
| `tracing`、`tracing-subscriber` | 结构化诊断日志 | stderr 日志与 stdout 协议/结果分离，并可按环境变量开关 |

> [!warning] 代价与暂不实现
> Workspace 不是“越多 Crate 越专业”。Phase 0A 只创建 `codex-spike`；数据库、Web、Relay 和通用 Agent Trait 都等到出现真实调用方时再加入。

> [!check] 本章怎样验证设计成立
> Cargo 能从根目录识别唯一成员；成员能继承版本、依赖和 Lint；`cargo test -p bridgehub-codex-spike` 可以只选择这个 Package。此时还不要求连接 Codex。

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

# process-wrap 为 Tokio Child 增加 Unix 进程组和 Windows Job Object 管理。
# tokio1 启用它的 Tokio API。
process-wrap = { version = "9.1", features = ["tokio1"] }

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
process-wrap.workspace = true
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

在开始写 Rust 前，用只读命令检查实际目录层级：

```bash
# find 从当前目录查找；最大向下四层；只输出普通文件；sort 让顺序稳定。
find . -maxdepth 4 -type f | sort
```

项目已经包含教程和 Git 文件，因此输出会比下面更多；至少应能找到 `.gitignore`、根 `Cargo.toml`、`rust-toolchain.toml` 和 `tools/codex-spike/Cargo.toml`。如果出现 `tool/codex-spike` 而不是 `tools/codex-spike`，先修正目录或 Workspace 清单，不能继续靠修改 Cargo 命令绕过。

- [ ] `rust-toolchain.toml` 位于项目根目录。
- [ ] 根 `Cargo.toml` 包含 `[workspace]`。
- [ ] 成员路径是 `tools/codex-spike`。
- [ ] Crate `Cargo.toml` 已创建。

---

# 第 3 章：第一个测试和第一个实现

## 3.0 为什么不直接写完整握手

> [!question] 要解决的问题
> 如果第一次 TDD 就同时写异步进程、JSONL、Serde 和错误处理，那么测试失败时你无法判断是测试环境没搭好，还是协议实现有错。我们需要先用一个与后续协议真实相关、但足够小的行为确认 Library 和测试链路能够工作。

候选切入点：

| 方案 | 判断 |
|---|---|
| 写一个与项目无关的加法测试 | 最简单，但它不能保护任何 BridgeHub 行为 |
| 直接测试完整握手 | 业务价值高，但第一次会同时暴露太多新概念 |
| 先测试稳定的客户端身份 | 与后续 `initialize.clientInfo.name` 直接相关，同时只需要一个函数和一个断言 |

> [!success] 本教程选定的方案
> 先用 `client_identity_is_stable` 描述“协议客户端名称必须是 `bridgehub`”，观察函数不存在时的红色阶段，再添加最小实现得到绿色阶段。这一步的主要目标是学会完整的红→绿反馈循环，不是假装它已经验证了 Codex 协议。

这里也存在一个小的表示选择：固定名称可以写成 `const CLIENT_NAME: &str`，也可以通过 `client_name() -> &'static str` 暴露。本教程保留函数形式，因为后面的请求构造会通过统一调用点取得身份，也便于第一次练习函数、返回值和测试模块边界。`const fn` 表示这个函数满足 Rust 的常量求值规则，因此允许在常量上下文调用；它仍然可以在运行时调用，也不等于“函数没有运行时状态”。如果未来身份来自配置，这个签名就需要重新设计。

> [!warning] 代价与暂不实现
> 这个测试只能证明名称稳定以及测试工具链正常，不能证明 JSON 字段、握手顺序或真实 Codex 兼容性。那些行为将在第 5 章由更有价值的协议测试覆盖。

> [!check] 写代码前先确定可观察行为
> 第一次运行必须因找不到 `client_name` 而失败；加入最小实现后，同一个测试必须通过。若第一次直接通过，说明测试没有真正覆盖待实现能力或文件中已经存在旧实现。

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

## 4.0 先决定通信边界

> [!question] 要解决的问题
> Rust 和 Codex 是两个独立进程，它们不能直接读取彼此的变量。我们需要选择一种稳定的机器接口，还要知道每条通道只允许承载什么内容。

> [!abstract] 约束和不变量
> Codex app-server 的 stdio 传输使用一行一条 JSON 的 JSONL；每个连接只能初始化一次；客户端必须先完成 `initialize` 请求和 `initialized` 通知，才能发送其他请求。stdout 因此是协议通道，不能混入给人看的调试文本；stderr 专门承载诊断信息。

本机阶段比较三种传输：

| 方案 | 是否选择 | 原因 |
|---|---|---|
| 交互式终端/PTY 文本 | 否 | 展示文本没有稳定消息边界和类型契约 |
| app-server WebSocket | 否 | 当前仍是实验传输，本阶段还会额外引入端口、连接和网络错误 |
| app-server stdio JSONL | 是 | 官方机器协议、本机无需开放端口、可以用内存 I/O 测试相同读写逻辑 |

> [!success] 本教程选定的方案
> 把 Codex stdin 看成 Rust 发请求的单向入口，把 Codex stdout 看成 Rust 接收协议消息的单向出口，把 stderr 看成独立诊断出口。所谓“管道”只是操作系统传递字节的通道；JSON 结构和“一行结束一条消息”都是两端程序共同遵守的协议。

```text
Rust                              Codex app-server
  │                                     │
  │ initialize（请求，id = 0）          │
  ├──────────── stdin ─────────────────▶│
  │                                     │
  │ initialize result（响应，id = 0）   │
  │◀─────────── stdout ─────────────────┤
  │                                     │
  │ initialized（通知，无 id）          │
  ├──────────── stdin ─────────────────▶│
  │                                     │
  │ 此后连接才可用于其他方法             │
```

请求带 `id`，因为发送方需要把未来的响应对应回这次调用；通知没有 `id`，因为发送方不等待响应。Phase 0A 只有一个请求，所以固定使用 `0`。等多个请求可能同时在途时，才需要 ID 生成器和 pending request 表。

> [!warning] 代价与暂不实现
> 逐行 JSON 简单且容易调试，但读取方必须等待换行才能知道消息结束，写入方也必须在每条 JSON 后添加 `\n` 并及时刷新自身缓冲。Phase 0A 不处理多请求乱序响应、服务端主动请求或持续事件流。

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

## 4.1 先理解 `async`、Future、`.await` 和 Task

后面的测试第一次会出现异步代码。先把四个概念分开：

| 概念 | 含义 | 不是什么 |
|---|---|---|
| `async fn` | 调用后产生一个 Future 的函数 | 不会因为写了 `async` 就自动创建线程 |
| Future | 描述“一项异步操作以后如何继续”的值，可能尚未完成 | 不是 Task 里的“子任务清单” |
| `.await` | 等待当前 Future 有进展；等待 I/O 时允许当前 Tokio Task 暂时让出执行机会 | 不是忙循环，也不是新建线程 |
| Tokio Task | Tokio 调度的一条异步执行路线，可以在运行时线程之间被推进 | 不是操作系统进程，也不保证独占一条线程 |
| `tokio::spawn(future)` | 把一个 Future 注册成新的 Tokio Task，使它能与当前 Task 并发推进 | 不是 `std::thread::spawn` |

可以把它类比成 Web 服务等待数据库：请求处理逻辑在 `.await` 数据库结果时先让出执行机会，数据就绪后再继续。第 5 章让当前 Task 运行真实 `handshake`，同时让另一个 Task 扮演伪 Codex；第 6 章才会创建真正的操作系统子进程。

- [ ] 我能画出 Rust 写 Codex stdin、读 stdout 和读 stderr 的三个方向。
- [ ] 我知道 `tokio::spawn` 创建 Task，不会创建 Codex 进程，也不保证新建操作系统线程。

---

# 第 5 章：实现完整握手

## 5.0 写代码前完成握手设计

> [!question] 要解决的问题
> 第 4 章只确定了字节怎样流动。现在还要把“发送正确请求、识别对应响应、处理服务端错误、发送最终通知”变成一段可测试的 Rust API。如果全部使用任意 JSON，字段拼错只能在运行时发现；如果现在就实现通用 JSON-RPC 框架，又会引入 Phase 0A 没有调用方的复杂度。

> [!abstract] 约束和不变量
>
> 1. 每条消息必须是 UTF-8 JSON，并以一个换行结束。
> 2. `initialize` 必须先于 `initialized`。
> 3. `initialize` 请求的 ID 是 `0`，响应必须返回同一个 ID。
> 4. 服务端错误不能被伪装成“缺少结果”。
> 5. 收到成功结果后，必须先转换成我们需要的明确 Rust 类型，再把它交给调用方。
> 6. 相同握手逻辑既要能连接内存伪服务端，也要能连接真实子进程管道。

### 5.0.1 比较三种实现方式

| 方案 | 优点 | 问题 | 结论 |
|---|---|---|---|
| 所有消息都用 `serde_json::Value` | 写得快、字段自由 | 字段名和必填项缺少编译期约束，业务代码到处写字符串索引 | 不选作主体 |
| 立即实现完整通用 JSON-RPC Client | 后续可以支持并发请求 | 现在需要 ID 分配、路由表、读取循环、取消和服务端请求，无法由本阶段充分验证 | 推迟到 Phase 0B |
| 具体握手类型 + 小型响应信封 + 通用异步 I/O | 当前协议字段清楚，同时能复用在测试和真实管道 | 后续多请求时需要升级传输层 | **选择** |

> [!success] 本教程选定的方案
> `handshake<R, W>` 只实现一次初始化事务。参数和最终结果使用明确结构体；外层响应先解析成 `ResponseEnvelope`，用它检查 `id` 和 `error`，其中成功的 `result` 暂存为 `Value`，通过检查后再转换成 `InitializeResponse`。这叫“在不确定的协议边界保持灵活，进入业务代码前恢复强类型”。

### 5.0.2 为什么 Reader 和 Writer 使用泛型

如果函数直接接收 `ChildStdout` 和 `ChildStdin`，测试就必须真的启动 Codex。这里让它接收任何实现 `AsyncBufRead + Unpin` 的 Reader 和任何实现 `AsyncWrite + Unpin` 的 Writer：

```text
测试时：tokio::io::duplex → handshake
真实运行：ChildStdout / ChildStdin → 同一个 handshake
```

这不是为了“泛型越多越高级”，而是为了把协议逻辑与进程创建分离。测试可以在内存中精确控制每条消息，真实程序不需要另一份握手实现。

### 5.0.3 为什么需要这些类型

| 类型 | 设计职责 |
|---|---|
| `Request<T>` | 固定请求外壳，同时允许 `params` 使用具体类型 |
| `Notification` | 表达“不含 ID、无需响应”的消息 |
| `InitializeParams` | 聚合客户端身份和本连接声明的能力 |
| `ClientInfo` | 明确告诉 Codex 调用方名称、标题和版本 |
| `InitializeCapabilities` | 明确关闭当前阶段不支持的实验 API 和 attestation 请求 |
| `ResponseEnvelope` | 在读取业务结果前检查响应 ID、RPC 错误和结果存在性 |
| `InitializeResponse` | 只把后续代码真正使用的成功字段暴露为强类型 |
| `HandshakeError` | 把 EOF、错误 ID、服务端拒绝、I/O 和 JSON 错误区分开 |

`#[serde(rename_all = "camelCase")]` 负责把 Rust 的 `client_info` 等 snake_case 字段映射成协议要求的 `clientInfo`。这种映射集中写在类型定义上，比手写字符串键更不容易漂移。

Codex app-server 使用 JSON-RPC 2.0 的请求、响应和通知语义，但它的线上消息明确省略标准的 `"jsonrpc": "2.0"` 字段，因此 `Request<T>` 只包含 `method`、`id` 和 `params`。这里不是漏写字段，而是遵守 Codex app-server 的具体传输契约。

错误使用枚举而不是一个自由文本 `String`，因为调用方将来可能对“Codex 已退出”和“Codex 拒绝初始化”采取不同恢复方式。`thiserror` 只生成标准 `Error`、`Display` 和 `From` 样板代码，不改变错误边界本身。

### 5.0.4 握手状态机

```text
Start
  │ 写 initialize(id=0)
  ▼
WaitingResponse
  ├─ EOF ─────────────────────────────▶ UnexpectedEof
  ├─ 非法 JSON ───────────────────────▶ Json
  ├─ id != 0 ─────────────────────────▶ UnexpectedResponseId
  ├─ error 存在 ──────────────────────▶ Server
  ├─ result 不存在 ───────────────────▶ MissingResult
  │ result 可转换为 InitializeResponse
  ▼
ResponseValidated
  │ 写 initialized
  ▼
Ready
```

> [!warning] 代价与暂不实现
> 当前函数一次只等待一条响应，并假设初始化期间没有需要路由的其他消息。这个限制与单请求 Phase 0A 一致。它不能直接扩展成多轮对话；Phase 0B 必须引入持续读取循环和请求关联，而不是继续把所有逻辑塞进 `handshake`。

> [!check] 写实现前先定义测试证据
> 内存伪 Codex 必须观察到 `initialize` 的方法名、ID、客户端名和 capability；随后返回成功响应；最后必须观察到完全等于 `{ "method": "initialized" }` 的通知。客户端一侧还要证明结果已经转换为 `InitializeResponse`。只检查“函数返回 Ok”不够，因为它无法证明消息形状和顺序。

这一章先写会失败的握手测试，再输入完整实现。`tokio::spawn` 在测试中启动的是一个可并发推进的 Tokio Task，用来扮演伪 Codex；它不是专门创建一个操作系统线程。

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

预期失败：找不到 `InitializeResponse` 和 `handshake`。这两项都会在下一步实现。

## 5.2 输入完整实现

测试已经描述了外部行为，接下来按“边界类型 → 错误类型 → 状态转换 → 通用写入”实现，而不是按想到什么写什么：

```text
协议字段类型
  ↓ 约束 JSON 形状
HandshakeError
  ↓ 约束失败出口
handshake
  ↓ 约束消息顺序和状态转换
write_json_line
  ↓ 集中保证 JSON + 换行 + flush
```

这里把 `write_json_line` 单独提取出来，是因为 `initialize` 请求和 `initialized` 通知都必须遵守完全相同的 JSONL 写入规则。若两处各自序列化和追加换行，未来很容易只修正其中一处。它保持为私有函数，因为目前只有这个模块需要该机制。

实现完成后，可以这样把设计映射回代码：

| 设计决定 | 代码落点 |
|---|---|
| 协议字段强类型 | `InitializeParams`、`ClientInfo`、`InitializeResponse` |
| 在边界暂存动态 JSON | `ResponseEnvelope.result: Option<Value>` |
| 错误可分类 | `HandshakeError` 的各个变体 |
| 测试与真实管道复用 | `handshake<R, W>` 的 Trait Bound |
| 每条消息统一成 JSONL | `write_json_line` |
| 固定协议顺序 | `handshake` 内先 request、后 response、再 notification |

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
    // #[from] 生成 From<std::io::Error>，所以 ? 可以自动转换错误。
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
    // Reader 必须支持异步缓冲读取；Unpin 满足这些扩展方法对固定位置的要求。
    R: AsyncBufRead + Unpin,
    // Writer 必须支持异步写入；这里同样需要 Unpin。
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
    // 刷新 Writer 自己尚未提交的用户态缓冲；这不表示 Codex 已经读取。
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

## 5.3 再验证三条失败契约

> [!question] 为什么成功测试通过后还要写失败测试
> `HandshakeError` 有多个变体，是为了让上层根据失败原因采取不同动作。若测试只覆盖成功路径，我们只能证明“正常响应能工作”，不能证明 EOF、错误请求 ID 和服务端拒绝真的会被分成三类。错误类型写得再漂亮，没有行为测试也只是未经验证的设计。

这里选择三个最靠近协议状态机分叉点的案例：

| 测试 | 模拟的真实情况 | 要保护的恢复信息 |
|---|---|---|
| 响应前 EOF | Codex 启动后立即关闭 stdout | 不能误报成 JSON 错误 |
| 响应 ID 为 7 | 收到的响应不属于请求 0 | 不能把其他请求的结果当成初始化结果 |
| 服务端返回 error | Codex 明确拒绝初始化 | 必须保留错误码和消息 |

`MissingResult`、I/O 错误和非法 JSON 已由同一个枚举表达，但本阶段不为每个 Serde/I/O 分支堆测试。当前三条失败测试加一条完整成功测试，覆盖握手状态机最重要的对外行为；以后某个错误开始驱动重试或用户提示时，再为那条恢复策略增加测试。

重新打开：

```bash
nvim tools/codex-spike/src/lib.rs
```

先把测试模块原来的 `use super::{InitializeResponse, handshake};` 替换成下面这行，让新增测试也能匹配错误变体：

```rust
// 从上一层导入错误类型、成功响应类型和握手函数。
use super::{HandshakeError, InitializeResponse, handshake};
```

然后把下面三个测试放到 `tests` 模块内部、最后一个 `}` 之前：

```rust
// 注册由 Tokio 运行的异步 EOF 测试。
#[tokio::test]
// 没有响应时必须返回明确的 UnexpectedEof。
async fn handshake_reports_eof_before_response() {
    // empty 创建一个立即返回 EOF 的异步 Reader。
    let mut reader = BufReader::new(tokio::io::empty());
    // sink 接收并丢弃 initialize 请求字节，避免测试依赖真实进程。
    let mut writer = tokio::io::sink();

    // 执行握手并保留 Result，不用 unwrap 或 expect。
    let result = handshake(&mut reader, &mut writer).await;

    // matches! 检查错误变体，而不是比较容易变化的显示文本。
    assert!(matches!(result, Err(HandshakeError::UnexpectedEof)));
}

// 注册错误响应 ID 测试。
#[tokio::test]
// 请求 0 不能接受属于请求 7 的响应。
async fn handshake_rejects_response_for_another_request() {
    // 字节切片模拟 Codex 返回的一条完整 JSONL 消息。
    let response = b"{\"id\":7,\"result\":{}}\n";
    // BufReader 让内存字节切片满足握手 Reader 的要求。
    let mut reader = BufReader::new(&response[..]);
    // 当前测试不关心客户端写出的具体内容。
    let mut writer = tokio::io::sink();

    // 执行握手。
    let result = handshake(&mut reader, &mut writer).await;

    // 同时检查变体和实际收到的 ID。
    assert!(matches!(
        result,
        Err(HandshakeError::UnexpectedResponseId { actual: 7 })
    ));
}

// 注册服务端拒绝测试。
#[tokio::test]
// RPC error 中的错误码和消息必须保留下来。
async fn handshake_preserves_server_error() {
    // 模拟 Codex 返回 error 而不是 result。
    let response =
        b"{\"id\":0,\"error\":{\"code\":-32600,\"message\":\"initialize rejected\"}}\n";
    // 把固定响应包装成异步缓冲 Reader。
    let mut reader = BufReader::new(&response[..]);
    // 丢弃客户端写出的 initialize 请求。
    let mut writer = tokio::io::sink();

    // 执行握手。
    let result = handshake(&mut reader, &mut writer).await;

    // 模式守卫额外检查 String 中的真实消息内容。
    assert!(matches!(
        result,
        Err(HandshakeError::Server { code: -32600, message })
            if message == "initialize rejected"
    ));
}
```

保存后运行：

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

预期：5 个测试通过。此时你已经分别证明了一条成功状态转换和三条关键失败状态转换，而不是只证明代码能够编译。

- [ ] 我理解为什么按错误变体断言比按错误字符串断言稳定。
- [ ] 我看到 5 个测试通过。

---

# 第 6 章：启动真实 Codex 进程树

## 6.0 先设计进程的所有权和生命周期

> [!question] 要解决的问题
> 第 5 章只在内存中证明协议逻辑。真实运行时，Rust 还必须找到 Codex、启动它的完整进程树、取得三条标准流、给握手设置期限，并保证普通成功和错误路径最终都进入统一清理。这里的核心不是一条 `spawn()`，而是谁从启动到退出始终拥有并管理这些资源。

### 6.0.1 为什么选择 Tokio 进程，而不是标准库线程

| 方案 | 优点 | 问题 | 结论 |
|---|---|---|---|
| `std::process` + 阻塞读取 + 手动线程 | 不需要异步运行时 | stdout、stderr、超时和退出需要自己协调多个线程，后续事件流还要再次重构 | 不选 |
| `tokio::process` + 异步 I/O | 能与握手 Future、stderr Task 和 timeout 使用同一运行时 | 必须理解 Task、`.await` 和资源所有权 | **选择** |
| 通过 Shell 拼接命令字符串 | 写起来短 | 参数转义、错误边界和可移植性更差，还多出一个无意义的 Shell 进程 | 不选 |

Tokio 的 `Command::spawn()` 启动的是一个真实操作系统子进程；`tokio::spawn(...)` 启动的是 Tokio 调度的异步 Task。两者名字相似，但不是同一层概念。

> [!success] 本教程选定的方案
> `run` 独占包装后的 `ChildWrapper`，直到完成 `wait`。握手函数只借用 stdin/stdout，不拥有整个进程；stderr 移交给一个独立 Tokio Task 持续读取。这样进程生命周期只有一个负责人，协议逻辑仍可独立测试。

### 6.0.2 为什么不能只管理一个 `Child`

本机的 `/opt/homebrew/bin/codex` 不是最终执行 Agent 的单个二进制。它是 Node.js 启动脚本，随后还会启动原生 Codex：

```text
BridgeHub Rust 进程
└── Node.js Codex 启动器（直接 Child）
    └── 原生 Codex app-server（孙进程）
```

如果只对直接 Child 调用 Tokio `kill()`，强制终止 Node 启动器时，原生 Codex 可能还活着。把私有的原生二进制路径硬编码进 BridgeHub 也不正确，因为那个路径属于 Codex 的安装细节，升级后可能变化。

本教程先检查了现成方案：旧的 `command-group` 已被维护者标记弃用，而且它在 Unix 上的 `kill_on_drop` 不能提供这里需要的整组兜底；它的维护者指定 `process-wrap` 为后继项目。最终选择是：

| 平台 | 包装方式 | 强制终止的对象 |
|---|---|---|
| macOS / Linux / BSD | `ProcessGroup::leader()` | 同一 Unix 进程组中的启动器和后代 |
| Windows | `JobObject` | Job Object 中的整棵进程树 |
| 所有平台的额外兜底 | `KillOnDrop` | Tokio 能直接拥有的启动器；不能代替显式整组清理 |

> [!success] 本教程选定的方案
> 使用维护中的 `process-wrap 9.1` 保留 Tokio 原始 stdin/stdout/stderr，同时让 `start_kill()` 和 `wait()` 经过平台对应的进程树包装。正常错误路径始终显式清理整组；`KillOnDrop` 只是直接 Child 的最后一道兜底，绝不把它描述成 macOS 上完整的进程树保证。

这里没有选择功能更大的进程 Supervisor 库，因为 Phase 0A 只需要“单次启动、统一终止、等待回收”；重启、退避和长期监督还没有调用方。也没有自己写 `killpg`/Windows API，因为现成库已经封装了平台差异，手写只会把操作系统细节混入协议 Spike。

### 6.0.3 三条标准流怎样分配所有权

```text
ChildWrapper（进程组 / Job Object）
├─ stdin  ── run 暂时拥有 ─── 借给 handshake 写消息 ── 完成后 drop
├─ stdout ── run 取出 ─────── BufReader ────────────── 借给 handshake
├─ stderr ── 移入 stderr_task ─ 持续读取到 EOF
└─ 进程树句柄 ─ run 始终拥有 ── try_wait / start_kill / wait
```

调用 `.take()` 是把流句柄从 `ChildWrapper` 的 `Option` 中移出来，确保同一条流不会同时有两个所有者。`handshake` 接收 `&mut` 借用，因此完成后 `run` 仍然拥有 stdin，可以明确关闭它。

stderr 必须与 stdout 同时读取。如果子进程持续写 stderr 而父进程完全不读，操作系统管道缓冲区最终可能写满，Codex 会停在一次写操作上；此时即使 stdout 逻辑正确，整个程序也可能像“莫名卡死”一样。独立 Task 的职责只是排空并记录日志，不参与协议状态。

### 6.0.4 为什么握手和退出需要两个期限

`handshake` 可能卡在“等待 initialize 响应”，而握手成功后 Codex 也可能卡在“收到 stdin EOF 后仍不退出”。这是两个不同阶段，不能只给后者设置超时：

| 期限 | 本教程数值 | 保护的等待 |
|---|---:|---|
| `HANDSHAKE_TIMEOUT` | 10 秒 | 启动后迟迟收不到 initialize 响应 |
| `EXIT_GRACE_PERIOD` | 2 秒 | 关闭 stdin 后等待 Codex 自然退出 |
| `STDERR_DRAIN_TIMEOUT` | 1 秒 | 进程结束后 stderr Reader 仍未收尾 |

这些值是 Spike 的可观察边界，不是未来 Connector 的永久配置。长期连接会用取消令牌、心跳和 Process Actor 重新设计。

### 6.0.5 退出策略必须在启动前设计

```text
spawn 成功
  │
  ├─ 握手成功 / 握手错误 / 握手超时
  │    └─ 先保存结果，不用 ? 提前返回
  ▼
drop(stdin) + drop(reader)
  ▼
最多等待 2 秒自然退出（try_wait + sleep）
  ├─ 已退出 ─────────────────────────▶ 保存 ExitStatus
  └─ 仍运行 ─▶ 整组 start_kill ─────▶ wait 回收
  ▼
限时收拢 stderr Task
  ▼
优先返回原始握手结果；握手成功时再检查清理和退出状态
```

`start_kill()` 只请求强制终止包装器负责的整组进程，`wait().await` 才等待直接 Child 最终退出并取得 `ExitStatus`。正常路径先关闭 stdin，是给 app-server 看见 EOF 并自行退出的机会。

为什么宽限期不用 `timeout(grace, child.wait())`？`process-wrap` 的 Unix 进程组等待需要在后台回收组内进程；取消这个 wait Future 后再等待会形成难以推理的回收竞态。本教程改用 `try_wait()` 非阻塞检查和短暂 `sleep()`，直到期限到达；整个过程中从未启动一个随后被取消的 wait。

`handshake` 的结果先保存起来，清理完成后才使用 `?` 返回。这样 EOF、非法 JSON、服务端拒绝和握手超时都会经过同一清理路径。若握手和清理同时失败，原始握手错误是主错误，清理错误写入诊断日志；若握手成功，清理错误才成为最终错误。这条优先级能保留最接近根因的信息。

`KillOnDrop` 不是正常关闭方案。它无法替代进程组的显式 `start_kill + wait`，也无法处理宿主进程被 `SIGKILL`、断电或 `abort` 导致析构函数根本不运行的情况。Phase 0A 保证的是程序能够执行的普通 `Result` 成功/失败路径；未来公网 Connector 还需要外部服务管理器提供更高一级的存活与重启保证。

> [!warning] 代价与暂不实现
> 当前 `run` 同时承担 CLI 装配和一次性进程监督，适合 Spike。长期运行的 Connector 需要专门的 Process Actor、重启退避、取消令牌和持续消息路由；现在没有这些真实需求，暂不抽象 Supervisor。

> [!check] 写代码前先定义可观察行为
> `--help` 能看到可覆盖的 `--codex-bin`；握手最多等待 10 秒；成功、协议错误和超时都会先清理；程序返回后进程集合与运行前一致。仅仅“成功打印 handshake=ok”还不能证明生命周期正确。

## 6.1 先测试启动参数

`app-server --listen stdio://` 不是随手拼出的字符串，而是 BridgeHub 选择的 Codex 传输契约。它既被真实进程入口使用，也需要一个不启动 Codex就能运行的快速测试保护。

| 方案 | 问题或收益 | 结论 |
|---|---|---|
| 直接把三个字符串写在 `main.rs` 的 `.args(...)` 中 | 代码最短，但测试它就必须连带测试真实进程装配，协议入口也不容易单独定位 | 不选 |
| 从配置文件读取 | 可以修改，但当前产品并不允许用户任意改变 app-server 模式；把协议不变量做成配置反而会产生无效状态 | 不选 |
| Library 中提供固定数组函数并做同步测试 | `main` 只有一个明确调用点，测试无需启动外部进程，未来协议升级时也只有一个修改位置 | **选择** |

函数保持非常小，是因为它只拥有“用什么参数进入支持的传输模式”这一条事实，不负责创建进程。它现在是 `pub`，原因是同一 Package 的 Binary Crate 需要通过 Library Crate 的公共边界调用它；这不是为了提前设计一个通用命令构造框架。

这个测试只能防止参数被误删或改错，不能证明当前安装的 Codex 真正接受它们。因此第 9 章仍需要真实 Smoke Test。

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

`lib.rs` 负责“如何说 Codex 协议”，`main.rs` 负责“如何让真实 Codex 进程树活着并最终退出”。分开这两个文件后，测试不需要启动外部程序，CLI 也不需要知道每个 JSON 字段的细节。

Library 使用可匹配的 `HandshakeError`，因为它是会被其他模块调用的协议边界；这个一次性 Spike 的 `main` 使用 `Result<(), Box<dyn Error>>` 汇总 CLI 解析之后可能出现的 I/O、握手和进程错误，让入口保持薄。未来 Connector 需要根据错误决定重启或降级时，二进制入口也要升级成明确的应用错误，而不是把 `Box<dyn Error>` 扩散到核心模块。

诊断信息使用 `tracing`，并显式通过 `.with_writer(io::stderr)` 写入父进程 stderr；握手结果才使用 `println!` 写到 stdout。`tracing-subscriber` 的默认 Writer 是 stdout，因此这里必须主动覆盖。这样将来 stdout 被脚本读取时，调试日志不会破坏机器可读结果。stderr Task 始终在排空 Codex 日志；`RUST_LOG` 只决定这些 `debug!` 记录是否显示，不负责启动读取。

接下来这段代码按七个阶段组织：

1. 初始化日志并解析 CLI 参数。
2. 使用平台对应包装器配置并启动进程树。
3. 取出三条流；如果这个不变量意外失败，先整组终止再返回错误。
4. 立即启动 stderr 排空 Task。
5. 限时执行 `handshake`，只保存结果，不提前 `?` 返回。
6. 关闭管道，执行“轮询宽限期 → 超时整组 start_kill → 最终 wait”，再限时收拢 stderr Task。
7. 清理结束后按既定优先级返回错误或打印成功结果。

阅读大段代码时先寻找这七个阶段，不要把它看成一百多行彼此无关的 Rust 语法。`take_child_pipes`、`wait_or_kill` 和 `finish_stderr_task` 都只提取了一项有独立名字的生命周期职责，并不是为了“函数越多越专业”。

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
// 从标准库导入错误、I/O、路径、退出状态、标准流和时间长度类型。
use std::{
    error::Error,
    io,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

// 从我们自己的 Library 导入启动参数和握手函数。
use bridgehub_codex_spike::{app_server_args, handshake};
// 导入 Clap 的命令行解析 Trait。
use clap::Parser;
// Windows 使用 Job Object 管理进程树。
#[cfg(windows)]
use process_wrap::tokio::JobObject;
// Unix 系统使用进程组管理启动器及其后代。
#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;
// 导入 process-wrap 的跨平台 Child 接口、命令包装器和直接 Child 兜底。
use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
// 从 Tokio 导入异步逐行读取、Task 句柄以及计时工具。
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
// 导入 debug 日志宏。
use tracing::debug;
// 导入通过环境变量配置日志级别的 Filter。
use tracing_subscriber::EnvFilter;

// initialize 握手最多等待十秒。
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
// 关闭 stdin 后，给 Codex 两秒自然退出时间。
const EXIT_GRACE_PERIOD: Duration = Duration::from_secs(2);
// 宽限期内每 25 毫秒非阻塞检查一次退出状态。
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(25);
// 进程结束后，最多再等一秒让 stderr Task 收尾。
const STDERR_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

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
    // 日志初始化只属于程序入口。
    init_tracing();
    // 入口把解析后的参数交给真正的运行函数。
    run(Args::parse()).await
}

// run 拥有从进程树启动到清理结束的完整生命周期。
async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    // 创建包住 Tokio Command 的可组合命令。
    let mut command = CommandWrap::with_new(&args.codex_bin, |command| {
        // 添加 app-server --listen stdio:// 参数。
        command
            .args(app_server_args())
            // 把 Codex stdin 连接成 Rust 可写管道。
            .stdin(Stdio::piped())
            // 把 Codex stdout 连接成 Rust 可读管道。
            .stdout(Stdio::piped())
            // 把 Codex stderr 连接成 Rust 可读管道。
            .stderr(Stdio::piped());
    });

    // ChildWrapper 被意外丢弃时，至少终止 Tokio 直接拥有的启动器。
    command.wrap(KillOnDrop);
    // Unix 把 Codex 启动器设为新进程组的 leader。
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());
    // Windows 把 Codex 进程树放入 Job Object。
    #[cfg(windows)]
    command.wrap(JobObject);

    // 真正启动进程；此处失败时还没有需要清理的 Child。
    let mut child = command.spawn()?;

    // 三条管道在配置为 piped 后都应该存在，集中取出并检查不变量。
    let pipes = take_child_pipes(child.as_mut());
    let (mut stdin, stdout, stderr) = match pipes {
        // 不变量成立，继续运行。
        Ok(pipes) => pipes,
        // 若库或平台意外没有给出管道，先终止和等待，再返回原始错误。
        Err(error) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(error.into());
        }
    };

    // 创建 Tokio 异步任务持续读取 stderr；它不是 Codex 子线程。
    let mut stderr_task = tokio::spawn(async move {
        // 把 stderr 包装成可以逐行读取的 Reader。
        let mut lines = BufReader::new(stderr).lines();
        // 每次异步等待一行；None 表示 stderr 已关闭。
        while let Some(line) = lines.next_line().await? {
            // RUST_LOG 允许时显示；无论是否显示，上面的读取都会发生。
            debug!(target: "codex_app_server", message = %line);
        }
        // 明确异步任务的成功类型和错误类型。
        Ok::<(), io::Error>(())
    });

    // 用 BufReader 包装 stdout，满足 handshake 的 AsyncBufRead 要求。
    let mut reader = BufReader::new(stdout);
    // 给握手单独设置期限，并把成功或失败都保存起来。
    let handshake_result: Result<_, Box<dyn Error>> =
        match timeout(HANDSHAKE_TIMEOUT, handshake(&mut reader, &mut stdin)).await {
            // 握手在期限内结束，把 HandshakeError 统一装入 Box。
            Ok(result) => result.map_err(Into::into),
            // 期限到达，构造明确的 TimedOut 错误；此处仍不提前返回。
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Codex initialize handshake exceeded {HANDSHAKE_TIMEOUT:?}"),
            )
            .into()),
        };

    // 关闭 stdin，告诉 Codex 不会再收到消息。
    drop(stdin);
    // 关闭父进程持有的 stdout Reader，让生命周期边界更明确。
    drop(reader);

    // 无论握手结果如何，都先等待自然退出，必要时整组终止并回收。
    let shutdown_result = wait_or_kill(child.as_mut(), EXIT_GRACE_PERIOD).await;
    // 进程清理后，限时收拢负责 stderr 的 Task。
    finish_stderr_task(&mut stderr_task).await;

    // 若握手已经失败，保留它作为主错误；额外的清理错误只写诊断日志。
    if handshake_result.is_err()
        && let Err(error) = &shutdown_result
    {
        debug!(%error, "Codex cleanup also failed after handshake error");
    }

    // 到这里清理已经执行；现在才传播握手错误。
    let initialized = handshake_result?;
    // 握手成功时，清理错误才是最终错误。
    let status = shutdown_result?;
    // Codex 非成功退出时，不把整个运行报告成成功。
    if !status.success() {
        return Err(format!("Codex app-server exited with {status}").into());
    }

    // 所有验证和清理都成功后，再向 stdout 打印机器结果。
    println!("handshake=ok");
    println!("user_agent={}", initialized.user_agent);
    // display() 把 PathBuf 转成适合显示的路径。
    println!("codex_home={}", initialized.codex_home.display());
    println!("platform_family={}", initialized.platform_family);
    println!("platform_os={}", initialized.platform_os);
    Ok(())
}

// 从包装后的 Child 中一次性取出三条标准流。
fn take_child_pipes(
    child: &mut dyn ChildWrapper,
) -> io::Result<(
    tokio::process::ChildStdin,
    tokio::process::ChildStdout,
    tokio::process::ChildStderr,
)> {
    // 取得 Rust 用来写 Codex 的 stdin。
    let stdin = child
        .stdin()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdin was not piped"))?;
    // 取得 Rust 用来读取协议的 stdout。
    let stdout = child
        .stdout()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdout was not piped"))?;
    // 取得 Rust 用来排空诊断的 stderr。
    let stderr = child
        .stderr()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stderr was not piped"))?;
    // 三条流作为一个不可拆散的成功结果返回。
    Ok((stdin, stdout, stderr))
}

// 给进程树自然退出的机会；期限到达后整组终止并等待回收。
async fn wait_or_kill(
    child: &mut dyn ChildWrapper,
    grace_period: Duration,
) -> io::Result<ExitStatus> {
    // Instant 是单调时钟，适合计算相对期限。
    let deadline = Instant::now() + grace_period;

    loop {
        // try_wait 不阻塞当前 Task；Some 表示已经取得退出状态。
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }

        // 到达宽限期后进入强制终止路径。
        if Instant::now() >= deadline {
            // start_kill 经过 ProcessGroup 或 JobObject，目标是整组进程。
            if let Err(kill_error) = child.start_kill() {
                // try_wait 与 start_kill 之间进程可能刚好自然退出，再检查一次。
                if let Some(status) = child.try_wait()? {
                    return Ok(status);
                }
                // 进程仍在且终止失败，保留真正的 kill 错误。
                return Err(kill_error);
            }
            // 终止请求成功后必须 wait，取得退出状态并回收直接 Child。
            return child.wait().await;
        }

        // 暂停当前 Task，避免在循环中占满 CPU。
        sleep(EXIT_POLL_INTERVAL).await;
    }
}

// 限时等待 stderr Task；被后代长期持有的管道不能让主程序永久挂住。
async fn finish_stderr_task(stderr_task: &mut JoinHandle<io::Result<()>>) {
    match timeout(STDERR_DRAIN_TIMEOUT, &mut *stderr_task).await {
        // Task 与内部 I/O 都成功。
        Ok(Ok(Ok(()))) => {}
        // Task 正常返回，但读取 stderr 失败。
        Ok(Ok(Err(error))) => debug!(%error, "failed while draining Codex stderr"),
        // Task 自身 Panic 或被取消。
        Ok(Err(error)) => debug!(%error, "Codex stderr task failed"),
        // 一秒后仍未完成，主动取消这个只负责日志的 Task。
        Err(_) => {
            stderr_task.abort();
            // 等待取消完成；正常的 cancelled 不需要再打印错误。
            if let Err(error) = stderr_task.await
                && !error.is_cancelled()
            {
                debug!(%error, "Codex stderr task failed while being stopped");
            }
        }
    }
}

// 初始化 tracing 日志订阅器。
fn init_tracing() {
    // 优先读取 RUST_LOG；没有设置时使用安全默认值。
    let filter = EnvFilter::try_from_default_env()
        // 默认显示本程序 info，关闭 Codex app-server debug 噪音。
        .unwrap_or_else(|_| EnvFilter::new("bridgehub_codex_spike=info,codex_app_server=off"));
    // 创建文本日志 Subscriber Builder。
    tracing_subscriber::fmt()
        // 应用上面得到的日志过滤规则。
        .with_env_filter(filter)
        // 默认 Writer 是 stdout；这里显式改成 stderr，避免污染程序结果。
        .with_writer(io::stderr)
        // 注册为当前进程的全局 Subscriber。
        .init();
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

预期：6 个测试通过，帮助信息包含：

```text
--codex-bin <CODEX_BIN>
```

- [ ] 我理解为什么启动参数属于可测试的协议入口，而不是用户配置。
- [ ] 我看到 6 个测试通过，并在 `--help` 中看到 `--codex-bin`。

---

# 第 7 章：审计 Codex 进程树的安全退出

第 6 章已经在代码之前确定了退出策略。本章不是事后补充理由，而是检查设计是否真的落入了每一条路径：

## 7.1 先按“结果”和“资源”分开思考

握手返回一个业务结果，Codex 进程树和三条标准流则是系统资源。这两件事不能写成“握手一失败就 `?` 返回”，因为业务函数返回了，不代表操作系统会自动替我们回收所有后代。

当前代码让每一条普通 `Result` 路径都经过同一条清理主干：

| 触发情况 | 先保存的握手结果 | 随后的资源动作 | 最终主结果 |
|---|---|---|---|
| 正常响应 | `Ok(InitializeResponse)` | 关闭管道，给进程树宽限期，等待退出 | 进程成功退出后才打印成功 |
| EOF、错误 ID、RPC 错误、非法 JSON | 对应 `HandshakeError` | 同样关闭、等待，必要时整组终止 | 返回原始握手错误 |
| 10 秒没有响应 | `TimedOut` | 同样关闭、等待，必要时整组终止 | 返回握手超时 |
| stdin 关闭后 2 秒仍运行 | 原握手结果保持不变 | `start_kill()` 终止进程组或 Job Object，再 `wait()` | 握手成功时报告清理错误；握手失败时保留握手错误并记录清理错误 |

“先保存结果、完成清理、最后传播错误”是这一章最重要的工程模式。它和 Web 请求里的 `finally`、RAII 资源守卫或数据库事务收尾解决的是同一类问题：失败路径也必须执行资源生命周期规则。

## 7.2 为什么管理的是进程树，不只是 Child

本机的 `codex` 启动文件会形成“Node 启动器 → 原生 Codex”两层。`ChildWrapper` 对外仍提供 stdin、stdout、stderr、`try_wait`、`start_kill` 和 `wait`，但包装器在不同平台改变了终止范围：Unix 使用 `ProcessGroup`，Windows 使用 `JobObject`。

这里必须区分三件事：

1. `start_kill()` 发出终止请求，不等于已经确认退出。
2. `wait().await` 等待直接 Child 退出并回收退出状态，所以强制终止后不能省略。
3. `KillOnDrop` 只是在包装对象意外被丢弃时尽力终止 Tokio 直接拥有的启动器；它不能代替普通路径中的显式进程树清理。

Tokio 原生 `Child::kill().await` 可以理解为“请求终止直接 Child，再等待它退出”的组合操作；本教程没有调用它，因为我们需要让 `process-wrap` 的 `start_kill()` 按进程组或 Job Object 的语义工作。

## 7.3 为什么退出宽限期使用 `try_wait + sleep`

正常关闭先 `drop(stdin)`，让 Codex 读取到 EOF 并自行退出。`wait_or_kill` 用 `try_wait()` 每 25 毫秒做一次非阻塞检查，期间用 `sleep()` 把执行机会交回 Tokio；两秒到期才强制终止。

这里没有把 `child.wait()` 直接包进 `timeout`。`process-wrap` 在 Unix 上等待进程组时还有组内回收工作；若超时取消一个已经开始的 `wait` Future，再次等待会引入难以推理的竞态。非阻塞轮询只观察状态，没有创建随后会被取消的 `wait`。

## 7.4 逐项对照代码证据

| 设计要求 | 当前代码证据 | 删除后的具体风险 |
|---|---|---|
| 握手不能永久等待 | `timeout(HANDSHAKE_TIMEOUT, handshake(...))` | Codex 不响应时 CLI 永久挂起 |
| 失败路径也必须清理 | 先保存 `handshake_result`，清理后才使用 `?` | EOF 或 JSON 错误会绕过资源回收 |
| 协议结束后允许自然退出 | `drop(stdin)` 与 `drop(reader)` | app-server 可能继续等待输入，父进程也继续持有无用管道 |
| 宽限期不能忙等 | `try_wait()` + `sleep(EXIT_POLL_INTERVAL)` | 循环占满 CPU，或父进程无期限等待 |
| 强制动作覆盖完整进程树 | `ProcessGroup::leader()` / `JobObject` + `start_kill()` | 只杀 Node 启动器，原生 Codex 可能成为遗留进程 |
| 终止后完成回收 | `child.wait().await` | 只发信号却不确认退出，Unix 还可能留下僵尸记录 |
| stderr 始终被排空 | `stderr_task` 持续 `next_line()` | 管道写满后 Codex 可能阻塞 |
| 日志任务不能拖死程序 | `finish_stderr_task` 的一秒期限与 `abort()` | 后代错误持有 stderr 时 CLI 无法结束 |
| 意外析构有最后兜底 | `command.wrap(KillOnDrop)` | 某个未来未覆盖的提前返回更容易留下直接 Child |

## 7.5 当前保证的边界

Phase 0A 保证的是：只要 Rust 进程仍能正常执行异步清理，成功、协议错误和超时这些普通路径都会尝试终止并等待进程树。它不能保证宿主被 `SIGKILL`、系统断电或进程 `abort` 时仍执行析构和清理。

长期运行的 Connector 应把这段职责提升为独立 Process Actor，并加入取消令牌、崩溃重启、退避和外部服务管理器。这里先把一次性进程生命周期做完整，不提前实现长期 Supervisor。

- [ ] 我能解释为什么不能在握手失败后立刻 `?` 返回。
- [ ] 我能区分自然退出宽限期、整组 `start_kill`、最终 `wait` 和 `KillOnDrop` 兜底。

---

# 第 8 章：检查完整代码

> [!question] 为什么要运行三个看起来相似的命令
> 格式、静态分析和行为测试检查的是不同问题。某一项通过不能推出另外两项也通过，因此工程门禁必须分层。

| 检查 | 它能证明什么 | 它不能证明什么 |
|---|---|---|
| `cargo fmt --all --check` | 所有 Rust 文件符合统一格式，而且检查过程没有偷偷改文件 | 代码能否编译、行为是否正确 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Library、Binary 和测试通过编译器/Clippy 静态规则，警告不会被忽略 | 与真实 Codex 的协议是否兼容 |
| `cargo test --workspace` | 当前自动化测试描述的行为全部成立 | 没有写进测试的真实环境、安装路径和进程清理问题 |

> [!success] 本教程选定的方案
> 每完成一个可工作的阶段都按 fmt → Clippy → test 的顺序检查。fmt 最快并先消除排版噪声；Clippy 接着发现编译和静态问题；测试最后证明行为。遇到失败从第一条错误开始修，不通过删除 Lint 或跳过测试来制造“绿色”。

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

预期：6 个测试通过，没有 Clippy 警告。

- [ ] rustfmt 检查、Clippy 和 6 个测试全部通过。
- [ ] 我知道三类检查解决不同问题，不能只跑其中一个代替全部。

---

# 第 9 章：连接真实 Codex

## 9.0 为什么已经有测试还要连接真实 Codex

内存伪 Codex 和真实 Codex Smoke Test 解决不同问题：

| 验证层 | 优点 | 无法证明的内容 |
|---|---|---|
| 内存伪 Codex | 快、确定、无需账号；可以精确断言请求字段和消息顺序 | 当前安装的 Codex 是否仍接受这些字段、真实进程能否启动和退出 |
| 真实 Codex Smoke Test | 覆盖真实可执行文件、当前 app-server 协议、操作系统管道和进程生命周期 | 很难稳定制造每一种错误，也不适合普通 CI 和每次快速测试 |

> [!success] 本教程选定的方案
> 日常回归依赖内存测试，阶段验收再运行真实 Smoke Test。真实测试不能替代伪服务端测试，因为“今天在本机成功一次”不能精确保护每个字段；伪测试也不能替代真实测试，因为假的服务端可能和当前 Codex 一起写错。

进程集合的前后对比是额外的生命周期证据。它不要求电脑上原本没有 Codex 进程，只要求这次运行没有新增遗留进程。

先检查环境：

```bash
rustc --version
cargo --version
which codex
codex --version
```

参考实现使用 Rust 1.96.0 和 Codex 0.144.1。Codex 补丁版本不同不一定是错误；如果协议解析失败，把版本和完整错误发给我。

这四条命令分别回答不同问题：

| 命令 | 它回答的问题 |
|---|---|
| `rustc --version` | 当前目录实际选中了哪个 Rust 编译器 |
| `cargo --version` | 当前使用哪个 Cargo；它通常随同一工具链安装 |
| `which codex` | Shell 从 `PATH` 中会启动哪一个 Codex 文件 |
| `codex --version` | 该文件对应的 Codex 版本是什么 |

`which codex` 没有输出时，不要继续运行写死的路径；先解决安装或 `PATH`。教程命令中的 `/opt/homebrew/bin/codex` 是当前这台 Mac 已验证的结果，不是所有电脑都必须相同。

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

# test 成功时退出码是 0；必须紧接着查看，不能先运行其他命令。
echo $?
```

中间的运行命令也可以从左到右拆开：

| 片段 | 含义 |
|---|---|
| `cargo run` | 必要时先编译，然后运行一个 Binary Crate |
| `-p bridgehub-codex-spike` | 在 Workspace 中明确选择这个 Package |
| 第一个 `--` | 结束 Cargo 自己的参数；其后的内容原样交给我们写的 CLI |
| `--codex-bin` | Clap 定义的 BridgeHub CLI 选项，用来覆盖默认的 `codex` |
| `/opt/homebrew/bin/codex` | 这次要启动的真实可执行文件路径 |

如果握手失败，需要临时查看 Codex 诊断，使用同一条命令并只为这一次运行设置日志环境变量：

```bash
# 等号前设置的 RUST_LOG 只影响这一条 cargo run；诊断写到 stderr。
RUST_LOG=codex_app_server=debug cargo run -p bridgehub-codex-spike -- --codex-bin /opt/homebrew/bin/codex
```

stderr Task 本来就一直读取 Codex 日志；这条命令只是允许 `debug!` 把读到的行显示出来。日志中若包含路径等个人信息，发给我之前先脱敏，绝不要发送 Token 或认证凭据。

这几行 Shell 分别做了以下事情：

| 写法 | 含义 | 为什么需要 |
|---|---|---|
| `$(...)` | 先运行括号中的命令，再把标准输出作为字符串赋值 | 保存运行前后的进程 ID 集合 |
| `pgrep -f` | 在完整命令行中寻找匹配文本，并输出进程 ID | `app-server` 参数不一定出现在短进程名中 |
| `sort` | 把 ID 按稳定顺序排列 | 避免相同集合只因顺序不同而比较失败 |
| `|| true` | 即使没有匹配进程也把这一段视为成功 | “运行前一个也没有”是正常状态，不应中断检查 |
| `test "$after" = "$before"` | 比较两个完整字符串；引号保留空值和换行边界 | 只接受前后集合完全一致 |
| `echo $?` | 打印上一条命令的退出码 | `0` 表示相等，非 `0` 表示有差异 |

已有 Codex app-server 可以同时出现在 `before` 和 `after` 中，这没有问题。我们只检查本次运行是否新增遗留进程。这个命令是有针对性的 Smoke Check，不是操作系统级的形式化证明：若结果不是 `0`，先分别运行 `printf '%s\n' "$before"` 和 `printf '%s\n' "$after"` 查看差异，不要直接终止所有 Codex 进程。

- [ ] 我看到真实握手输出，并且 `test` 后的退出码是 0。
- [ ] 我知道普通输出在 stdout，诊断日志在 stderr，`RUST_LOG` 只控制诊断是否显示。

---

# 第 10 章：回看整个程序

```text
用户启动 Rust CLI
→ Rust 用 ProcessGroup / JobObject 启动并拥有 Codex 进程树
→ Rust 从 ChildWrapper 取得 stdin、stdout、stderr 管道
→ Rust 启动 Tokio Task 持续排空 stderr
→ Rust 发送 initialize 请求
→ Codex 返回 InitializeResponse
→ Rust 解析为 InitializeResponse 结构体
→ Rust 发送 initialized 通知
→ Rust 保存握手成功或失败结果
→ Rust 关闭 stdin 和 stdout Reader
→ Rust 给 Codex 两秒自然退出时间
→ 若仍运行，Rust 终止整个进程组 / Job Object
→ Rust wait 回收并限时收拢 stderr Task
→ 只有握手和清理都成功才打印协商信息
→ Rust 按错误优先级退出
```

## 设计决定最终落在哪里

| 前面作出的工程决定 | 代码或配置落点 | 验证证据 |
|---|---|---|
| 使用虚拟 Workspace，只创建当前需要的成员 | 根 `Cargo.toml` 与 `tools/codex-spike/Cargo.toml` | Cargo 能选择 Package，Workspace 检查全部通过 |
| 固定工具链和统一质量规则 | `rust-toolchain.toml`、`[workspace.lints]` | rustfmt 与 Clippy 结果稳定 |
| 复用 Codex app-server，而不是重写 Agent | `app_server_args()` 和 `CommandWrap` | 真实 Codex 完成握手 |
| 使用 stdio JSONL | `Stdio::piped()`、`BufReader`、`write_json_line` | 伪服务端观察到逐行消息 |
| 协议边界强类型 | `InitializeParams`、`ResponseEnvelope`、`InitializeResponse` | 测试能直接比较 Rust 结构体结果 |
| 错误按恢复含义分类 | `HandshakeError` | EOF、错误 ID、RPC 错误分别有行为测试，解析/I/O 保持独立变体 |
| 协议逻辑与真实进程解耦 | 泛型 `handshake<R, W>` | 同一个函数同时用于 `duplex` 测试和 Child 管道 |
| stderr 与协议输出分离并并发排空 | `stderr_task` | 日志不污染 stdout，管道不会因无人读取而写满 |
| 进程树只有一个生命周期负责人 | `run` 持有 `ChildWrapper` 到最终 `wait` | 普通成功/失败都经过同一清理主干 |
| 终止范围覆盖启动器和原生 Codex | `ProcessGroup::leader()` / `JobObject` | 运行前后目标进程集合一致 |
| 握手等待有边界 | `HANDSHAKE_TIMEOUT` + `timeout(handshake(...))` | 不响应的 Codex 最多阻塞握手十秒 |
| 自然退出优先、整组终止兜底 | `drop(stdin)`、`try_wait`、`start_kill`、`wait` | 正常退出优先，卡住时仍能回收 |
| 意外析构有最后防线 | `KillOnDrop` | 它只兜底直接 Child，不被误当成进程树清理主路径 |

当你以后看到一段代码时，可以反向问三个问题：它在保护哪条约束？哪个测试或运行结果能证明它？删掉后会出现什么具体失败？这三个问题就是从“会照着写”走向“开始具备工程意识”的入口。

已经实现：

- Rust Workspace 和独立 Crate。
- 类型化 JSONL 初始化握手。
- 内存伪 Codex 测试。
- 真实 Codex 进程树启动和跨平台包装。
- stderr 异步排空。
- 握手期限、退出宽限期、整组终止和回收。

尚未实现：

- 聊天和多轮对话。
- Thread、Turn 和流式 Item。
- Tool Calling 与 Approval。
- Session 持久化。
- Relay、Connector 和 Web。

## 常见错误先查哪里

排错顺序始终是“先判断这是教程预期的红色阶段，还是意外错误；再查最靠近错误源的一层”。不要同时改多个文件，否则原始证据会消失。

| 识别信号 | 最可能原因 | 第一条检查命令或动作 | 何时停止并发回输出 |
|---|---|---|---|
| `pwd` 不是项目路径 | 当前目录错误 | `pwd` | 输出不是 `/Users/xulei/.dev/bridge-hub` 时立即停止 |
| 找不到或下载错误工具链 | 文件名、版本或网络问题 | `rustup show active-toolchain`，再检查根目录文件名 | 工具链仍不是 1.96.0，或安装命令失败时 |
| `failed to load manifest` / 找不到成员 | `tool` 与 `tools` 拼错，或 Crate 清单不存在 | `find . -maxdepth 4 -name Cargo.toml -print` | 实际路径与 Workspace `members` 对不上时 |
| TOML `expected ...` | 表头、引号、逗号或 `=` 输入错误 | 从错误给出的文件和行号向上检查 | 修正第一处后仍报同一行时 |
| 预期失败测试第一次就通过 | 文件中已有旧实现，或测试没有覆盖目标 | `git diff -- tools/codex-spike/src/lib.rs` | 不清楚该保留哪段已有代码时；不要为制造红色而删除未知改动 |
| `cannot find function ... in module super` | 红色阶段能力尚未实现，或 `super` 拼写错误 | 对照当前步骤：此处是否明确写着“预期失败” | 非预期失败阶段仍出现时 |
| 找不到 `bridgehub_codex_spike` | Package 名、Library 名或依赖清单不一致 | 检查 Crate `Cargo.toml` 的 `name` | 名称正确但仍无法导入时 |
| `invalid JSON from app-server` | 输入代码字段有误，或安装的 Codex 协议已变化 | `codex --version`，再检查完整原始错误 | 不要猜字段；把版本和错误一起发回 |
| `UnexpectedEof` | Codex 在响应前退出 | 使用 `RUST_LOG=codex_app_server=debug` 重跑一次 | stderr 显示认证、配置、崩溃或未知错误时 |
| RPC `Server` 错误 | Codex 明确拒绝初始化 | 保留错误码和完整消息 | 不要把它改成 `MissingResult`；直接发回原文 |
| `handshake exceeded 10s` | Codex 启动或 initialize 无响应 | `codex --version`，再带 debug 日志重跑 | 第二次仍超时，或出现残留进程差异时 |
| 找不到 Codex | 路径错误或未安装 | `which codex` | 没有输出，或输出与 `--codex-bin` 不一致时 |
| Codex 提示未登录/无凭据 | 本机 Codex 尚未完成认证 | 先在普通终端按 Codex 自身流程完成登录 | 不要把凭据贴进聊天；认证仍失败时只发脱敏错误 |
| `app-server exited with ...` | 握手后 Codex 非成功退出 | 打开 `RUST_LOG=codex_app_server=debug` 查看诊断 | 不能从日志明确判断原因时 |
| 运行后 `echo $?` 非 0 | 前后目标进程集合不同 | 分别打印 `$before` 和 `$after` | 不要执行 `pkill`；把两个集合发回 |
| Clippy 报 `unwrap_used` / `expect_used` | 使用了项目明确禁止的崩溃式取值 | 从第一条 Warning 所在行开始改为传播或匹配错误 | 不知道该返回哪种错误时 |
| Clippy 报未使用内容 | 导入、变量或函数没有真实调用方 | 确认是否漏敲下一步调用，再删除真正多余项 | 代码应被使用但找不到调用位置时 |

遇到教程未覆盖的错误时，不继续改代码。把下面三项发给我：

1. 你执行的完整命令。
2. 从第一行到最后一行的完整输出。
3. 当前正在编辑的文件绝对路径。

- [ ] 我能从启动、握手、统一清理到最终输出按顺序讲完整个程序。
- [ ] 我知道 Phase 0A 尚未实现 Thread、Turn、Tool Calling、Session、Relay、Connector 和 Web。
