<script setup lang="ts">
import { ref, computed, nextTick, onUnmounted } from "vue";
import {
  NCard, NButton, NIcon,
  NGradientText, NTag, NAlert, NInput, NSpin, NScrollbar,
  useMessage
} from "naive-ui";
import {
  FlashOutline, DocumentTextOutline, CodeSlashOutline,
  CheckmarkDoneOutline, SearchOutline, FolderOpenOutline,
  StopCircleOutline, EyeOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

// ── Constants ──
const CODEX_STEPS = [
  { id: 'read-plan', title: '读取计划', desc: '解析 plan 文件', icon: DocumentTextOutline, modeKey: '准备' },
  { id: 'execute', title: 'Codex 执行', desc: 'MCP搜索 + 代码实现', icon: FlashOutline, modeKey: '执行' },
  { id: 'review', title: 'Claude 审核', desc: '轻量验证变更', icon: SearchOutline, modeKey: '审核' },
  { id: 'audit', title: '多模型审查', desc: 'Codex + Gemini 交叉审查', icon: CodeSlashOutline, modeKey: '审核' },
  { id: 'deliver', title: '交付', desc: '报告结果与建议', icon: CheckmarkDoneOutline, modeKey: '交付' },
] as const;

// Mode markers from Claude output → step index (0-based)
const MODE_TO_STEP: Record<string, number> = {
  '准备': 0,
  '执行': 1,
  '审核': 2, // Phase 2 & 3 both use 审核, we advance via sequence
  '追加': 2, // Phase 2.5 stays on review
  '交付': 4,
};

// ── State ──
const message = useMessage();
const workspaceStore = useWorkspaceStore();
const planFilePath = ref('');
const planPreview = ref('');
const showPreview = ref(false);
const runId = ref<string | null>(null);
const isRunning = ref(false);
const isDone = ref(false);
const isFailed = ref(false);
const activeStepIndex = ref(-1); // -1 = not started
const output = ref('');
const startTime = ref<number | null>(null);
const now = ref(Date.now());
let durationTimer: ReturnType<typeof setInterval> | null = null;
let unlisteners: UnlistenFn[] = [];
// Track highest step reached so "审核" can distinguish Phase 2 vs Phase 3
let highestStep = -1;

const elapsed = computed(() => {
  if (!startTime.value) return '';
  const secs = Math.floor((now.value - startTime.value) / 1000);
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
});

const canStart = computed(() =>
  workspaceStore.currentWorkspaceId && planFilePath.value.trim() && !isRunning.value
);

// ── File Picker ──
async function browsePlanFile() {
  const ws = workspaceStore.currentWorkspace;
  const defaultPath = ws?.path ? `${ws.path}/.claude/plan` : undefined;
  const selected = await open({
    title: '选择计划文件 (.md)',
    defaultPath,
    filters: [{ name: 'Markdown', extensions: ['md'] }],
    multiple: false,
    directory: false,
  });
  if (selected && typeof selected === 'string') {
    planFilePath.value = selected;
    await loadPreview();
  }
}

async function loadPreview() {
  if (!planFilePath.value.trim()) {
    planPreview.value = '';
    return;
  }
  try {
    // Use the Rust backend to read the file (workspace-relative or absolute)
    const ws = workspaceStore.currentWorkspace;
    const path = planFilePath.value;
    // Try reading via invoke or fallback
    const content: string = await invoke('read_text_file', { path, basePath: ws?.path || '' });
    planPreview.value = content;
  } catch {
    planPreview.value = '(无法读取文件，请确认路径正确)';
  }
}

function togglePreview() {
  if (!showPreview.value && !planPreview.value) {
    loadPreview();
  }
  showPreview.value = !showPreview.value;
}

// ── Phase Detection ──
function detectPhase(line: string) {
  // Match [模式：xxx] pattern in stream-json text output
  const match = line.match(/\[模式[：:]\s*(.+?)\s*\]/);
  if (!match) return;
  const mode = match[1];
  let targetStep = MODE_TO_STEP[mode];
  if (targetStep === undefined) return;

  // "审核" appears twice (Phase 2 = step 2, Phase 3 = step 3)
  // If we've already reached step 2 and see 审核 again, advance to step 3
  if (mode === '审核' && highestStep >= 2) {
    targetStep = 3;
  }

  if (targetStep > highestStep) {
    // Mark previous steps as finished
    for (let i = 0; i <= highestStep; i++) {
      stepStatuses.value[i] = 'finish';
    }
    highestStep = targetStep;
    activeStepIndex.value = targetStep;
    stepStatuses.value[targetStep] = 'process';
  }
}

// ── Step statuses ──
const stepStatuses = ref<Record<number, 'wait' | 'process' | 'finish' | 'error'>>(
  Object.fromEntries(CODEX_STEPS.map((_, i) => [i, 'wait' as const]))
);

function resetState() {
  output.value = '';
  activeStepIndex.value = -1;
  highestStep = -1;
  isDone.value = false;
  isFailed.value = false;
  showPreview.value = false;
  stepStatuses.value = Object.fromEntries(CODEX_STEPS.map((_, i) => [i, 'wait' as const]));
}

// ── Execution ──
async function startCodexExec() {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !planFilePath.value.trim()) return;

  resetState();
  isRunning.value = true;
  startTime.value = Date.now();
  durationTimer = setInterval(() => { now.value = Date.now(); }, 1000);

  try {
    // 1. Create workflow run record
    const id: string = await invoke('start_workflow_run', {
      workspaceId: ws.id,
      mode: 'codex-exec',
      prompt: planFilePath.value,
    });
    runId.value = id;

    // 2. Setup event listeners
    await setupListeners(id);

    // 3. Launch single ccg:codex-exec command
    await invoke('run_ccg_command', {
      runId: id,
      workspacePath: ws.path,
      skill: 'ccg:codex-exec',
      prompt: planFilePath.value,
    });
    // run_ccg_command resolves when process exits; done event handles the rest
  } catch (e: any) {
    console.error('Codex exec failed', e);
    message.error(`执行失败: ${e}`);
    isFailed.value = true;
    isRunning.value = false;
    stopTimer();
  }
}

