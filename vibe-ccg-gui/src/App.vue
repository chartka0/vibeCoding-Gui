<script setup lang="ts">
import { ref, computed } from "vue";
import { darkTheme, NConfigProvider, NMessageProvider, NLayout, NLayoutContent } from "naive-ui";

import Sidebar from "./components/layout/Sidebar.vue";
import Header from "./components/layout/Header.vue";
import WorkflowPanel from "./components/panels/WorkflowPanel.vue";
import PlanPanel from "./components/panels/PlanPanel.vue";
import BuildPanel from "./components/panels/BuildPanel.vue";
import ReviewPanel from "./components/panels/ReviewPanel.vue";
import DiagnosticsPanel from "./components/panels/DiagnosticsPanel.vue";
import TestPanel from "./components/panels/TestPanel.vue";
import SpecPanel from "./components/panels/SpecPanel.vue";
import CommitPanel from "./components/panels/CommitPanel.vue";
import RollbackPanel from "./components/panels/RollbackPanel.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";

const selectedKey = ref('workflow');

const panelTitles: Record<string, { title: string; subtitle?: string }> = {
  workflow: { title: '完整心流', subtitle: '/ccg:workflow' },
  plan: { title: '规划面板', subtitle: '/ccg:plan' },
  build: { title: '构建监控', subtitle: '/ccg:execute' },
  review: { title: '代码审查', subtitle: '/ccg:review' },
  diagnostics: { title: '诊断优化', subtitle: '/ccg:debug' },
  test: { title: '测试面板', subtitle: '/ccg:test' },
  spec: { title: '约束画板', subtitle: '/ccg:spec-research' },
  commit: { title: '智能提交', subtitle: '/ccg:commit' },
  rollback: { title: '时光机', subtitle: '/ccg:rollback' },
  settings: { title: '设置' },
};

const currentPanel = computed(() => panelTitles[selectedKey.value] || { title: selectedKey.value });
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-layout has-sider style="height: 100vh;">
        <Sidebar v-model:selectedKey="selectedKey" />
        <n-layout>
          <Header :title="currentPanel.title" :subtitle="currentPanel.subtitle" />
          <n-layout-content style="padding: 20px; background: #101014;" :native-scrollbar="false">
            <WorkflowPanel v-if="selectedKey === 'workflow'" />
            <PlanPanel v-else-if="selectedKey === 'plan'" />
            <BuildPanel v-else-if="selectedKey === 'build'" />
            <ReviewPanel v-else-if="selectedKey === 'review'" />
            <DiagnosticsPanel v-else-if="selectedKey === 'diagnostics'" />
            <TestPanel v-else-if="selectedKey === 'test'" />
            <SpecPanel v-else-if="selectedKey === 'spec'" />
            <CommitPanel v-else-if="selectedKey === 'commit'" />
            <RollbackPanel v-else-if="selectedKey === 'rollback'" />
            <SettingsPanel v-else-if="selectedKey === 'settings'" />
            <div v-else style="display:flex; justify-content:center; align-items:center; height: 100%; color: #555; font-size: 14px;">
              模块开发中...
            </div>
          </n-layout-content>
        </n-layout>
      </n-layout>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: #101014;
}
</style>
