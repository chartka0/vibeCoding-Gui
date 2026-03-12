<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NSteps, NStep, NGrid, NGi,
  NGradientText, NTag, NStatistic, NAlert
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
  { title: '需求收集', desc: '输入文字描述或上传设计草图', icon: DocumentTextOutline },
  { title: '上下文对齐', desc: 'AI 澄清追问，消除歧义', icon: ChatbubblesOutline },
  { title: '工程文档生成', desc: '生成 UI_Spec / Logic_Spec / Test_Spec', icon: CodeSlashOutline },
  { title: '并行编码', desc: 'Codex(后端) + Gemini(前端) 同时开工', icon: FlashOutline },
  { title: '审查修复', desc: 'Claude 交叉审查，自愈修复', icon: GitBranchOutline },
  { title: '交付存档', desc: '自动生成 Commit，版本沉淀', icon: CheckmarkDoneOutline },
];

async function startWorkflow() {
  const currentId = workspaceStore.currentWorkspaceId;
  if (!currentId) return;

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
      <div style="text-align: center; padding: 30px 0 20px;">
        <n-gradient-text type="info" :size="32" style="font-weight: 800;">
          vibeCoding Workflow
        </n-gradient-text>
        <p style="color: #888; margin: 10px 0 24px; font-size: 14px;">
          Claude + Codex + Gemini 三模型协作，从需求到交付的全自动化闭环
        </p>
        <div v-if="workspaceStore.currentWorkspace" style="margin-bottom: 20px;">
          <n-tag type="success" size="medium" round>
            <template #icon><n-icon><FolderOpenOutline/></n-icon></template>
            当前项目: {{ workspaceStore.currentWorkspace.name }}
          </n-tag>
        </div>
        <n-button 
          type="info" 
          size="large" 
          round 
          style="padding: 0 40px; font-size: 16px; height: 48px;"
          :disabled="!workspaceStore.currentWorkspaceId"
          @click="startWorkflow"
        >
          <template #icon><n-icon :size="20"><RocketOutline /></n-icon></template>
          启动完整工作流
        </n-button>
      </div>

      <!-- Stats -->
      <div style="display: flex; justify-content: center; gap: 60px; padding: 16px 0 8px; border-top: 1px solid #2a2a2e; margin-top: 16px;">
        <n-statistic label="模型引擎" value="3" style="text-align: center;" />
        <n-statistic label="工作阶段" value="6" style="text-align: center;" />
        <n-statistic label="斜杠命令" value="26" style="text-align: center;" />
        <n-statistic label="专家提示词" value="13" style="text-align: center;" />
      </div>
    </n-card>

    <!-- Workflow Steps -->
    <n-card title="标准工作流 (The vibeCoding Flow)" size="medium" style="border-radius: 12px; margin-bottom: 20px;">
      <template #header-extra>
        <n-tag type="info" size="small" :bordered="false">6 Phases</n-tag>
      </template>
      <n-steps :current="currentStep" style="padding: 16px 0;">
        <n-step
          v-for="(step, i) in workflowSteps"
          :key="i"
          :title="step.title"
          :description="step.desc"
        />
      </n-steps>
    </n-card>

    <!-- Live Execution Logs -->
    <n-card v-if="logs.length > 0" title="执行日志 (Execution Logs)" size="medium" style="border-radius: 12px; margin-bottom: 20px;">
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
  </div>
</template>
