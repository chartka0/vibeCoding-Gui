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
const currentStepIndex = ref(1); // 1 to 6
const runId = ref<string | null>(null);

const isLoading = ref(false);
const logs = ref<string[]>([]);

const workflowSteps = [
  { id: 'research', title: '研究', desc: '需求收集、上下文与分析', icon: DocumentTextOutline },
  { id: 'ideation', title: '构思', desc: '双模型并行分析可行性', icon: ChatbubblesOutline },
  { id: 'planning', title: '计划', desc: '多模型产出前后端架构规划', icon: CodeSlashOutline },
  { id: 'execution', title: '执行', desc: '严格按批准计划编码实施', icon: FlashOutline },
  { id: 'optimization', title: '优化', desc: '多模型并行审查安全与设计', icon: GitBranchOutline },
  { id: 'review', title: '评审', desc: '最终评估与测试闭环交付', icon: CheckmarkDoneOutline },
];

const stepStatus = ref<Record<number, 'wait' | 'process' | 'finish' | 'error'>>({
  1: 'wait', 2: 'wait', 3: 'wait', 4: 'wait', 5: 'wait', 6: 'wait'
});

const stepResults = ref<Record<number, string>>({});

let unlistenLog: UnlistenFn | null = null;
let unlistenUpdate: UnlistenFn | null = null;

async function setupListeners(id: string) {
  if (unlistenLog) unlistenLog();
  if (unlistenUpdate) unlistenUpdate();

  unlistenLog = await listen<string>(`step-log-${id}`, (event) => {
    logs.value.push(event.payload);
  });

  unlistenUpdate = await listen<any>(`step-update-${id}`, (event) => {
    const data = event.payload;
    if (data.status === 'running') stepStatus.value[data.step_index] = 'process';
    else if (data.status === 'success') stepStatus.value[data.step_index] = 'finish';
    else if (data.status === 'failed') stepStatus.value[data.step_index] = 'error';
  });
}

async function startWorkflow(mode: 'full' | 'frontend' | 'backend') {
  const ws = workspaceStore.currentWorkspace;
  if (!ws || !workflowPrompt.value.trim()) return;

  isLoading.value = true;
  logs.value = [];
  stepStatus.value = { 1: 'wait', 2: 'wait', 3: 'wait', 4: 'wait', 5: 'wait', 6: 'wait' };
  currentStepIndex.value = 1;

  try {
    const id: string = await invoke('start_workflow_run', {
      workspaceId: ws.id,
      mode,
      prompt: workflowPrompt.value
    });
    runId.value = id;
    await setupListeners(id);
    await executeCurrentStep();
  } catch (e: any) {
    console.error("Start failed", e);
  } finally {
    isLoading.value = false;
  }
}

async function executeCurrentStep() {
  if (!runId.value) return;
  const ws = workspaceStore.currentWorkspace;
  if (!ws) return;

  const idx = currentStepIndex.value;
  const stepDef = workflowSteps[idx - 1];
  isLoading.value = true;
  stepStatus.value[idx] = 'process';

  try {
    const result: any = await invoke('execute_step', {
      runId: runId.value,
      stepIndex: idx,
      stepName: stepDef.id,
      workspacePath: ws.path,
      prompt: workflowPrompt.value, // In reality, we'd pass enhanced prompts here
      sessionId: null // TODO: Pass session ID from previous steps
    });

    stepStatus.value[idx] = result.status === 'success' ? 'finish' : 'error';
    if (result.output_text) {
      stepResults.value[idx] = result.output_text;
    }
  } catch (e: any) {
    console.error(`Step ${idx} failed`, e);
    stepStatus.value[idx] = 'error';
  } finally {
    isLoading.value = false;
  }
}

async function nextStep() {
  if (currentStepIndex.value < 6) {
    currentStepIndex.value++;
    // Auto-start the next step immediately for now. We can add manual confirmation later.
    await executeCurrentStep(); 
  }
}

async function retryStep() {
  logs.value = [...logs.value, `[Sys] Retrying Step ${currentStepIndex.value}...`];
  await executeCurrentStep();
}