async function setupListeners(id: string) {
  cleanupListeners();

  // Stream log lines
  const logUn = await listen<string>(`step-log-${id}`, (event) => {
    const line = event.payload;
    output.value += line + '\n';
    detectPhase(line);
    nextTick(scrollToBottom);
  });
  unlisteners.push(logUn);

  // Process done signal
  const doneUn = await listen<string>(`run-done-${id}`, (event) => {
    isRunning.value = false;
    stopTimer();
    if (event.payload === 'done') {
      isDone.value = true;
      // Mark all remaining steps as finished
      for (let i = 0; i < CODEX_STEPS.length; i++) {
        if (stepStatuses.value[i] !== 'error') {
          stepStatuses.value[i] = 'finish';
        }
      }
      activeStepIndex.value = CODEX_STEPS.length - 1;
    } else {
      isFailed.value = true;
      // Mark current step as error
      if (activeStepIndex.value >= 0) {
        stepStatuses.value[activeStepIndex.value] = 'error';
      }
    }
  });
  unlisteners.push(doneUn);
}

function stopExec() {
  if (runId.value) {
    // Cancel via the orchestrator's cancel command
    invoke('cancel_workflow_step', { runId: runId.value, stepIndex: 0 }).catch(() => {});
  }
  isRunning.value = false;
  isFailed.value = true;
  stopTimer();
  if (activeStepIndex.value >= 0) {
    stepStatuses.value[activeStepIndex.value] = 'error';
  }
}

function resetAll() {
  cleanupListeners();
  resetState();
  runId.value = null;
  isRunning.value = false;
  startTime.value = null;
  stopTimer();
}

// ── Utilities ──
function stopTimer() {
  if (durationTimer) {
    clearInterval(durationTimer);
    durationTimer = null;
  }
}

function cleanupListeners() {
  for (const fn of unlisteners) fn();
  unlisteners = [];
}

const outputScrollRef = ref<InstanceType<typeof NScrollbar> | null>(null);
function scrollToBottom() {
  outputScrollRef.value?.scrollTo({ top: 999999, behavior: 'smooth' });
}

function stepColor(status: string) {
  switch (status) {
    case 'process': return '#8a2be2';
    case 'finish': return '#3fb950';
    case 'error': return '#f85149';
    default: return '#484f58';
  }
}

