<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NSteps, NStep,
  NGradientText, NTag, NAlert, NInput
} from "naive-ui";
import {
  FlashOutline, DocumentTextOutline, CodeSlashOutline,
  CheckmarkDoneOutline, SearchOutline, FolderOpenOutline
} from "@vicons/ionicons5";
import { useWorkspaceStore } from "../../store/workspace";
import TerminalEmulator from "../terminal/TerminalEmulator.vue";

const workspaceStore = useWorkspaceStore();
const terminalRef = ref<InstanceType<typeof TerminalEmulator>>();
const planFilePath = ref('');
const currentStep = ref(0);
const isRunning = ref(false);

const codexSteps = [
  { title: '读取计划', desc: '解析 plan 文件', icon: DocumentTextOutline },
  { title: 'Codex 执行', desc: 'MCP搜索 + 代码实现', icon: FlashOutline },
  { title: 'Claude 审核', desc: '轻量验证变更', icon: SearchOutline },
  { title: '多模型审查', desc: 'Codex + Gemini 交叉审查', icon: CodeSlashOutline },
  { title: '交付', desc: '报告结果与建议', icon: CheckmarkDoneOutline },
];

const modeToStep: Record<string, number> = {
  '准备': 1,
  '执行': 2,
  '审核': 3,
  '审查': 4,
  '交付': 5,
  '追加': 3,
};

function onModeChange(mode: string) {
  const step = modeToStep[mode];
  if (step !== undefined) {
    currentStep.value = step;
  }
}

function startCodexExec() {
  const currentWs = workspaceStore.currentWorkspace;
  if (!currentWs || !planFilePath.value.trim()) return;

  isRunning.value = true;
  currentStep.value = 0;

  const escapedPath = planFilePath.value.replace(/"/g, '\\"');
  const cmd = `claude "/ccg:codex-exec ${escapedPath}"`;

  terminalRef.value?.executeCommand(cmd);
}
</script>

<template>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <n-alert v-if="!workspaceStore.currentWorkspaceId" type="warning" style="margin-bottom: 16px;">
      请先在左侧边栏选择或创建一个项目 (Workspace)。
    </n-alert>

    <!-- Command Input Section -->
    <n-card style="border-radius: 12px; margin-bottom: 16px; background: linear-gradient(135deg, #18181c 0%, #221a2e 100%); flex-shrink: 0;">
      <div style="padding: 16px 12px 12px;">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
          <n-gradient-text type="warning" :size="22" style="font-weight: 800;">
            Codex 全权执行
          </n-gradient-text>
          <n-tag v-if="workspaceStore.currentWorkspace" type="success" size="small" round>
            <template #icon><n-icon><FolderOpenOutline/></n-icon></template>
            {{ workspaceStore.currentWorkspace.name }}
          </n-tag>
        </div>

        <p style="color: #888; font-size: 12px; margin: 0 0 16px;">
          需先用「规划面板」生成计划文件，再由 Codex 一把梭全权执行。Claude Token 消耗极低。
        </p>

        <div style="margin-bottom: 12px;">
          <div style="margin-bottom: 6px; font-weight: 600; color: #ccc; font-size: 13px;">
            计划文件路径 (Plan File):
          </div>
          <n-input
            v-model:value="planFilePath"
            placeholder=".claude/plan/user-auth.md"
            style="border-radius: 8px; font-size: 13px; background: rgba(0, 0, 0, 0.2);"
          />
        </div>

        <div style="display: flex; gap: 10px; margin-bottom: 12px;">
          <n-button
            color="#8a2be2"
            size="medium"
            style="border-radius: 8px; font-weight: 600;"
            :disabled="!workspaceStore.currentWorkspaceId || !planFilePath.trim() || isRunning"
            @click="startCodexExec"
          >
            <template #icon><n-icon><FlashOutline /></n-icon></template>
            启动 Codex 全权执行
          </n-button>
        </div>

        <!-- 5-Step Progress -->
        <n-steps :current="currentStep" size="small" style="padding: 4px 0;">
          <n-step
            v-for="(step, i) in codexSteps"
            :key="i"
            :title="step.title"
          />
        </n-steps>
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
