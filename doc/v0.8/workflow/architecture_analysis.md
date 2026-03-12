# v0.8 完整工作流架构分析 — CCG 真实运行模型

## 1. 核心发现：CCG 的真实运行架构

经过对 `ccg-workflow` 仓库源码的完整审计（`installer.ts`、`workflow.md`、`frontend.md`、`codeagent-wrapper/main.go`），得出以下**决定性结论**：

### CCG 不是一个可以被 spawn 的独立进程

CCG 的本质是一套**安装进 Claude Code CLI 的 Markdown 提示词模板**。

```
npx ccg-workflow (npm 安装器)
     │
     └─→ 将 26 个 .md 文件 copy 到  ~/.claude/commands/ccg/
         将 13 个专家角色提示词 copy 到  ~/.claude/.ccg/prompts/
         将 codeagent-wrapper 二进制 copy 到  ~/.claude/bin/
```

用户使用时，是在 **Claude Code CLI 会话内部**输入 `/ccg:workflow 需求描述`，Claude Code 读取对应的 `.md` 模板，然后**作为 Claude 自己的行为指令**来执行那 6 个阶段。

### 运行时调用链

```
用户在 Claude Code CLI 输入: /ccg:workflow 帮我加个登录页
              │
              ▼
Claude Code 读取 ~/.claude/commands/ccg/workflow.md
              │
              ▼
Claude (编排者) 按 workflow.md 中的 6 阶段指令执行:
   [模式：研究] → Claude 自己做 Prompt 增强 + 项目扫描
   [模式：构思] → Claude 调用 codeagent-wrapper 启动 Codex 和 Gemini 并行分析
   [模式：计划] → Claude 综合两方输出，写入 .claude/plan/xxx.md
   [模式：执行] → Claude 自己写代码（可能再次调用外部模型）
   [模式：优化] → Claude 再次并行调用 Codex/Gemini 做代码审查
   [模式：评审] → Claude 自己做最终验证
              │
              ▼
每个阶段结束后，Claude 会 AskUserQuestion 请求用户确认
（用户说"继续"才进入下一阶段）
```

### codeagent-wrapper 的角色

`codeagent-wrapper` 是一个 Go 编写的**子进程桥接器**：
- 被 Claude Code 内部通过 `Bash()` 工具调用
- 它的职责是启动 `codex` CLI 或 `gemini` CLI，把任务和角色提示词传给它们
- 它本身不是工作流引擎，只是一个"信使"

---

## 2. 对我们 Tauri 桌面端的架构影响

### ❌ 我们之前的错误假设

我们在 `lib.rs` 中的 `start_workflow` 试图：
```rust
// 错误！ccg-workflow 不是一个可以这样跑的命令
let mut cmd = AsyncCommand::new("ccg-workflow");
cmd.arg("/ccg:workflow").arg(&prompt);
```

这完全不可能工作，因为：
1. `ccg-workflow` 是一个 npm 安装器，不是运行时命令
2. `/ccg:workflow` 是 Claude Code 的斜杠命令，只能在 Claude Code CLI 会话里使用
3. 整个工作流是**交互式的**（每步需要用户确认），不是全自动的

### ✅ 正确的集成方案

我们的 Tauri 桌面端要和 CCG 工作流集成，有以下可行路径：

#### 方案 A：内嵌 Claude Code CLI 终端 (推荐)

在桌面端内嵌一个 PTY 终端（类似 VS Code 内置终端），让用户直接在里面跑 `claude` 命令：
- 用户在 GUI 输入 Prompt → 桌面端自动组装成 `claude "/ccg:workflow 需求描述"` 命令
- 在内嵌终端中执行，用户可以看到实时输出并与 Claude 交互（确认每个阶段）
- 桌面端监听终端输出中的 `[模式：X]` 标签，自动更新 UI 上的步骤条

**优点**：100% 兼容 CCG 原生行为，包括交互式确认
**缺点**：需要引入 PTY 库（如 `portable-pty`）

#### 方案 B：API 模式调用 Claude (进阶)

如果 Claude Code 支持 API/SDK 模式（非交互式），可以：
- 通过 API 发送带 `/ccg:workflow` 的消息
- 流式接收输出并解析
- 但目前 Claude Code 的 API 模式对斜杠命令的支持有限

#### 方案 C：拆解为 GUI 原生编排 (最灵活但最重)

不依赖 Claude Code CLI，而是：
- 桌面端自己读取 `~/.claude/commands/ccg/workflow.md` 的流程定义
- 自己实现 6 阶段编排逻辑
- 直接调用 `codeagent-wrapper` 与 Codex/Gemini 通信
- Claude 的部分通过 API 调用实现

**优点**：完全自主控制，可以做出最精美的 UI
**缺点**：工作量巨大，相当于重写整个 Claude Code 的编排层

---

## 3. 推荐实施路径

**短期 (v0.8)**：方案 A — 内嵌终端
- 在 WorkflowPanel 中嵌入一个终端组件
- 用户填写 Prompt 后，自动在终端中执行 `claude "/ccg:workflow {prompt}"`
- 解析终端输出，驱动 UI 步骤条
- 保留当前的 4 模式按钮，映射为不同的 `/ccg:*` 命令

**中期 (v1.0)**：方案 A + B 混合
- 对于不需要交互确认的简单命令（如 `/ccg:frontend`、`/ccg:backend`），可以用非交互模式
- 对于复杂的 `/ccg:workflow`，保留终端交互

**长期 (v2.0)**：方案 C
- 完全自主编排，脱离 Claude Code CLI 依赖