onUnmounted(() => {
  cleanupListeners();
  stopTimer();
});
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%; gap: 16px;">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning">
      请先在左侧边栏选择或创建一个项目 (Workspace)。
    </n-alert>

    <!-- Hero / Input Section -->
    <n-card style="border-radius: 12px; background: linear-gradient(135deg, #18181c 0%, #221a2e 100%); flex-shrink: 0;">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
        <n-gradient-text type="warning" :size="22" style="font-weight: 800;">
          Codex 全权执行
        </n-gradient-text>
        <div style="display: flex; align-items: center; gap: 8px;">
          <span v-if="isRunning" style="color: #8a2be2; font-size: 13px; font-family: 'Cascadia Code', monospace;">
            {{ elapsed }}
          </span>
          <n-tag v-if="workspaceStore.currentWorkspace" type="success" size="small" round>
            <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
            {{ workspaceStore.currentWorkspace.name }}
          </n-tag>
        </div>
      </div>

      <p style="color: #888; font-size: 12px; margin: 0 0 16px;">
        导入计划文件，由 Codex 一把梭全权执行。Claude 仅做编排与审核，Token 消耗极低。
      </p>

      <!-- Plan File Input -->
      <div style="margin-bottom: 16px;">
        <div style="margin-bottom: 6px; font-weight: 600; color: #ccc; font-size: 13px;">
          计划文件 (Plan File):
        </div>
        <div style="display: flex; gap: 8px;">
          <n-input
            v-model:value="planFilePath"
            placeholder=".claude/plan/user-auth.md"
            style="border-radius: 8px; font-size: 13px; flex: 1;"
            :disabled="isRunning"
            @blur="loadPreview"
          />
          <n-button
            quaternary
            style="border-radius: 8px;"
            :disabled="isRunning"
            @click="browsePlanFile"
          >
            <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
          </n-button>
          <n-button
            quaternary
            style="border-radius: 8px;"
            :disabled="!planFilePath.trim()"
            @click="togglePreview"
          >
            <template #icon><n-icon><EyeOutline /></n-icon></template>
          </n-button>
        </div>
      </div>

      <!-- Plan Preview -->
      <div v-if="showPreview && planPreview" style="margin-bottom: 16px;">
        <n-scrollbar style="max-height: 200px; background: rgba(0,0,0,0.3); border-radius: 8px; padding: 12px;">
          <pre style="margin: 0; white-space: pre-wrap; font-family: 'Cascadia Code', monospace; font-size: 12px; color: #8b949e; line-height: 1.5;">{{ planPreview }}</pre>
        </n-scrollbar>
      </div>

      <!-- Action Buttons -->
      <div v-if="!runId" style="display: flex; gap: 12px;">
        <n-button
          color="#8a2be2"
          size="medium"
          style="border-radius: 8px; font-weight: 600;"
          :disabled="!canStart"
          @click="startCodexExec"
        >
          <template #icon><n-icon><FlashOutline /></n-icon></template>
          启动 Codex 全权执行
        </n-button>
      </div>
      <div v-else style="display: flex; gap: 12px; align-items: center;">
        <n-button
          v-if="isRunning"
          type="error"
          ghost
          size="medium"
          style="border-radius: 8px;"
          @click="stopExec"
        >
          <template #icon><n-icon><StopCircleOutline /></n-icon></template>
          中止执行
        </n-button>
        <n-button
          v-if="isDone || isFailed"
          type="warning"
          ghost
          size="medium"
          style="border-radius: 8px;"
          @click="resetAll"
        >
          重置
        </n-button>
        <n-tag v-if="isDone" type="success" size="small" round>执行完成</n-tag>
        <n-tag v-if="isFailed" type="error" size="small" round>执行失败</n-tag>
      </div>
    </n-card>

    <!-- Orchestrator View -->
    <div v-if="runId" style="display: flex; flex: 1; min-height: 0; gap: 16px;">

      <!-- Left: Step Indicator -->
      <n-card style="width: 220px; border-radius: 12px; flex-shrink: 0;" :content-style="{ padding: '16px' }">
        <div style="margin-bottom: 12px; font-size: 12px; color: #666; font-weight: 600;">执行阶段</div>
        <div v-for="(step, i) in CODEX_STEPS" :key="step.id" style="display: flex; align-items: flex-start; gap: 10px; margin-bottom: 16px;">
          <!-- Status dot -->
          <div style="margin-top: 2px;">
            <n-spin v-if="stepStatuses[i] === 'process'" :size="14" stroke="#8a2be2" />
            <div
              v-else
              :style="{
                width: '14px', height: '14px', borderRadius: '50%',
                background: stepColor(stepStatuses[i]),
                transition: 'background 0.3s',
              }"
            />
          </div>
          <!-- Text -->
          <div style="flex: 1; min-width: 0;">
            <div :style="{
              fontSize: '13px', fontWeight: activeStepIndex === i ? 700 : 400,
              color: activeStepIndex === i ? '#e6e6e6' : '#888',
              transition: 'all 0.3s',
            }">
              {{ step.title }}
            </div>
            <div style="font-size: 11px; color: #555; margin-top: 2px;">{{ step.desc }}</div>
          </div>
        </div>
      </n-card>

      <!-- Right: Output Stream -->
      <n-card style="flex: 1; border-radius: 12px; display: flex; flex-direction: column; min-width: 0;" :content-style="{ padding: '0', flex: '1', display: 'flex', flexDirection: 'column' }">
        <template #header>
          <div style="display: flex; align-items: center; gap: 8px; padding: 4px 0;">
            <n-spin v-if="isRunning" :size="14" stroke="#8a2be2" />
            <span style="font-size: 13px; color: #ccc;">
              {{ isRunning ? (activeStepIndex >= 0 ? CODEX_STEPS[activeStepIndex].title + ' 执行中...' : '启动中...') : (isDone ? '执行完成' : isFailed ? '执行失败' : '') }}
            </span>
            <span v-if="startTime" style="font-size: 12px; color: #555; margin-left: auto; font-family: 'Cascadia Code', monospace;">
              {{ elapsed }}
            </span>
          </div>
        </template>
        <n-scrollbar ref="outputScrollRef" style="flex: 1; padding: 12px;">
          <pre style="margin: 0; white-space: pre-wrap; font-family: 'Cascadia Code', monospace; font-size: 12px; color: #8b949e; line-height: 1.6;">{{ output || '等待输出...' }}</pre>
        </n-scrollbar>
      </n-card>
    </div>
  </div>
</template>
