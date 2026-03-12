<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  NCard, NButton, NIcon, NSteps, NStep,
  NGradientText, NTag, NAlert, NInput, NSpin, NScrollbar, NCollapse, NCollapseItem
} from "naive-ui";
import {
  RocketOutline, DocumentTextOutline, ChatbubblesOutline,
  CodeSlashOutline, GitBranchOutline, CheckmarkDoneOutline,
  FlashOutline, BrowsersOutline, FolderOpenOutline, RefreshOutline, PlayOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

const workspaceStore = useWorkspaceStore();
const workflowPrompt = ref('');
const currentStepIndex = ref(0); // 0 = not started
const runId = ref<string | null>(null);
const isRunning = ref(false);
const logs = ref<string[]>([]);

const workflowSteps = [
  { id: 'research',     title: '研究',  desc: '需求收集、上下文与分析',        icon: DocumentTextOutline },
  { id: 'ideation',    title: '构思',  desc: '双模型并行分析可行性',            icon: ChatbubblesOutline },
  { id: 'planning',    title: '计划',  desc: '多模型产出前后端架构规划',        icon: CodeSlashOutline },
  { id: 'execution',   title: '执行',  desc: '严格按批准计划编码实施',          icon: FlashOutline },
  { id: 'optimization',title: '优化',  desc: '多模型并行审查安全与设计',        icon: GitBranchOutline },
  { id: 'review',      title: '评审',  desc: '最终评估与测试闭环交付',          icon: CheckmarkDoneOutline },
];

// Map [模式：X] tag → step index
const modeToStep: Record<string, number> = {
  '研究': 1, '研究与分析': 1,
  '构思': 2, '方案构思': 2,
  '计划': 3, '详细规划': 3,
  '执行': 4, '实施': 4,
  '优化': 5, '代码优化': 5,
  '评审': 6, '质量审查': 6,
};

const stepStatus = ref<Record<number, 'wait' | 'process' | 'finish' | 'error'>>({
  1: 'wait', 2: 'wait', 3: 'wait', 4: 'wait', 5: 'wait', 6: 'wait'
});

let unlistenLog: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;

async function startWorkflow(skill: 'ccg:workflow' | 'ccg:frontend' | 'ccg:backend') {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !workflowPrompt.value.trim()) return;

  isRunning.value = true;
  logs.value = [];
  stepStatus.value = { 1: 'wait', 2: 'wait', 3: 'wait', 4: 'wait', 5: 'wait', 6: 'wait' };
  currentStepIndex.value = 0;

  try {
    const id: string = await invoke('start_workflow_run', {
      workspaceId: ws.id,
      mode: skill,
      prompt: workflowPrompt.value
    });
    runId.value = id;

    // Listen for log lines
    if (unlistenLog) unlistenLog();
    unlistenLog = await listen<string>(`step-log-${id}`, (event) => {
      const line = event.payload;
      logs.value.push(line);

      // Detect [模式：X] tags to drive step indicator
      const match = line.match(/\[模式[：:]\s*([^\]]+)\]/);
      if (match) {
        const mode = match[1].trim();
        const step = modeToStep[mode];
        if (step) {
          // Mark previous step done
          if (currentStepIndex.value > 0) {
            stepStatus.value[currentStepIndex.value] = 'finish';
          }
          currentStepIndex.value = step;
          stepStatus.value[step] = 'process';
        }
      }
    });

    // Listen for completion
    if (unlistenDone) unlistenDone();
    unlistenDone = await listen<string>(`run-done-${id}`, (event) => {
      isRunning.value = false;
      if (event.payload === 'done') {
        if (currentStepIndex.value > 0) stepStatus.value[currentStepIndex.value] = 'finish';
      } else {
        if (currentStepIndex.value > 0) stepStatus.value[currentStepIndex.value] = 'error';
      }
    });

    // Fire and forget — streaming happens via events
    invoke('run_ccg_command', {
      runId: id,
      workspacePath: ws.path,
      skill,
      prompt: workflowPrompt.value
    }).catch((e: any) => {
      logs.value.push(`[错误] ${e}`);
      isRunning.value = false;
      if (currentStepIndex.value > 0) stepStatus.value[currentStepIndex.value] = 'error';
    });

  } catch (e: any) {
    console.error('Start failed', e);
    isRunning.value = false;
  }
}

