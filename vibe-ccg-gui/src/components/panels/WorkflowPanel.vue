<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NSteps, NStep, NGrid, NGi,
  NGradientText, NTag, NStatistic, NAlert, NModal, NInput
} from "naive-ui";
import {
  RocketOutline, DocumentTextOutline, ChatbubblesOutline,
  CodeSlashOutline, GitBranchOutline, CheckmarkDoneOutline,
  FlashOutline, BrowsersOutline, FolderOpenOutline, RefreshOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const currentStep = ref(0);
const workspaceStore = useWorkspaceStore();

const isRunning = ref(false);
const logs = ref<string[]>([]);
let unlistenLog: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;

const workflowSteps = [
  { title: '[模式：研究]', desc: '需求收集、上下文与分析', icon: DocumentTextOutline },
  { title: '[模式：构思]', desc: '双模型并行分析可行性', icon: ChatbubblesOutline },
  { title: '[模式：计划]', desc: '多模型产出前后端架构规划', icon: CodeSlashOutline },
  { title: '[模式：执行]', desc: '严格按批准计划编码实施', icon: FlashOutline },
  { title: '[模式：优化]', desc: '多模型并行审查安全与设计', icon: GitBranchOutline },
  { title: '[模式：评审]', desc: '最终评估与测试闭环交付', icon: CheckmarkDoneOutline },
];

const showWizard = ref(false);
const workflowPrompt = ref('');

function openWizard() {
  showWizard.value = true;
  workflowPrompt.value = '';
}

async function confirmStartWorkflow() {
  const currentId = workspaceStore.currentWorkspaceId;
  if (!currentId) return;

  showWizard.value = false;
  isRunning.value = true;
  logs.value = [];
  logs.value.push("初始化工作流环境...");
  currentStep.value = 1;

  try {
    // Setup event listeners for this specific workspace execution
    unlistenLog = await listen<{ line: string }>(`workflow-log-${currentId}`, (event) => {
      logs.value.push(event.payload.line);
      // Rough simulation of progress steps based on log volume or content
      if (logs.value.length > 5 && currentStep.value < 3) currentStep.value = 3;
    });

    unlistenDone = await listen(`workflow-done-${currentId}`, () => {
      isRunning.value = false;
      currentStep.value = 6;
      cleanupListeners();
    });

    // Invoke Rust process spawn
    await invoke('start_workflow', { workspaceId: currentId });
    logs.value.push("后端子进程已分发...");

  } catch (err) {
    console.error("Workflow trigger failed:", err);
    logs.value.push(`[ERROR] 工作流启动失败: ${err}`);
    isRunning.value = false;
    cleanupListeners();
  }
}

function cleanupListeners() {
  if (unlistenLog) { unlistenLog(); unlistenLog = null; }
  if (unlistenDone) { unlistenDone(); unlistenDone = null; }
}
</script>

<template>
  <div>
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning" style="margin-bottom: 20px;">
      请先在左侧边栏选择或创建一个项目 (Workspace) 以启动工作流。
    </n-alert>
    
    <!-- Hero Section -->
    <n-card style="border-radius: 12px; margin-bottom: 20px; background: linear-gradient(135deg, #18181c 0%, #1a2332 100%);">
      <div style="text-align: center; padding: 20px 0 16px;">
        <n-gradient-text type="info" :size="28" style="font-weight: 800;">
          vibeCoding Workflow
        </n-gradient-text>
        <p style="color: #888; margin: 8px 0 20px; font-size: 13px;">
          Claude + Codex + Gemini 三模型协作，从需求到交付的全自动化闭环
        </p>
        <div v-if="workspaceStore.currentWorkspace" style="margin-bottom: 16px;">
          <n-tag type="success" size="small" round>
            <template #icon><n-icon><FolderOpenOutline/></n-icon></template>
            当前项目: {{ workspaceStore.currentWorkspace.name }}
          </n-tag>
        </div>
        <n-button 
          type="info" 
          size="medium" 
          round 
          style="padding: 0 32px; font-size: 15px; height: 40px;"
          :disabled="!workspaceStore.currentWorkspaceId"
          @click="openWizard"
        >
          <template #icon><n-icon :size="18"><RocketOutline /></n-icon></template>
          启动完整工作流
        </n-button>
      </div>

      <!-- Stats -->
      <div style="display: flex; justify-content: center; gap: 40px; padding: 12px 0 4px; border-top: 1px solid #2a2a2e; margin-top: 12px;">
        <n-statistic label="模型引擎" value="3" style="text-align: center;">
          <template #label><span style="font-size: 12px">模型引擎</span></template>
        </n-statistic>
        <n-statistic label="工作阶段" value="6" style="text-align: center;">
          <template #label><span style="font-size: 12px">工作阶段</span></template>
        </n-statistic>
        <n-statistic label="斜杠命令" value="26" style="text-align: center;">
          <template #label><span style="font-size: 12px">斜杠命令</span></template>
        </n-statistic>
        <n-statistic label="专家提示词" value="13" style="text-align: center;">
          <template #label><span style="font-size: 12px">专家提示词</span></template>
        </n-statistic>
      </div>
    </n-card>

    <!-- Workflow Steps -->
    <n-card title="标准工作流 (The vibeCoding Flow)" size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <template #header-extra>
        <n-tag type="info" size="small" :bordered="false">6 Phases</n-tag>
      </template>
      <n-steps :current="currentStep" style="padding: 12px 0;" size="small">
        <n-step
          v-for="(step, i) in workflowSteps"
          :key="i"
          :title="step.title"
          :description="step.desc"
        />
      </n-steps>
    </n-card>

    <!-- Live Execution Logs -->
    <n-card v-if="logs.length > 0" title="执行日志 (Execution Logs)" size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <div style="background: #000; border-radius: 8px; padding: 16px; font-family: 'Fira Code', 'Consolas', monospace; font-size: 13px; color: #a9b7c6; max-height: 400px; overflow-y: auto;">
        <div v-for="(log, idx) in logs" :key="idx" style="margin-bottom: 4px; line-height: 1.5;">
          <span style="color: #629755; margin-right: 8px;">></span> {{ log }}
        </div>
        <div v-if="isRunning" style="display: flex; align-items: center; gap: 8px; margin-top: 12px; color: #63e2b7;">
          <n-icon class="spin"><RefreshOutline /></n-icon> 正在运行后端任务...
        </div>
      </div>
    </n-card>

    <!-- Quick Actions -->
    <n-grid :x-gap="16" :y-gap="16" :cols="3">
      <n-gi>
        <n-card hoverable style="border-radius: 12px; cursor: pointer;">
          <div style="display: flex; align-items: center; gap: 12px;">
            <n-icon :size="28" color="#63e2b7"><BrowsersOutline /></n-icon>
            <div>
              <div style="font-weight: 600; font-size: 14px;">极速前端</div>
              <div style="color: #888; font-size: 12px; margin-top: 2px;">Gemini 生成前端 UI 代码</div>
            </div>
          </div>
          <div style="margin-top: 12px;">
            <n-tag size="small" type="success" :bordered="false">/ccg:frontend</n-tag>
          </div>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card hoverable style="border-radius: 12px; cursor: pointer;">
          <div style="display: flex; align-items: center; gap: 12px;">
            <n-icon :size="28" color="#f2c97d"><CodeSlashOutline /></n-icon>
            <div>
              <div style="font-weight: 600; font-size: 14px;">极速后端</div>
              <div style="color: #888; font-size: 12px; margin-top: 2px;">Codex 编写核心逻辑</div>
            </div>
          </div>
          <div style="margin-top: 12px;">
            <n-tag size="small" type="warning" :bordered="false">/ccg:backend</n-tag>
          </div>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card hoverable style="border-radius: 12px; cursor: pointer;">
          <div style="display: flex; align-items: center; gap: 12px;">
            <n-icon :size="28" color="#70c0e8"><FlashOutline /></n-icon>
            <div>
              <div style="font-weight: 600; font-size: 14px;">Codex 全权执行</div>
              <div style="color: #888; font-size: 12px; margin-top: 2px;">极低 Token 消耗模式</div>
            </div>
          </div>
          <div style="margin-top: 12px;">
            <n-tag size="small" type="info" :bordered="false">/ccg:codex-exec</n-tag>
          </div>
        </n-card>
      </n-gi>
    </n-grid>

    <!-- Workflow Initialization Wizard Modal -->
    <n-modal v-model:show="showWizard" preset="card" title="初始化工作流 (Initialize Workflow)" style="width: 600px; border-radius: 12px;">
      <n-alert type="warning" show-icon style="margin-bottom: 20px;">
        ⚠️ <strong>架构防坑建议 (Architectural Advice)</strong><br />
        【完整工作流】不建议用于大型项目或全局大需求，因为 AI 上下文会指数级膨胀导致 Token 爆炸或幻觉问题。<br />
        对于复杂系统需求，强烈建议改用左侧的 <strong>「规划面板 (plan)」</strong> 结合 <strong>「分段执行 (execute)」</strong>！
      </n-alert>
      
      <div style="margin-bottom: 8px; font-weight: bold;">一句话需求描述 (Prompt):</div>
      <n-input
        v-model:value="workflowPrompt"
        type="textarea"
        placeholder="例如：帮我写一个基于 Vue3 的 Todo 列表页面，支持本地存储和拖拽排序..."
        :autosize="{ minRows: 4, maxRows: 8 }"
        style="margin-bottom: 24px;"
      />
      
      <div style="display: flex; justify-content: flex-end; gap: 12px;">
        <n-button @click="showWizard = false">取消 (Cancel)</n-button>
        <n-button type="info" :disabled="!workflowPrompt.trim()" @click="confirmStartWorkflow">
          <template #icon><n-icon><RocketOutline /></n-icon></template>
          确认并启动 (Confirm & Start)
        </n-button>
      </div>
    </n-modal>
  </div>
</template>