onUnmounted(() => {
  if (unlistenLog) unlistenLog();
  if (unlistenUpdate) unlistenUpdate();
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
        :disabled="runId !== null"
      />

      <div v-if="!runId" style="display: flex; flex-wrap: wrap; gap: 10px;">
        <n-button type="info" size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim()" @click="startWorkflow('full')">
          <template #icon><n-icon><RocketOutline /></n-icon></template> 完整工作流
        </n-button>
        <n-button type="success" secondary size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim()" @click="startWorkflow('frontend')">
          极速前端 (Gemini)
        </n-button>
        <n-button type="warning" secondary size="medium" style="border-radius: 8px; font-weight: 600;" :disabled="!workflowPrompt.trim()" @click="startWorkflow('backend')">
          极速后端 (Codex)
        </n-button>
      </div>
      <div v-else>
        <n-button type="error" ghost size="medium" style="border-radius: 8px;" @click="runId = null">
          中止并重置
        </n-button>
      </div>
    </n-card>

    <!-- Orchestrator View (Shown only when running) -->
    <div v-if="runId" style="display: flex; flex: 1; min-height: 0; gap: 16px;">
      
      <!-- Left: Step Indicator -->
      <n-card style="width: 240px; border-radius: 12px; flex-shrink: 0;" :content-style="{ padding: '16px' }">
        <n-steps vertical :current="currentStepIndex" :status="stepStatus[currentStepIndex]">
          <n-step
            v-for="(step, i) in workflowSteps"
            :key="i"
            :title="step.title"
            :description="step.desc"
            :status="stepStatus[i + 1]"
          />
        </n-steps>
      </n-card>

      <!-- Right: Main Content -->
      <div style="flex: 1; display: flex; flex-direction: column; min-width: 0; gap: 16px;">
        
        <!-- Result / Action Card -->
        <n-card style="border-radius: 12px; flex-shrink: 0;">
          <template #header>
            <div style="display: flex; align-items: center; gap: 8px;">
              <n-icon :size="20" color="#58a6ff"><component :is="workflowSteps[currentStepIndex-1].icon" /></n-icon>
              <span>{{ workflowSteps[currentStepIndex-1].title }}</span>
              <n-spin v-if="isLoading" :size="16" style="margin-left: 8px;" />
              <n-tag v-else-if="stepStatus[currentStepIndex] === 'finish'" type="success" size="small" style="margin-left: 8px;">已完成</n-tag>
              <n-tag v-else-if="stepStatus[currentStepIndex] === 'error'" type="error" size="small" style="margin-left: 8px;">失败</n-tag>
            </div>
          </template>

          <div v-if="isLoading" style="color: #888; padding: 20px 0; text-align: center;">
            <n-spin size="large" />
            <div style="margin-top: 12px;">Claude 正在后台执行，请稍候...</div>
          </div>
          
          <div v-else-if="stepStatus[currentStepIndex] === 'finish'">
            <n-alert type="success" :show-icon="false" style="margin-bottom: 16px; background: rgba(63, 185, 80, 0.1); border: 1px solid rgba(63, 185, 80, 0.2);">
              <pre style="margin: 0; white-space: pre-wrap; font-family: 'Cascadia Code', monospace; font-size: 13px; color: #c9d1d9;">{{ stepResults[currentStepIndex] || '执行成功，暂无输出摘要。' }}</pre>
            </n-alert>
            <div style="display: flex; gap: 12px;">
              <n-button type="primary" @click="nextStep" v-if="currentStepIndex < 6">
                <template #icon><n-icon><PlayOutline/></n-icon></template> 继续下一步
              </n-button>
              <n-button type="success" v-else>
                <template #icon><n-icon><CheckmarkDoneOutline/></n-icon></template> 完成工作流
              </n-button>
              <n-button ghost @click="retryStep">
                <template #icon><n-icon><RefreshOutline/></n-icon></template> 重新执行
              </n-button>
            </div>
          </div>

          <div v-else-if="stepStatus[currentStepIndex] === 'error'">
            <n-alert type="error" style="margin-bottom: 16px;">
              执行过程中发生错误，请查看下方控制台日志。
            </n-alert>
            <n-button type="warning" @click="retryStep">
              <template #icon><n-icon><RefreshOutline/></n-icon></template> 重试此步骤
            </n-button>
          </div>
        </n-card>

        <!-- Logs Accordion -->
        <n-collapse default-expanded-names="logs" style="background: #0d1117; border-radius: 12px; border: 1px solid #30363d; flex: 1; display: flex; flex-direction: column;">
          <n-collapse-item title="原始控制台日志 (Raw Output)" name="logs" style="display: flex; flex-direction: column; flex: 1;">
            <n-scrollbar style="max-height: 300px; padding: 12px; background: #0d1117; border-radius: 8px;">
              <div v-for="(log, idx) in logs" :key="idx" style="font-family: 'Cascadia Code', monospace; font-size: 12px; color: #8b949e; line-height: 1.5; white-space: pre-wrap;">
                {{ log }}
              </div>
              <div v-if="logs.length === 0" style="color: #484f58; text-align: center; font-style: italic;">暂无日志输出...</div>
            </n-scrollbar>
          </n-collapse-item>
        </n-collapse>
        
      </div>
    </div>
  </div>
</template>

<style scoped>
:deep(.n-collapse-item__content-inner) {
  padding: 0 !important;
  display: flex;
  flex-direction: column;
}
</style>