function resetWorkflow() {
  runId.value = null;
  isRunning.value = false;
  logs.value = [];
  currentStepIndex.value = 0;
  stepStatus.value = { 1: 'wait', 2: 'wait', 3: 'wait', 4: 'wait', 5: 'wait', 6: 'wait' };
}

onUnmounted(() => {
  if (unlistenLog) unlistenLog();
  if (unlistenDone) unlistenDone();
});
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%; gap: 16px;">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning">
      请先在左侧边栏选择或创建一个项目 (Workspace) 以启动工作流。
    </n-alert>

    <!-- Hero / Input Section -->
    <n-card style="border-radius: 12px; background: linear-gradient(135deg, #18181c 0%, #1a2332 100%); flex-shrink: 0;">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
        <n-gradient-text type="info" :size="22" style="font-weight: 800;">
          vibeCoding Command Center
        </n-gradient-text>
        <n-tag v-if="workspaceStore.currentWorkspace" type="success" size="small" round>
          <template #icon><n-icon><FolderOpenOutline/></n-icon></template>
          {{ workspaceStore.currentWorkspace.name }}
        </n-tag>
      </div>

      <n-input
        v-model:value="workflowPrompt"
        type="textarea"
        placeholder="输入你的需求，例如：帮我写一个基于 Vue3 的 Todo 列表页面，支持本地存储..."
        :autosize="{ minRows: 2, maxRows: 5 }"
        style="border-radius: 8px; font-size: 13px; background: rgba(0, 0, 0, 0.2); margin-bottom: 16px;"
        :disabled="isRunning"
      />

      <div v-if="!isRunning" style="display: flex; flex-wrap: wrap; gap: 10px;">
        <n-button type="info" size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim() || !workspaceStore.currentWorkspace" @click="startWorkflow('ccg:workflow')">
          <template #icon><n-icon><RocketOutline /></n-icon></template> 完整工作流
        </n-button>
        <n-button type="success" secondary size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim() || !workspaceStore.currentWorkspace" @click="startWorkflow('ccg:frontend')">
          极速前端 (Gemini)
        </n-button>
        <n-button type="warning" secondary size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim() || !workspaceStore.currentWorkspace" @click="startWorkflow('ccg:backend')">
          极速后端 (Codex)
        </n-button>
      </div>
      <div v-else style="display: flex; align-items: center; gap: 12px;">
        <n-spin :size="16" />
        <span style="color: #888; font-size: 13px;">Claude 正在执行工作流，请在日志区查看实时输出...</span>
        <n-button type="error" ghost size="medium" style="border-radius: 8px; margin-left: auto;" @click="resetWorkflow">
          中止并重置
        </n-button>
      </div>
    </n-card>

    <!-- Step Indicator + Logs (always visible once runId set) -->
    <div v-if="runId" style="display: flex; flex: 1; min-height: 0; gap: 16px;">

      <!-- Left: Step Indicator -->
      <n-card style="width: 200px; border-radius: 12px; flex-shrink: 0;" :content-style="{ padding: '16px' }">
        <n-steps vertical :current="currentStepIndex" size="small">
          <n-step
            v-for="(step, i) in workflowSteps"
            :key="i"
            :title="step.title"
            :description="step.desc"
            :status="stepStatus[i + 1]"
          />
        </n-steps>
      </n-card>

      <!-- Right: Logs -->
      <n-card style="flex: 1; border-radius: 12px; background: #0d1117; border: 1px solid #30363d; display: flex; flex-direction: column; min-width: 0;" :content-style="{ padding: '12px', display: 'flex', flexDirection: 'column', flex: '1', minHeight: '0' }">
        <div style="font-size: 11px; color: #484f58; margin-bottom: 8px; font-family: monospace;">实时日志输出 — {{ workspaceStore.currentWorkspace?.path }}</div>
        <n-scrollbar style="flex: 1;">
          <div v-for="(log, idx) in logs" :key="idx"
            :style="{
              fontFamily: 'Cascadia Code, Consolas, monospace',
              fontSize: '12px',
              lineHeight: '1.6',
              whiteSpace: 'pre-wrap',
              color: log.startsWith('[stderr]') ? '#f85149' : log.includes('[模式') ? '#58a6ff' : '#c9d1d9',
              padding: '1px 0'
            }">
            {{ log }}
          </div>
          <div v-if="logs.length === 0" style="color: #484f58; text-align: center; font-style: italic; padding: 20px;">等待 Claude 输出...</div>
        </n-scrollbar>
      </n-card>
    </div>
  </div>
</template>

<style scoped>
</style>
