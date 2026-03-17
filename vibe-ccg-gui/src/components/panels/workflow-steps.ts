import {
  SearchOutline, BulbOutline, MapOutline, CodeSlashOutline,
  SparklesOutline, CheckmarkCircleOutline,
} from "@vicons/ionicons5";
import type { Component } from "vue";

export interface StepDef {
  id: string;
  name: string;
  desc: string;
  icon: Component;
  promptBuilder: (userPrompt: string, feedback: string) => string;
}

export const STEPS: StepDef[] = [
  {
    id: 'research', name: '研究分析', desc: '需求理解、上下文收集、完整性评估',
    icon: SearchOutline,
    promptBuilder: (p, _f) =>
      `你正在进行结构化开发工作流的【研究分析】阶段。请全面分析以下需求：\n\n${p}\n\n请输出：\n1. 核心目标与预期成果\n2. 技术约束与依赖\n3. 隐含假设\n4. 缺失信息或模糊点\n5. 需求完整性评分 (0-10)\n\n请用中文回答，简洁专业。`,
  },
  {
    id: 'ideation', name: '方案构思', desc: '多方案对比、技术可行性分析',
    icon: BulbOutline,
    promptBuilder: (_p, f) =>
      `${f ? `用户反馈：${f}\n\n` : ''}请进入【方案构思】阶段。基于前面的研究分析，提出 2-3 个实现方案。\n\n每个方案包含：\n- 架构概述\n- 关键技术选型\n- 优点与缺点\n- 复杂度评估 (低/中/高)\n\n最后推荐最佳方案并说明理由。`,
  },
  {
    id: 'planning', name: '详细规划', desc: '架构设计、实施步骤分解',
    icon: MapOutline,
    promptBuilder: (_p, f) =>
      `${f ? `用户选择/反馈：${f}\n\n` : ''}请进入【详细规划】阶段。制定详细实施计划：\n\n1. 需要新建/修改的文件清单\n2. 模块/组件划分\n3. API 设计（如适用）\n4. 分步实施顺序\n5. 风险点与应对措施`,
  },
  {
    id: 'execution', name: '编码实施', desc: '按计划编码、里程碑反馈',
    icon: CodeSlashOutline,
    promptBuilder: (_p, f) =>
      `${f ? `用户备注：${f}\n\n` : ''}请进入【编码实施】阶段。严格按照批准的计划实施代码变更。\n\n- 遵循项目现有代码风格\n- 编写清晰、结构良好的代码\n- 包含必要的错误处理`,
  },
  {
    id: 'optimization', name: '优化审查', desc: '安全审查、性能优化、质量检查',
    icon: SparklesOutline,
    promptBuilder: (_p, f) =>
      `${f ? `用户备注：${f}\n\n` : ''}请进入【优化审查】阶段。审查并优化已实现的代码：\n\n1. 安全性审查\n2. 性能分析与优化\n3. 代码质量检查\n4. 错误处理完善\n5. 应用必要的优化改进`,
  },
  {
    id: 'review', name: '最终评审', desc: '完成度检查、测试验证、交付',
    icon: CheckmarkCircleOutline,
    promptBuilder: (_p, f) =>
      `${f ? `用户备注：${f}\n\n` : ''}请进入【最终评审】阶段：\n\n1. 对照计划检查完成情况\n2. 验证所有需求已满足\n3. 总结所有变更\n4. 列出遗留问题或 TODO\n5. 整体质量评分 (0-10)`,
  },
];
