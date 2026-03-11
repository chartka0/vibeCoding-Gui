<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NDataTable, NTag, NSpace, NGrid, NGi, NTabs, NTabPane
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { h } from "vue";
import {
  SearchOutline, HammerOutline, SpeedometerOutline
} from "@vicons/ionicons5";

const activeTab = ref('debug');

// Mock diagnostics data
interface DiagItem {
  severity: string;
  file: string;
  line: number;
  description: string;
  suggestion: string;
}

const debugIssues = ref<DiagItem[]>([
  { severity: 'error', file: 'src/services/auth.service.ts', line: 23, description: 'TypeError: Cannot read property \'id\' of undefined', suggestion: '添加空值检查: if (!user) return null' },
  { severity: 'error', file: 'src/utils/token.ts', line: 8, description: 'JWT_SECRET 环境变量未定义', suggestion: '检查 .env 文件是否包含 JWT_SECRET' },
  { severity: 'warning', file: 'src/components/LoginForm.vue', line: 45, description: '表单未设置 prevent-default', suggestion: '添加 @submit.prevent 修饰符' },
  { severity: 'info', file: 'src/services/email.service.ts', line: 12, description: '未捕获的 Promise rejection', suggestion: '添加 try-catch 包裹 async 调用' },
]);

const optimizeIssues = ref<DiagItem[]>([
  { severity: 'warning', file: 'src/services/auth.service.ts', line: 15, description: '数据库查询未使用索引', suggestion: '为 email 字段创建唯一索引' },
  { severity: 'info', file: 'src/components/LoginForm.vue', line: 3, description: '未使用的 import: NAlert', suggestion: '移除未使用的导入以减小包体积' },
  { severity: 'warning', file: 'src/utils/token.ts', line: 20, description: 'Token 过期时间硬编码为 24h', suggestion: '提取为配置项 TOKEN_EXPIRY' },
]);

const createColumns = (): DataTableColumns<DiagItem> => [
  {
    title: '严重度',
    key: 'severity',
    width: 80,
    render(row) {
      const typeMap: Record<string, string> = { error: 'error', warning: 'warning', info: 'info' };
      const labelMap: Record<string, string> = { error: 'Error', warning: 'Warn', info: 'Info' };
      return h(NTag, { type: typeMap[row.severity] as any, size: 'small', bordered: false }, { default: () => labelMap[row.severity] });
    }
  },
  {
    title: '文件',
    key: 'file',
    width: 220,
    render(row) {
      return h('span', { style: 'font-family: monospace; font-size: 12px;' }, `${row.file}:${row.line}`);
    }
  },
  { title: '描述', key: 'description', ellipsis: { tooltip: true } },
  { title: '建议修复', key: 'suggestion', ellipsis: { tooltip: true } },
];

const columns = createColumns();

// Mock console output
const consoleOutput = ref(`[14:35:01] [Debug] 启动全局诊断扫描...
[14:35:02] [Debug] 扫描 src/services/ 目录 (3 files)
[14:35:03] [Debug] 发现 2 个 Error，1 个 Warning，1 个 Info
[14:35:04] [Debug] auth.service.ts:23 - TypeError: null reference
[14:35:04] [Debug] token.ts:8 - 环境变量缺失
[14:35:05] [Debug] LoginForm.vue:45 - 表单事件处理不完整
[14:35:06] [Debug] 扫描完毕，等待用户确认修复方案...`);
</script>

<template>
  <div>
    <!-- Action Bar -->
    <n-card size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <n-space>
          <n-button type="info" size="medium">
            <template #icon><n-icon><SearchOutline /></n-icon></template>
            一键智能诊断
          </n-button>
          <n-button type="warning" size="medium" secondary>
            <template #icon><n-icon><SpeedometerOutline /></n-icon></template>
            性能优化扫描
          </n-button>
        </n-space>
        <n-button type="success" size="medium">
          <template #icon><n-icon><HammerOutline /></n-icon></template>
          自动修复全部
        </n-button>
      </div>
    </n-card>

    <n-tabs v-model:value="activeTab" type="segment" style="margin-bottom: 16px;">
      <n-tab-pane name="debug" tab="问题诊断 (/ccg:debug)">
        <n-data-table
          :columns="columns"
          :data="debugIssues"
          :bordered="false"
          :single-line="false"
          size="small"
          style="border-radius: 8px;"
        />
      </n-tab-pane>
      <n-tab-pane name="optimize" tab="优化建议 (/ccg:optimize)">
        <n-data-table
          :columns="columns"
          :data="optimizeIssues"
          :bordered="false"
          :single-line="false"
          size="small"
          style="border-radius: 8px;"
        />
      </n-tab-pane>
    </n-tabs>

    <!-- Console Output -->
    <n-card title="诊断控制台" size="small" style="border-radius: 12px; margin-top: 16px;">
      <div style="background: #0a0a0e; border-radius: 6px; padding: 12px; height: 180px; overflow-y: auto; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.8; color: #b0b0b0; white-space: pre-wrap;">{{ consoleOutput }}</div>
    </n-card>
  </div>
</template>
