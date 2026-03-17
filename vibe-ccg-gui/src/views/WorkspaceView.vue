<script setup lang="ts">
import { ref, computed, type Component, markRaw } from "vue";
import { NLayout, NLayoutContent } from "naive-ui";
import { useWorkspaceStore } from "../store/workspace";

import Sidebar from "../components/layout/Sidebar.vue";
import Header from "../components/layout/Header.vue";
import WorkflowPanel from "../components/panels/WorkflowPanel.vue";
import CodexExecPanel from "../components/panels/CodexExecPanel.vue";
import PlanPanel from "../components/panels/PlanPanel.vue";
import BuildPanel from "../components/panels/BuildPanel.vue";
import ReviewPanel from "../components/panels/ReviewPanel.vue";
import DiagnosticsPanel from "../components/panels/DiagnosticsPanel.vue";
import TestPanel from "../components/panels/TestPanel.vue";
import SpecPanel from "../components/panels/SpecPanel.vue";
import CommitPanel from "../components/panels/CommitPanel.vue";
import RollbackPanel from "../components/panels/RollbackPanel.vue";
import SettingsPanel from "../components/panels/SettingsPanel.vue";
import EnvironmentPanel from "../components/panels/EnvironmentPanel.vue";
import HistoryPanel from "../components/panels/HistoryPanel.vue";

const props = defineProps<{
  workspaceId: string;
}>();

const emit = defineEmits<{
  (e: 'back-to-overview'): void
}>();

const workspaceStore = useWorkspaceStore();
// Ensure store knows the current workspace
workspaceStore.currentWorkspaceId = props.workspaceId;

const selectedKey = ref('workflow');

const panelTitles: Record<string, { title: string; subtitle?: string }> = {
  workflow: { title: '完整工作流', subtitle: '/ccg:workflow' },
  'codex-exec': { title: 'Codex 全权执行', subtitle: '/ccg:codex-exec' },
  plan: { title: '规划面板', subtitle: '/ccg:plan' },
  build: { title: '构建监控', subtitle: '/ccg:execute' },
  review: { title: '代码审查', subtitle: '/ccg:review' },
  diagnostics: { title: '诊断优化', subtitle: '/ccg:debug' },
  test: { title: '测试面板', subtitle: '/ccg:test' },
  spec: { title: '约束画板', subtitle: '/ccg:spec-research' },
  commit: { title: '智能提交', subtitle: '/ccg:commit' },
  rollback: { title: '时光机', subtitle: '/ccg:rollback' },
  history: { title: '执行历史', subtitle: '/ccg:history' },
  settings: { title: '设置' },
  environment: { title: '环境配置', subtitle: '工具链检测' },
};

// Panel component registry — markRaw to avoid reactive overhead on components
const panelComponents: Record<string, Component> = {
  workflow: markRaw(WorkflowPanel),
  'codex-exec': markRaw(CodexExecPanel),
  plan: markRaw(PlanPanel),
  build: markRaw(BuildPanel),
  review: markRaw(ReviewPanel),
  diagnostics: markRaw(DiagnosticsPanel),
  test: markRaw(TestPanel),
  spec: markRaw(SpecPanel),
  commit: markRaw(CommitPanel),
  rollback: markRaw(RollbackPanel),
  history: markRaw(HistoryPanel),
  settings: markRaw(SettingsPanel),
  environment: markRaw(EnvironmentPanel),
};

const currentPanel = computed(() => panelTitles[selectedKey.value] || { title: selectedKey.value });
const currentComponent = computed(() => panelComponents[selectedKey.value] || null);

function handleBack() {
  workspaceStore.currentWorkspaceId = null;
  emit('back-to-overview');
}
</script>

<template>
  <n-layout has-sider style="height: 100vh;">
    <Sidebar
      v-model:selectedKey="selectedKey"
      @back-to-overview="handleBack"
    />
    <n-layout>
      <Header :title="currentPanel.title" :subtitle="currentPanel.subtitle" />
      <n-layout-content style="padding: 20px; background: #101014;" :native-scrollbar="false">
        <KeepAlive>
          <component :is="currentComponent" v-if="currentComponent" :key="selectedKey" />
        </KeepAlive>
        <div v-if="!currentComponent" style="display:flex; justify-content:center; align-items:center; height: 100%; color: #555; font-size: 14px;">
          模块开发中...
        </div>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>
