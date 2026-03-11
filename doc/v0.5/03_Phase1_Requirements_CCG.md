# vibeCoding 工作流产品需求文档 (v0.4) - 阶段1: 需求定义与上下文构建 (Phase 1)

## 1. 目标与定位
Phase 1 是整个工作流的灵魂。受 CCG Workflow (Context-Creation-Generation) 理念启发，本阶段的核心目标是将人类模糊、发散的想法，通过合理的“上下文构建(Context)”和“提示工程(Prompting)”，转化为AI模型（Claude/Codex/Gemini）能够**无歧义执行**的高质量工程文档。

## 2. 参与角色交互
- **Human (用户)**：提供原始需求（文字、语音、草图、参考截图）。
- **Gemini (UI/UX 侧)**：专门负责读取由 Human 提供的视觉材料，提取 UI 布局、配色意向和组件级结构。
- **Claude (Tech Lead 侧)**：统筹全局，负责将抽象需求细化为数据结构设计、边界异常分析和接口定义。

## 3. 标准处理流 (The CCG Context-Creation Flow)

### 步骤 1.1: 原始输入收集 (Input Collection)
- **动作**：用户在 GUI 界面输入文字描述，或拖拽上传设计草图/参考网页截图。
- **系统处理**：Tauri 容器将输入材料进行分类打包。如果是纯文本，直接压入主控队列；如果有图像图纸，则优先推送给 Gemini 进行视觉树(Visual UI Tree)预解析。

### 步骤 1.2: 上下文对齐与追问 (Context Alignment)
- **动作**：AI 不会立即开始写代码，而是进入“需求澄清循环”。
- **系统处理**：
  1. Claude 结合 Gemini 的视觉解析结果，生成一份**初步产品梳理单**。
  2. 如果发现关键信息缺失（例如“登录失败后是否限制尝试次数？”、“该按钮在移动端的展现形式是什么？”），Claude 会主动在界面上向用户发起**追问 (Clarification Prompt)**。
  3. 用户逐一回答，直至 Claude 认为上下文(Context)已完全闭环且充分。

### 步骤 1.3: 工程文档生成 (Engineering Doc Generation)
- **动作**：确认无歧义后，系统自动生成“工程化执行文档”。
- **系统处理**：Claude 依据收集到的完备 Context，自动在项目隐藏目录（如 `.vibecoding/specs/`）下生成结构化的 Markdown 格式 Task 说明书。
- **文档包应包含**：
  - `UI_Spec.md`: 前端组件库(Naive UI)规范、状态管理、DOM 结构树。
  - `Logic_Spec.md`: 后端数据流、API 接口契约、核心算法伪代码。
  - `Test_Spec.md`: 需求对应的验收覆盖标准（Acceptance Criteria）。

## 4. 人工卡点 (Human-in-the-loop)
- **卡点位置**：在步骤 1.3 生成工程文档后、实际派发给 Codex/Gemini 编码前。
- **卡点形式**：用户界面会弹出一个“需求评审面板”，左右分栏。左侧为用户的初始需求，右侧为 Claude 翻译成的 `Task 说明书` 关键摘要。用户点击 [Approve & Build] 后，工作流正式进入 Phase 2。
