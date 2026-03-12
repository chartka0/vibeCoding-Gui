<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NSteps, NStep,
  NGradientText, NTag, NAlert, NInput
} from "naive-ui";
import {
  RocketOutline, DocumentTextOutline, ChatbubblesOutline,
  CodeSlashOutline, GitBranchOutline, CheckmarkDoneOutline,
  FlashOutline, BrowsersOutline, FolderOpenOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import TerminalEmulator from "../terminal/TerminalEmulator.vue";

const workspaceStore = useWorkspaceStore();
const terminalRef = ref<InstanceType<typeof TerminalEmulator>>();
const workflowPrompt = ref('');
const currentStep = ref(0);
const isRunning = ref(false);

const workflowSteps = [
  { title: '研究', desc: '需求收集、上下文与分析', icon: DocumentTextOutline },
  { title: '构思', desc: '双模型并行分析可行性', icon: ChatbubblesOutline },
  { title: '计划', desc: '多模型产出前后端架构规划', icon: CodeSlashOutline },
  { title: '执行', desc: '严格按批准计划编码实施', icon: FlashOutline },
  { title: '优化', desc: '多模型并行审查安全与设计', icon: GitBranchOutline },
  { title: '评审', desc: '最终评估与测试闭环交付', icon: CheckmarkDoneOutline },
];

// Map mode tags from CCG output to step index
const modeToStep: Record<string, number> = {
  '研究': 1,
  '构思': 2,
  '计划': 3,
  '执行': 4,
  '优化': 5,
  '评审': 6,
  '准备': 1,
};

function onModeChange(mode: string) {
  const step = modeToStep[mode];
  if (step !== undefined) {
    currentStep.value = step;
  }
}

function startWorkflow(mode: 'workflow' | 'frontend' | 'backend') {
  const currentWs = workspaceStore.currentWorkspace;
  if (!currentWs || !workflowPrompt.value.trim()) return;

  isRunning.value = true;
  currentStep.value = 0;

  // Build the Claude CLI command with the CCG slash command
  const ccgCommand = `/ccg:${mode}`;
  const escapedPrompt = workflowPrompt.value.replace(/"/g, '\\"');
  const cmd = `claude "${ccgCommand} ${escapedPrompt}"`;

  // Execute in the embedded terminal
  terminalRef.value?.executeCommand(cmd);
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning" style="margin-bottom: 16px;">
      请先在左侧边栏选择或创建一个项目 (Workspace) 以启动工作流。
    </n-alert>
    
    <!-- Command Input Section -->
    <n-card style="border-radius: 12px; margin-bottom: 16px; background: linear-gradient(135deg, #18181c 0%, #1a2332 100%); flex-shrink: 0;">
      <div style="padding: 16px 12px 12px;">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
          <n-gradient-text type="info" :size="22" style="font-weight: 800;">
            vibeCoding Command Center
          </n-gradient-text>
          <n-tag v-if="workspaceStore.currentWorkspace" type="success" size="small" round>
            <template #icon><n-icon><FolderOpenOutline/></n-icon></template>
            {{ workspaceStore.currentWorkspace.name }}
          </n-tag>
        </div>

        <div style="margin-bottom: 12px;">
          <n-input
            v-model:value="workflowPrompt"
            type="textarea"
            placeholder="输入你的需求，例如：帮我写一个基于 Vue3 的 Todo 列表页面，支持本地存储..."
            :autosize="{ minRows: 2, maxRows: 5 }"
            style="border-radius: 8px; font-size: 13px; background: rgba(0, 0, 0, 0.2);"
          />
        </div>

        <!-- 3 Mode Buttons -->
        <div style="display: flex; flex-wrap: wrap; gap: 10px; margin-bottom: 12px;">
          <n-button 
            type="info" 
            size="medium" 
            style="border-radius: 8px; font-weight: 600;"
            :disabled="!workspaceStore.currentWorkspaceId || !workflowPrompt.trim() || isRunning"
            @click="startWorkflow('workflow')"
          >
            <template #icon><n-icon><RocketOutline /></n-icon></template>
            完整工作流 (6阶段)
          </n-button>
          
          <n-button 
            type="success" 
            size="medium" 
            secondary
            style="border-radius: 8px; font-weight: 600;"
            :disabled="!workspaceStore.currentWorkspaceId || !workflowPrompt.trim() || isRunning"
            @click="startWorkflow('frontend')"
          >
            <template #icon><n-icon><BrowsersOutline /></n-icon></template>
            极速前端 (Gemini)
          </n-button>

          <n-button 
            type="warning" 
            size="medium" 
            secondary
            style="border-radius: 8px; font-weight: 600;"
            :disabled="!workspaceStore.currentWorkspaceId || !workflowPrompt.trim() || isRunning"
            @click="startWorkflow('backend')"
          >
            <template #icon><n-icon><CodeSlashOutline /></n-icon></template>
            极速后端 (Codex)
          </n-button>
        </div>

        <!-- Step Progress (compact) -->
        <n-steps :current="currentStep" size="small" style="padding: 4px 0;">
          <n-step
            v-for="(step, i) in workflowSteps"
            :key="i"
            :title="step.title"
          />
        </n-steps>

        <n-alert type="warning" :show-icon="false" style="margin-top: 8px; font-size: 11px; background: rgba(240, 160, 32, 0.05); border: 1px dashed rgba(240, 160, 32, 0.2);">
          ⚠️ 大型系统级需求建议使用左侧「规划面板」拆解后分段执行。
        </n-alert>
      </div>
    </n-card>

    <!-- Embedded Terminal -->
    <n-card 
      title="终端 (Terminal)" 
      size="small" 
      style="border-radius: 12px; flex: 1; min-height: 0; display: flex; flex-direction: column;"
      :content-style="{ flex: 1, padding: '8px', display: 'flex', minHeight: 0 }"
    >
      <TerminalEmulator ref="terminalRef" @mode-change="onModeChange" />
    </n-card>
  </div>
</template>
