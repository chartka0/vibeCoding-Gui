# vibeCoding 工作流产品需求文档 (PRD v0.3)

## 1. 产品概述
### 1.1 背景
当前AI辅助编程已经成为趋势，针对碎片化的“问答式代码生成”，我们希望打造一款“vibeCoding”工作流驱动的**跨平台桌面端应用**。

### 1.2 目标与定位
本产品是一款支持 Windows 和 macOS 的桌面级应用。基于 **Tauri + Vue** 构建轻量化、高性能的本地客户端。在其核心调度引擎中，联动 **Claude**、**Codex** 和 **Gemini** 三大模型的 CLI 工具，实现由“需求 -> 分发 -> 开发 (逻辑+UI) -> 审查 -> 提交流程”的全自动化/半自动化闭环。

## 2. 核心架构：多智能体(Multi-Agent)协作模型与技术栈

### 2.1 技术栈选型
- **底层容器与系统调度**：Tauri (Rust)
  - *优势*：极低的内存占用，极小的打包体积，原生的多进程管道(Pipeline)调度能力，完美契合后台高频挂起与调用多个 CLI 的场景。
- **前端页面框架**：Vue 3 (Composition API)
- **UI 组件库**：待定 (需满足现代化、类IDE、极客感等风格)

### 2.2 模型角色分工
- **Claude (Tech Lead / 审查员 / Orchestrator)**：
  - **职责**：负责任务分解与派发、全局代码审查(Code Review)、检查代码规范，并控制整体工作流的流转。Tauri 主进程将大量与之通信，进行流程状态流转。
- **Codex (Backend / Core Developer)**：
  - **职责**：负责复杂算法编写、核心业务逻辑实现、数据处理等纯代码层面的生产能力。
- **Gemini (Frontend UI/UX / BA)**：
  - **职责**：负责前端页面的 UI/交互层面代码生成、读取并解析图文并茂的需求文档/设计稿，将视觉和文档转化为结构化任务。

## 3. 标准工作流定义 (The vibeCoding Flow)

### Phase 1: 需求解析 (Idea -> Task)
- **动作**：用户在 Tauri 前端输入需求或导入文档，**Gemini** 负责阅读理解文档并输出 UI/交互要点，**Claude** 结合这些要点拆解出开发任务列表，反馈至 UI 面板。

### Phase 2: 并行开发 (Task -> Code)
- **动作**：
  - 子流1：Tauri 唤起 **Codex** CLI 接收核心代码与算法任务，专注输出业务逻辑。
  - 子流2：Tauri 唤起 **Gemini** CLI 接收前端绘制任务，专注输出 Vue 结构化代码。

### Phase 3: 审查与自愈 (Code -> Validated)
- **动作**：**Claude** 接管产出的所有代码，进行静态扫描和逻辑推演。如果报错或代码规范不符，Claude 生成修复建议并由 Tauri 重新派发给 Codex 或 Gemini 进行 Self-healing。

### Phase 4: 交付存档 (Validated -> Repo)
- **动作**：所有测试与 Review 通过，应用底层（Rust端）自动执行常规的 Git 提交流程完成版本沉淀。
