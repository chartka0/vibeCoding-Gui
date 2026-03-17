<script setup lang="ts">
import { ref, computed, nextTick, onUnmounted } from "vue";
import {
  NCard, NButton, NIcon, NInput, NSpin, NScrollbar, NTag,
  NAlert, NGradientText, useMessage
} from "naive-ui";
import {
  PlayOutline, RefreshOutline, ChevronDownOutline, ChevronUpOutline,
  FolderOpenOutline, ArrowForwardOutline, StopCircleOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { STEPS } from "./workflow-steps";

// ── Types ──
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

// ── State ──
const message = useMessage();
const workspaceStore = useWorkspaceStore();
const userPrompt = ref('');
const runId = ref<string | null>(null);
const sessionId = ref<string | null>(null);
const activeStep = ref(-1);
const waitingForUser = ref(false);
const userFeedback = ref('');
const workflowDone = ref(false);

// #4: Reactive clock for live duration display
const now = ref(Date.now());
let durationTimer: ReturnType<typeof setInterval> | null = null;

function startDurationTimer() {
  if (!durationTimer) {
    durationTimer = setInterval(() => { now.value = Date.now(); }, 1000);
  }
}
function stopDurationTimer() {
  if (durationTimer) {
    clearInterval(durationTimer);
    durationTimer = null;
  }
}

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

// ── Helpers ──
const STATUS_COLORS: Record<string, string> = {
  success: '#63e2b7',
  failed: '#f85149',
  running: '#58a6ff',
  pending: '#333',
};

function stepColor(status: string): string {
  return STATUS_COLORS[status] || STATUS_COLORS.pending;
}

// #4: Uses reactive `now` for live updates when step is still running
function formatDuration(start: number | null, end: number | null): string {
  if (!start) return '';
  const elapsed = (end || now.value) - start;
  if (elapsed < 1000) return `${elapsed}ms`;
  const secs = Math.floor(elapsed / 1000);
  if (secs < 60) return `${secs}s`;
  return `${Math.floor(secs / 60)}m ${secs % 60}s`;
}

// #1: Scroll the active output area to bottom
function scrollOutputToBottom() {
  nextTick(() => {
    const el = document.querySelector('.wf-output-scroll .n-scrollbar-container');
    if (el) el.scrollTop = el.scrollHeight;
  });
}

// ── Listeners ──
let unlisteners: UnlistenFn[] = [];

async function listenStep(rId: string, stepIdx: number) {
  for (const fn of unlisteners) fn();
  unlisteners = [];

  const logUn = await listen<string>(`wf-step-log-${rId}-${stepIdx}`, (event) => {
    stepStates.value[stepIdx].output += event.payload;
    scrollOutputToBottom();
  });
  unlisteners.push(logUn);

  const doneUn = await listen<StepDonePayload>(`wf-step-done-${rId}`, (event) => {
    const p = event.payload;
    if (p.step_index !== stepIdx) return;

    const s = stepStates.value[stepIdx];
    s.endTime = Date.now();
    stopDurationTimer();

    if (p.status === 'success') {
      s.status = 'success';
      if (p.session_id) sessionId.value = p.session_id;
      if (stepIdx === STEPS.length - 1) {
        workflowDone.value = true;
      } else {
        waitingForUser.value = true;
        userFeedback.value = '';
      }
    } else {
      s.status = 'failed';
      waitingForUser.value = true;
    }
  });
  unlisteners.push(doneUn);
}

// ── Actions ──
async function startWorkflow(mode: 'workflow' | 'frontend' | 'backend' = 'workflow') {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !userPrompt.value.trim()) return;

  workflowDone.value = false;
  waitingForUser.value = false;
  userFeedback.value = '';
  sessionId.value = crypto.randomUUID();
  stepStates.value = STEPS.map(() => ({
    status: 'pending' as const, output: '', startTime: null, endTime: null, expanded: false, retryCount: 0,
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
    // #6: Show error to user instead of only console.error
    console.error('Start failed', e);
    message.error(`工作流启动失败: ${e}`);
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

  stepStates.value.forEach((st, i) => { if (i !== index) st.expanded = false; });

  startDurationTimer();
  const prompt = STEPS[index].promptBuilder(userPrompt.value, userFeedback.value);
  await listenStep(runId.value, index);

  // #3: await invoke so failures are caught; clean up listeners on error
  try {
    await invoke('run_workflow_step', {
      runId: runId.value,
      stepIndex: index,
      stepName: STEPS[index].name,
      workspacePath: ws.path,
      prompt,
      sessionId: sessionId.value,
    });
  } catch (e: any) {
    for (const fn of unlisteners) fn();
    unlisteners = [];
    stopDurationTimer();
    s.status = 'failed';
    s.output += `\n[错误] ${e}`;
    s.endTime = Date.now();
    waitingForUser.value = true;
  }
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

// #5: Cancel backend process + reset frontend state
async function resetWorkflow() {
  if (runId.value && activeStep.value >= 0 && isRunning.value) {
    try {
      await invoke('cancel_workflow_step', {
        runId: runId.value,
        stepIndex: activeStep.value,
      });
    } catch (_e) {
      // Process may have already exited — safe to ignore
    }
  }

  for (const fn of unlisteners) fn();
  unlisteners = [];
  stopDurationTimer();
  runId.value = null;
  sessionId.value = null;
  activeStep.value = -1;
  waitingForUser.value = false;
  workflowDone.value = false;
  userFeedback.value = '';
  stepStates.value = STEPS.map(() => ({
    status: 'pending' as const, output: '', startTime: null, endTime: null, expanded: false, retryCount: 0,
  }));
}

function toggleExpand(index: number) {
  stepStates.value[index].expanded = !stepStates.value[index].expanded;
}

onUnmounted(() => {
  for (const fn of unlisteners) fn();
  stopDurationTimer();
});
</script>

<template>
  <div class="wf-root">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning" class="wf-alert">
      请先在左侧边栏选择或创建一个项目 (Workspace) 以启动工作流。
    </n-alert>

    <!-- ─── Hero / Input ─── -->
    <n-card class="wf-hero">
      <div class="wf-hero-header">
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
        class="wf-prompt-input"
        :disabled="isRunning"
      />

      <div v-if="activeStep === -1" class="wf-actions">
        <n-button type="info" size="medium" class="wf-btn" :disabled="!canStart" @click="startWorkflow('workflow')">
          <template #icon><n-icon><PlayOutline /></n-icon></template>
          完整工作流
        </n-button>
        <n-button type="success" secondary size="medium" class="wf-btn" :disabled="!canStart" @click="startWorkflow('frontend')">
          极速前端 (Gemini)
        </n-button>
        <n-button type="warning" secondary size="medium" class="wf-btn" :disabled="!canStart" @click="startWorkflow('backend')">
          极速后端 (Codex)
        </n-button>
      </div>
      <div v-else class="wf-running-bar">
        <n-spin v-if="isRunning" :size="16" />
        <span v-if="isRunning" class="wf-running-text">
          {{ STEPS[activeStep]?.name }} 执行中...
        </span>
        <span v-else-if="workflowDone" class="wf-done-text">
          工作流已完成
        </span>
        <n-button type="error" ghost size="small" class="wf-btn wf-reset-btn" @click="resetWorkflow">
          <template #icon><n-icon><StopCircleOutline /></n-icon></template>
          重置
        </n-button>
      </div>
    </n-card>

    <!-- ─── Step Cards ─── -->
    <div v-if="runId" class="wf-steps">
      <div v-for="(step, idx) in STEPS" :key="step.id">
        <n-card
          size="small"
          :style="{
            borderRadius: '12px',
            borderLeft: `3px solid ${stepColor(stepStates[idx].status)}`,
            opacity: stepStates[idx].status === 'pending' ? 0.5 : 1,
            transition: 'all 0.3s ease',
          }"
        >
          <!-- Step Header -->
          <div class="wf-step-header" @click="stepStates[idx].output && toggleExpand(idx)">
            <n-icon :size="20" :color="stepColor(stepStates[idx].status)">
              <component :is="step.icon" />
            </n-icon>

            <div style="flex: 1; min-width: 0;">
              <div class="wf-step-title-row">
                <span class="wf-step-name">{{ idx + 1 }}. {{ step.name }}</span>
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
              <div class="wf-step-desc">{{ step.desc }}</div>
            </div>

            <span v-if="stepStates[idx].startTime" class="wf-step-duration">
              {{ formatDuration(stepStates[idx].startTime, stepStates[idx].endTime) }}
            </span>

            <n-icon v-if="stepStates[idx].output" :size="16" color="#555">
              <component :is="stepStates[idx].expanded ? ChevronUpOutline : ChevronDownOutline" />
            </n-icon>
          </div>

          <!-- Step Output (expandable) -->
          <div
            v-if="stepStates[idx].expanded && stepStates[idx].output"
            class="wf-step-output"
          >
            <!-- #1: class for scroll targeting -->
            <n-scrollbar :class="{ 'wf-output-scroll': stepStates[idx].status === 'running' }" style="max-height: 300px;">
              <pre class="wf-output-pre">{{ stepStates[idx].output }}</pre>
            </n-scrollbar>
          </div>

          <!-- ─── Decision Area ─── -->
          <div v-if="waitingForUser && idx === activeStep" class="wf-decision">
            <template v-if="stepStates[idx].status === 'success'">
              <n-input
                v-model:value="userFeedback"
                type="textarea"
                :placeholder="idx === STEPS.length - 1
                  ? '工作流已到最后一步'
                  : '输入反馈、选择方案、或留空直接继续...'"
                :autosize="{ minRows: 1, maxRows: 3 }"
                class="wf-feedback-input"
              />
              <div class="wf-actions">
                <n-button
                  v-if="idx < STEPS.length - 1"
                  type="info" size="small" class="wf-btn"
                  @click="continueToNext"
                >
                  <template #icon><n-icon><ArrowForwardOutline /></n-icon></template>
                  继续下一步
                </n-button>
                <n-button type="warning" ghost size="small" class="wf-btn" @click="retryCurrentStep">
                  <template #icon><n-icon><RefreshOutline /></n-icon></template>
                  重新执行
                </n-button>
              </div>
            </template>

            <template v-else-if="stepStates[idx].status === 'failed'">
              <n-alert type="error" class="wf-alert" :show-icon="false">
                步骤执行失败，可以重试或修改反馈后重试。
              </n-alert>
              <n-input
                v-model:value="userFeedback"
                type="textarea"
                placeholder="修改指令或补充信息（可选）..."
                :autosize="{ minRows: 1, maxRows: 3 }"
                class="wf-feedback-input"
              />
              <n-button type="warning" size="small" class="wf-btn" @click="retryCurrentStep">
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
        <n-button type="info" ghost size="small" class="wf-btn" style="margin-top: 12px;" @click="resetWorkflow">
          开始新的工作流
        </n-button>
      </n-card>
    </div>
  </div>
</template>

<style scoped>
.wf-root {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 16px;
}
.wf-hero {
  border-radius: 12px;
  background: linear-gradient(135deg, #18181c 0%, #1a2332 100%);
  flex-shrink: 0;
}
.wf-hero-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.wf-prompt-input {
  border-radius: 8px;
  font-size: 13px;
  margin-bottom: 16px;
}
.wf-actions {
  display: flex;
  gap: 10px;
}
.wf-btn {
  border-radius: 8px;
  font-weight: 600;
}
.wf-running-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}
.wf-running-text {
  color: #888;
  font-size: 13px;
}
.wf-done-text {
  color: #63e2b7;
  font-size: 13px;
  font-weight: 600;
}
.wf-reset-btn {
  margin-left: auto;
}
.wf-steps {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.wf-step-header {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  user-select: none;
}
.wf-step-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.wf-step-name {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
}
.wf-step-desc {
  font-size: 11px;
  color: #666;
  margin-top: 2px;
}
.wf-step-duration {
  font-size: 11px;
  color: #555;
  font-family: 'Cascadia Code', Consolas, monospace;
}
.wf-step-output {
  margin-top: 12px;
  border-top: 1px solid #262626;
  padding-top: 12px;
}
.wf-output-pre {
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  color: #c9d1d9;
  margin: 0;
}
.wf-decision {
  margin-top: 12px;
  border-top: 1px solid #262626;
  padding-top: 12px;
}
.wf-feedback-input {
  border-radius: 8px;
  font-size: 12px;
  margin-bottom: 10px;
}
.wf-alert {
  border-radius: 8px;
  margin-bottom: 10px;
}
</style>
