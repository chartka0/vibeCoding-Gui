<script setup lang="ts">
import { ref, computed, nextTick, onUnmounted } from "vue";
import {
  NCard, NButton, NIcon, NInput, NSpin, NScrollbar, NTag,
  NAlert, NGradientText
} from "naive-ui";
import {
  SearchOutline, BulbOutline, MapOutline, CodeSlashOutline,
  SparklesOutline, CheckmarkCircleOutline, PlayOutline,
  RefreshOutline, ChevronDownOutline, ChevronUpOutline,
  FolderOpenOutline, ArrowForwardOutline, StopCircleOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Types ──
interface StepDef {
  id: string;
  name: string;
  desc: string;
  icon: any;
  promptBuilder: (userPrompt: string, feedback: string) => string;
}

interface StepState {
  status: 'pending' | 'running' | 'success' | 'failed';
  output: string;
  startTime: number | null;
  endTime: number | null;
  expanded: boolean;
  retryCount: number;
}

interface StepDonePayload {
  step_index: number;
  status: string;
  session_id: string | null;
}

// ── Step definitions ──
const STEPS: StepDef[] = [
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

// ── State ──
const workspaceStore = useWorkspaceStore();
const userPrompt = ref('');
const runId = ref<string | null>(null);
const sessionId = ref<string | null>(null);
const activeStep = ref(-1);           // current running step index
const waitingForUser = ref(false);     // waiting for user decision
const userFeedback = ref('');          // user input between steps
const workflowDone = ref(false);

const stepStates = ref<StepState[]>(STEPS.map(() => ({
  status: 'pending',
  output: '',
  startTime: null,
  endTime: null,
  expanded: false,
  retryCount: 0,
})));

// ── Computed ──
const canStart = computed(() =>
  userPrompt.value.trim() && workspaceStore.currentWorkspace && activeStep.value === -1
);
const isRunning = computed(() => activeStep.value >= 0 && !waitingForUser.value && !workflowDone.value);

// ── Listeners ──
let unlisteners: UnlistenFn[] = [];

async function listenStep(rId: string, stepIdx: number) {
  // Clean previous listeners
  for (const fn of unlisteners) fn();
  unlisteners = [];

  const logUn = await listen<string>(`wf-step-log-${rId}-${stepIdx}`, (event) => {
    stepStates.value[stepIdx].output += event.payload;
    nextTick(scrollOutputToBottom);
  });
  unlisteners.push(logUn);

  const doneUn = await listen<StepDonePayload>(`wf-step-done-${rId}`, (event) => {
    const p = event.payload;
    if (p.step_index !== stepIdx) return;

    const s = stepStates.value[stepIdx];
    s.endTime = Date.now();

    if (p.status === 'success') {
      s.status = 'success';
      if (p.session_id) sessionId.value = p.session_id;

      // Last step? workflow done
      if (stepIdx === STEPS.length - 1) {
        workflowDone.value = true;
      } else {
        waitingForUser.value = true;
        userFeedback.value = '';
      }
    } else {
      s.status = 'failed';
      waitingForUser.value = true; // allow retry
    }
  });
  unlisteners.push(doneUn);
}

// ── Actions ──
async function startWorkflow(mode: 'workflow' | 'frontend' | 'backend' = 'workflow') {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !userPrompt.value.trim()) return;

  // Reset
  workflowDone.value = false;
  waitingForUser.value = false;
  userFeedback.value = '';
  sessionId.value = crypto.randomUUID();
  stepStates.value = STEPS.map(() => ({
    status: 'pending', output: '', startTime: null, endTime: null, expanded: false, retryCount: 0,
  }));

  try {
    const id: string = await invoke('start_workflow_run', {
      workspaceId: ws.id,
      mode,
      prompt: userPrompt.value,
    });
    runId.value = id;
    await runStep(0);
  } catch (e: any) {
    console.error('Start failed', e);
  }
}

async function runStep(index: number) {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !runId.value || !sessionId.value) return;

  activeStep.value = index;
  waitingForUser.value = false;
  const s = stepStates.value[index];
  s.status = 'running';
  s.output = '';
  s.startTime = Date.now();
  s.endTime = null;
  s.expanded = true;

  // Collapse previous steps
  stepStates.value.forEach((st, i) => { if (i !== index) st.expanded = false; });

  const prompt = STEPS[index].promptBuilder(userPrompt.value, userFeedback.value);

  await listenStep(runId.value, index);

  invoke('run_workflow_step', {
    runId: runId.value,
    stepIndex: index,
    stepName: STEPS[index].name,
    workspacePath: ws.path,
    prompt,
    sessionId: sessionId.value,
  }).catch((e: any) => {
    s.status = 'failed';
    s.output += `\n[错误] ${e}`;
    s.endTime = Date.now();
    waitingForUser.value = true;
  });
}

function continueToNext() {
  const nextIdx = activeStep.value + 1;
  if (nextIdx < STEPS.length) {
    runStep(nextIdx);
  }
}

