<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NGrid, NGi, NProgress, NTag, NSpace,
  NDataTable, NStatistic
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { h } from "vue";
import {
  FlaskOutline, CheckmarkCircleOutline, CloseCircleOutline, RemoveCircleOutline
} from "@vicons/ionicons5";

// Mock coverage data
const coverage = ref({
  statements: 78,
  branches: 62,
  functions: 85,
  lines: 76,
});

// Mock test results
interface TestResult {
  name: string;
  suite: string;
  status: 'passed' | 'failed' | 'skipped';
  duration: string;
}

const testResults = ref<TestResult[]>([
  { name: '正常登录流程', suite: 'auth.test.ts', status: 'passed', duration: '120ms' },
  { name: '密码错误拒绝登录', suite: 'auth.test.ts', status: 'passed', duration: '85ms' },
  { name: '连续5次失败锁定账户', suite: 'auth.test.ts', status: 'failed', duration: '230ms' },
  { name: '邮箱格式校验', suite: 'validation.test.ts', status: 'passed', duration: '15ms' },
  { name: 'Token 过期自动刷新', suite: 'token.test.ts', status: 'passed', duration: '340ms' },
  { name: '验证码过期场景', suite: 'email.test.ts', status: 'skipped', duration: '-' },
  { name: '并发登录限制', suite: 'auth.test.ts', status: 'passed', duration: '450ms' },
]);

const columns: DataTableColumns<TestResult> = [
  {
    title: '状态',
    key: 'status',
    width: 80,
    render(row) {
      const map: Record<string, { type: string; label: string }> = {
        passed: { type: 'success', label: 'Pass' },
        failed: { type: 'error', label: 'Fail' },
        skipped: { type: 'default', label: 'Skip' },
      };
      const item = map[row.status];
      return h(NTag, { type: item.type as any, size: 'small', bordered: false }, { default: () => item.label });
    }
  },
  { title: '测试用例', key: 'name' },
  {
    title: '测试套件',
    key: 'suite',
    width: 180,
    render(row) {
      return h('span', { style: 'font-family: monospace; font-size: 12px; color: #888;' }, row.suite);
    }
  },
  { title: '耗时', key: 'duration', width: 80 },
];

const passedCount = ref(testResults.value.filter(t => t.status === 'passed').length);
const failedCount = ref(testResults.value.filter(t => t.status === 'failed').length);
const skippedCount = ref(testResults.value.filter(t => t.status === 'skipped').length);
</script>

<template>
  <div>
    <!-- Action & Stats -->
    <n-grid :x-gap="16" :y-gap="16" :cols="5" style="margin-bottom: 20px;">
      <!-- Generate Button -->
      <n-gi :span="1">
        <n-card size="small" style="border-radius: 12px; height: 100%; display: flex; align-items: center; justify-content: center;">
          <n-button type="info" size="large" block style="height: 80px;">
            <template #icon><n-icon :size="24"><FlaskOutline /></n-icon></template>
            生成单元测试
          </n-button>
        </n-card>
      </n-gi>
      <!-- Coverage Rings -->
      <n-gi :span="4">
        <n-card title="覆盖率指标" size="small" style="border-radius: 12px;">
          <div style="display: flex; justify-content: space-around; align-items: center;">
            <div style="text-align: center;">
              <n-progress type="circle" :percentage="coverage.statements" :stroke-width="6" style="width: 80px;" />
              <div style="font-size: 12px; color: #888; margin-top: 6px;">Statements</div>
            </div>
            <div style="text-align: center;">
              <n-progress type="circle" :percentage="coverage.branches" :stroke-width="6" style="width: 80px;" status="warning" />
              <div style="font-size: 12px; color: #888; margin-top: 6px;">Branches</div>
            </div>
            <div style="text-align: center;">
              <n-progress type="circle" :percentage="coverage.functions" :stroke-width="6" style="width: 80px;" />
              <div style="font-size: 12px; color: #888; margin-top: 6px;">Functions</div>
            </div>
            <div style="text-align: center;">
              <n-progress type="circle" :percentage="coverage.lines" :stroke-width="6" style="width: 80px;" />
              <div style="font-size: 12px; color: #888; margin-top: 6px;">Lines</div>
            </div>
          </div>
        </n-card>
      </n-gi>
    </n-grid>

    <!-- Test Summary Bar -->
    <n-card size="small" style="border-radius: 12px; margin-bottom: 16px;">
      <div style="display: flex; gap: 32px; align-items: center;">
        <n-statistic label="Total" :value="testResults.length" />
        <div style="display: flex; align-items: center; gap: 6px;">
          <n-icon color="#63e2b7"><CheckmarkCircleOutline /></n-icon>
          <span style="color: #63e2b7; font-weight: 600;">{{ passedCount }} Passed</span>
        </div>
        <div style="display: flex; align-items: center; gap: 6px;">
          <n-icon color="#e88080"><CloseCircleOutline /></n-icon>
          <span style="color: #e88080; font-weight: 600;">{{ failedCount }} Failed</span>
        </div>
        <div style="display: flex; align-items: center; gap: 6px;">
          <n-icon color="#888"><RemoveCircleOutline /></n-icon>
          <span style="color: #888; font-weight: 600;">{{ skippedCount }} Skipped</span>
        </div>
      </div>
    </n-card>

    <!-- Test Results Table -->
    <n-data-table
      :columns="columns"
      :data="testResults"
      :bordered="false"
      :single-line="false"
      size="small"
      style="border-radius: 8px;"
    />
  </div>
</template>
