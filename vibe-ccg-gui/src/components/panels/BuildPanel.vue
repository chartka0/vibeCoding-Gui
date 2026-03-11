<script setup lang="ts">
import { ref } from "vue";
import {
  NGrid, NGi, NCard, NTag, NProgress, NIcon, NButton,
  NTree, NLog, NSpace, NTimeline, NTimelineItem, NSwitch
} from "naive-ui";
import type { TreeOption } from "naive-ui";
import {
  PlayOutline, PauseOutline, StopOutline,
  FlashOutline, LogoVue, ShieldCheckmarkOutline
} from "@vicons/ionicons5";

const isRunning = ref(true);

// Mock pipelines
const pipelines = ref([
  {
    name: 'Codex Pipeline',
    role: '核心逻辑层',
    tag: 'warning',
    progress: 72,
    status: 'active',
    currentTask: '正在生成 auth.service.ts ...',
  },
  {
    name: 'Gemini Pipeline',
    role: '前端视图层',
    tag: 'success',
    progress: 45,
    status: 'active',
    currentTask: '正在组装 LoginForm.vue ...',
  },
  {
    name: 'Claude Review',
    role: '交叉审查',
    tag: 'info',
    progress: 0,
    status: 'pending',
    currentTask: '等待前置任务完毕...',
  },
]);

// Mock file tree
const fileTree: TreeOption[] = [
  {
    label: 'src',
    key: 'src',
    children: [
      {
        label: 'services',
        key: 'services',
        children: [
          { label: 'auth.service.ts', key: 'auth.service.ts', prefix: () => '~' },
          { label: 'email.service.ts', key: 'email.service.ts', prefix: () => '+' },
        ]
      },
      {
        label: 'components',
        key: 'components',
        children: [
          { label: 'LoginForm.vue', key: 'LoginForm.vue', prefix: () => '~' },
          { label: 'PasswordReset.vue', key: 'PasswordReset.vue', prefix: () => '+' },
        ]
      },
      {
        label: 'utils',
        key: 'utils',
        children: [
          { label: 'token.ts', key: 'token.ts', prefix: () => '+' },
        ]
      },
    ]
  },
];

// Mock terminal logs
const terminalLogs = `[14:32:01] [Codex] 正在分析 auth.service.ts 的依赖关系...
[14:32:03] [Codex] 生成 JWT 认证中间件骨架
[14:32:05] [Gemini] 读取 UI_Spec.md，解析登录表单组件需求
[14:32:08] [Gemini] 正在组装 LoginForm.vue (Naive UI NForm)...
[14:32:10] [Codex] auth.service.ts 核心逻辑已完成 (72%)
[14:32:12] [Gemini] LoginForm.vue 组件结构已生成 (45%)
[14:32:14] [Claude] 审查队列中有 2 个待检文件...
[14:32:15] [System] 当前进度: Codex 72% | Gemini 45% | Review 等待中`;
</script>

<template>
  <div>
    <!-- Pipeline Status Cards -->
    <n-grid :x-gap="16" :y-gap="16" :cols="3" style="margin-bottom: 20px;">
      <n-gi v-for="pipe in pipelines" :key="pipe.name">
        <n-card size="small" style="border-radius: 12px;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
            <span style="font-weight: 600; font-size: 14px;">{{ pipe.name }}</span>
            <n-tag :type="pipe.tag as any" size="small" :bordered="false">{{ pipe.role }}</n-tag>
          </div>
          <n-progress
            type="line"
            :percentage="pipe.progress"
            :status="pipe.status === 'pending' ? 'default' : 'success'"
            :indicator-placement="'inside'"
            :height="20"
            :border-radius="4"
            style="margin-bottom: 8px;"
          />
          <div style="color: #888; font-size: 12px; font-family: monospace;">
            {{ pipe.currentTask }}
          </div>
        </n-card>
      </n-gi>
    </n-grid>

    <n-grid :x-gap="16" :cols="5">
      <!-- File Tree (2 cols) -->
      <n-gi :span="2">
        <n-card title="文件变更树" size="small" style="border-radius: 12px; height: 400px;">
          <template #header-extra>
            <n-space :size="4">
              <n-tag size="tiny" type="success" :bordered="false">+ 新增</n-tag>
              <n-tag size="tiny" type="warning" :bordered="false">~ 修改</n-tag>
            </n-space>
          </template>
          <n-tree
            :data="fileTree"
            block-line
            expand-on-click
            default-expand-all
            selectable
            style="font-family: monospace; font-size: 13px;"
          />
        </n-card>
      </n-gi>

      <!-- Terminal Log (3 cols) -->
      <n-gi :span="3">
        <n-card title="构建日志" size="small" style="border-radius: 12px; height: 400px;">
          <template #header-extra>
            <n-space :size="6">
              <n-button size="tiny" secondary type="warning" v-if="isRunning">
                <template #icon><n-icon><PauseOutline /></n-icon></template>
              </n-button>
              <n-button size="tiny" secondary type="success" v-else>
                <template #icon><n-icon><PlayOutline /></n-icon></template>
              </n-button>
              <n-button size="tiny" secondary type="error">
                <template #icon><n-icon><StopOutline /></n-icon></template>
              </n-button>
            </n-space>
          </template>
          <div style="background: #0a0a0e; border-radius: 6px; padding: 12px; height: 310px; overflow-y: auto; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.8; color: #b0b0b0; white-space: pre-wrap;">{{ terminalLogs }}</div>
        </n-card>
      </n-gi>
    </n-grid>
  </div>
</template>