function retryCurrentStep() {
  const idx = activeStep.value;
  if (idx >= 0) {
    stepStates.value[idx].retryCount++;
    runStep(idx);
  }
}

function resetWorkflow() {
  for (const fn of unlisteners) fn();
  unlisteners = [];
  runId.value = null;
  sessionId.value = null;
  activeStep.value = -1;
  waitingForUser.value = false;
  workflowDone.value = false;
  userFeedback.value = '';
  stepStates.value = STEPS.map(() => ({
    status: 'pending', output: '', startTime: null, endTime: null, expanded: false, retryCount: 0,
  }));
}

function toggleExpand(index: number) {
  stepStates.value[index].expanded = !stepStates.value[index].expanded;
}

function formatDuration(start: number | null, end: number | null): string {
  if (!start) return '';
  const elapsed = (end || Date.now()) - start;
  if (elapsed < 1000) return `${elapsed}ms`;
  const secs = Math.floor(elapsed / 1000);
  if (secs < 60) return `${secs}s`;
  return `${Math.floor(secs / 60)}m ${secs % 60}s`;
}

function scrollOutputToBottom() {
  // auto-scroll handled by n-scrollbar
}

onUnmounted(() => {
  for (const fn of unlisteners) fn();
});
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%; gap: 16px;">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning" style="border-radius: 12px;">
      请先在左侧边栏选择或创建一个项目 (Workspace) 以启动工作流。
    </n-alert>

    <!-- ─── Hero / Input ─── -->
    <n-card style="border-radius: 12px; background: linear-gradient(135deg, #18181c 0%, #1a2332 100%); flex-shrink: 0;">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
        <n-gradient-text type="info" :size="22" style="font-weight: 800;">
          vibeCoding Workflow
        </n-gradient-text>
        <n-tag v-if="workspaceStore.currentWorkspace" type="success" size="small" round>
          <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
          {{ workspaceStore.currentWorkspace.name }}
        </n-tag>
      </div>

      <n-input
        v-model:value="userPrompt"
        type="textarea"
        placeholder="输入你的需求，例如：帮我写一个基于 Vue3 的 Todo 列表页面，支持本地存储..."
        :autosize="{ minRows: 2, maxRows: 5 }"
        style="border-radius: 8px; font-size: 13px; margin-bottom: 16px;"
        :disabled="isRunning || waitingForUser"
      />

      <div v-if="activeStep === -1" style="display: flex; gap: 10px;">
        <n-button
          type="info" size="medium" style="border-radius: 8px; font-weight: 600;"
          :disabled="!canStart"
          @click="startWorkflow('workflow')"
        >
          <template #icon><n-icon><PlayOutline /></n-icon></template>
          完整工作流
        </n-button>
        <n-button
          type="success" secondary size="medium" style="border-radius: 8px; font-weight: 600;"
          :disabled="!canStart"
          @click="startWorkflow('frontend')"
        >
          极速前端 (Gemini)
        </n-button>
        <n-button
          type="warning" secondary size="medium" style="border-radius: 8px; font-weight: 600;"
          :disabled="!canStart"
          @click="startWorkflow('backend')"
        >
          极速后端 (Codex)
        </n-button>
      </div>
      <div v-else style="display: flex; align-items: center; gap: 12px;">
        <n-spin v-if="isRunning" :size="16" />
        <span v-if="isRunning" style="color: #888; font-size: 13px;">
          {{ STEPS[activeStep]?.name }} 执行中...
        </span>
        <span v-else-if="workflowDone" style="color: #63e2b7; font-size: 13px; font-weight: 600;">
          工作流已完成
        </span>
        <n-button
          type="error" ghost size="small" style="border-radius: 8px; margin-left: auto;"
          @click="resetWorkflow"
        >
          <template #icon><n-icon><StopCircleOutline /></n-icon></template>
          重置
        </n-button>
      </div>
    </n-card>

    <!-- ─── Step Cards ─── -->
    <div v-if="runId" style="flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 12px;">
      <div v-for="(step, idx) in STEPS" :key="step.id">
        <n-card
          size="small"
          :style="{
            borderRadius: '12px',
            borderLeft: `3px solid ${
              stepStates[idx].status === 'success' ? '#63e2b7' :
              stepStates[idx].status === 'failed' ? '#f85149' :
              stepStates[idx].status === 'running' ? '#58a6ff' : '#333'
            }`,
            opacity: stepStates[idx].status === 'pending' ? 0.5 : 1,
            transition: 'all 0.3s ease',
          }"
        >
          <!-- Step Header -->
          <div
            style="display: flex; align-items: center; gap: 12px; cursor: pointer; user-select: none;"
            @click="stepStates[idx].output && toggleExpand(idx)"
          >
            <!-- Icon -->
            <n-icon :size="20" :color="
              stepStates[idx].status === 'success' ? '#63e2b7' :
              stepStates[idx].status === 'failed' ? '#f85149' :
              stepStates[idx].status === 'running' ? '#58a6ff' : '#555'
            ">
              <component :is="step.icon" />
            </n-icon>

            <!-- Name + Desc -->
            <div style="flex: 1; min-width: 0;">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-size: 14px; font-weight: 600; color: #e0e0e0;">
                  {{ idx + 1 }}. {{ step.name }}
                </span>
                <n-tag v-if="stepStates[idx].status === 'running'" type="info" size="tiny" round>
                  <n-spin :size="10" style="margin-right: 4px;" /> 运行中
                </n-tag>
                <n-tag v-else-if="stepStates[idx].status === 'success'" type="success" size="tiny" round>
                  完成
                </n-tag>
                <n-tag v-else-if="stepStates[idx].status === 'failed'" type="error" size="tiny" round>
                  失败
                  <span v-if="stepStates[idx].retryCount > 0" style="margin-left: 4px; opacity: 0.7;">
                    (重试 {{ stepStates[idx].retryCount }}次)
                  </span>
                </n-tag>
              </div>
              <div style="font-size: 11px; color: #666; margin-top: 2px;">{{ step.desc }}</div>
            </div>

            <!-- Duration -->
            <span v-if="stepStates[idx].startTime" style="font-size: 11px; color: #555; font-family: monospace;">
              {{ formatDuration(stepStates[idx].startTime, stepStates[idx].endTime) }}
            </span>

            <!-- Expand toggle -->
            <n-icon v-if="stepStates[idx].output" :size="16" color="#555">
              <component :is="stepStates[idx].expanded ? ChevronUpOutline : ChevronDownOutline" />
            </n-icon>
          </div>

          <!-- Step Output (expandable) -->
          <div
            v-if="stepStates[idx].expanded && stepStates[idx].output"
            style="margin-top: 12px; border-top: 1px solid #262626; padding-top: 12px;"
          >
            <n-scrollbar style="max-height: 300px;">
              <pre style="
                font-family: 'Cascadia Code', Consolas, monospace;
                font-size: 12px;
                line-height: 1.6;
                white-space: pre-wrap;
                word-break: break-word;
                color: #c9d1d9;
                margin: 0;
              ">{{ stepStates[idx].output }}</pre>
            </n-scrollbar>
          </div>

          <!-- ─── Decision Area (between steps) ─── -->
          <div
            v-if="waitingForUser && idx === activeStep"
            style="margin-top: 12px; border-top: 1px solid #262626; padding-top: 12px;"
          >
            <template v-if="stepStates[idx].status === 'success'">
              <n-input
                v-model:value="userFeedback"
                type="textarea"
                :placeholder="idx === STEPS.length - 1
                  ? '工作流已到最后一步'
                  : '输入反馈、选择方案、或留空直接继续...'"
                :autosize="{ minRows: 1, maxRows: 3 }"
                style="border-radius: 8px; font-size: 12px; margin-bottom: 10px;"
              />
              <div style="display: flex; gap: 8px;">
                <n-button
                  v-if="idx < STEPS.length - 1"
                  type="info" size="small" style="border-radius: 8px;"
                  @click="continueToNext"
                >
                  <template #icon><n-icon><ArrowForwardOutline /></n-icon></template>
                  继续下一步
                </n-button>
                <n-button
                  type="warning" ghost size="small" style="border-radius: 8px;"
                  @click="retryCurrentStep"
                >
                  <template #icon><n-icon><RefreshOutline /></n-icon></template>
                  重新执行
                </n-button>
              </div>
            </template>

            <template v-else-if="stepStates[idx].status === 'failed'">
              <n-alert type="error" style="margin-bottom: 10px; border-radius: 8px;" :show-icon="false">
                步骤执行失败，可以重试或修改反馈后重试。
              </n-alert>
              <n-input
                v-model:value="userFeedback"
                type="textarea"
                placeholder="修改指令或补充信息（可选）..."
                :autosize="{ minRows: 1, maxRows: 3 }"
                style="border-radius: 8px; font-size: 12px; margin-bottom: 10px;"
              />
              <n-button
                type="warning" size="small" style="border-radius: 8px;"
                @click="retryCurrentStep"
              >
                <template #icon><n-icon><RefreshOutline /></n-icon></template>
                重试此步骤
              </n-button>
            </template>
          </div>
        </n-card>
      </div>

      <!-- ─── Workflow Complete ─── -->
      <n-card v-if="workflowDone" style="border-radius: 12px; border: 1px solid #63e2b7; text-align: center;">
        <n-gradient-text type="success" :size="16" style="font-weight: 700;">
          工作流全部完成
        </n-gradient-text>
        <div style="color: #888; font-size: 12px; margin-top: 4px;">
          所有 {{ STEPS.length }} 个阶段均已执行完毕
        </div>
        <n-button type="info" ghost size="small" style="margin-top: 12px; border-radius: 8px;" @click="resetWorkflow">
          开始新的工作流
        </n-button>
      </n-card>
    </div>
  </div>
</template>
