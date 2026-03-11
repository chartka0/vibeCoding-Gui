# vibeCoding 工作流产品需求文档 (v0.4) - 标准工作流定义

本文档详细定义了基于多智能体(Multi-Agent)协作的“vibeCoding”标准工作流 (The vibeCoding Flow)。

## 1. Phase 1: 需求解析与派发 (Idea -> Task)
- **动作**：用户在 Tauri 前端输入一段需求，或者导入 Markdown/图文稿件。
- **执行过程**：
  1. 系统自动唤起 **Gemini** CLI 去阅读和理解原始文档，尤其侧重于提取 UI 组件、交互状态、视觉要求等信息。
  2. **Claude** 拿到 Gemini 解析出的关键点后，结合全局系统的架构，将这段需求拆解为可被系统独立执行的“前端任务列表”和“核心逻辑任务列表”。
  3. 任务拆解的结果将反馈至 Naive UI 面板，等待用户的最终确认(Human-in-the-loop)。

## 2. Phase 2: 并行开发与编码 (Task -> Code)
- **动作**：任务拆解确认后，系统同时分发任务进入编码管道。
- **执行过程**：
  - **前端视图层 (Gemini Pipeline)**：Tauri 唤起 Gemini CLI，针对前端的绘制任务输出带有 Naive UI 组件库规范的 Vue 3 (Composition API) 代码。
  - **核心逻辑层 (Codex Pipeline)**：Tauri 唤起 Codex CLI，接收业务逻辑及后端算法任务，输出可靠的逻辑代码骨架或功能闭环。

## 3. Phase 3: 审查与自我修复 (Code -> Validated)
- **动作**：编码阶段产出的所有文件并不直接落盘作为最终版本，而是进入审查池。
- **执行过程**：
  1. **Claude** 接管产出的所有代码，进行静态扫描和上下文比对。
  2. 如果发现报错、接口不对齐、或没有遵循设定好的代码规范，Claude 将生成“自愈提示(Self-healing Prompt)”。
  3. Tauri 主程序拦截这些提示，并将打回的内容重新派发给 Codex 或 Gemini，要求它们根据反馈**重新修改代码**，直至通过 Claude 的审核。

## 4. Phase 4: 交付存档 (Validated -> Repo)
- **动作**：所有测试与代码 Review 通过。
- **执行过程**：
  1. 应用底层（Rust 端）生成符合规范的 Git Commit Message (例如：`feat(ui): 新增左侧任务栏拆解面板`)。
  2. 自动或半自动地执行常规的 `git add` 与 `git commit` 操作，完成研发版本的沉淀存档。
