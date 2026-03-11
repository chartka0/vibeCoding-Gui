<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NTag, NSpace, NInput, NGrid, NGi,
  NList, NListItem, NThing, NSelect
} from "naive-ui";
import {
  GitCommitOutline, CloudUploadOutline, CheckmarkOutline
} from "@vicons/ionicons5";

// Mock commit type options
const commitTypeOptions = [
  { label: 'feat: 新功能', value: 'feat' },
  { label: 'fix: 修复Bug', value: 'fix' },
  { label: 'refactor: 重构', value: 'refactor' },
  { label: 'style: 样式调整', value: 'style' },
  { label: 'docs: 文档变更', value: 'docs' },
  { label: 'test: 测试相关', value: 'test' },
  { label: 'chore: 构建/工具', value: 'chore' },
];

const commitType = ref('feat');
const commitScope = ref('auth');
const commitMessage = ref('新增登录界面找回密码功能');
const commitBody = ref('- 添加邮箱验证码发送服务\n- 实现密码重置 Token 管理\n- 新增 PasswordReset.vue 组件');

// Mock changed files
const changedFiles = ref([
  { name: 'src/services/auth.service.ts', status: 'modified', additions: 45, deletions: 12 },
  { name: 'src/services/email.service.ts', status: 'added', additions: 68, deletions: 0 },
  { name: 'src/components/LoginForm.vue', status: 'modified', additions: 23, deletions: 8 },
  { name: 'src/components/PasswordReset.vue', status: 'added', additions: 95, deletions: 0 },
  { name: 'src/utils/token.ts', status: 'added', additions: 32, deletions: 0 },
]);

// Computed preview
const commitPreview = ref('');
function updatePreview() {
  const scope = commitScope.value ? `(${commitScope.value})` : '';
  commitPreview.value = `${commitType.value}${scope}: ${commitMessage.value}`;
}
updatePreview();
</script>

<template>
  <div>
    <n-grid :x-gap="20" :cols="5">
      <!-- Left: Changed Files (2 cols) -->
      <n-gi :span="2">
        <n-card title="变更文件" size="medium" style="border-radius: 12px; height: 100%;">
          <template #header-extra>
            <n-tag type="info" size="small" :bordered="false">{{ changedFiles.length }} 文件</n-tag>
          </template>
          <n-list hoverable clickable style="background: transparent;">
            <n-list-item v-for="file in changedFiles" :key="file.name">
              <n-thing>
                <template #header>
                  <span style="font-family: monospace; font-size: 13px;">{{ file.name }}</span>
                </template>
                <template #header-extra>
                  <n-space :size="6">
                    <n-tag
                      :type="file.status === 'added' ? 'success' : 'warning'"
                      size="tiny"
                      :bordered="false"
                    >
                      {{ file.status === 'added' ? 'NEW' : 'MOD' }}
                    </n-tag>
                    <span style="font-size: 12px; color: #63e2b7;">+{{ file.additions }}</span>
                    <span v-if="file.deletions > 0" style="font-size: 12px; color: #e88080;">-{{ file.deletions }}</span>
                  </n-space>
                </template>
              </n-thing>
            </n-list-item>
          </n-list>
        </n-card>
      </n-gi>

      <!-- Right: Commit Form (3 cols) -->
      <n-gi :span="3">
        <n-card title="提交信息" size="medium" style="border-radius: 12px;">
          <template #header-extra>
            <n-tag type="success" size="small" :bordered="false">Conventional Commit</n-tag>
          </template>

          <n-space vertical :size="16">
            <!-- Type & Scope -->
            <n-grid :x-gap="12" :cols="2">
              <n-gi>
                <div style="font-size: 12px; color: #888; margin-bottom: 4px;">类型</div>
                <n-select v-model:value="commitType" :options="commitTypeOptions" size="medium" @update:value="updatePreview" />
              </n-gi>
              <n-gi>
                <div style="font-size: 12px; color: #888; margin-bottom: 4px;">作用域 (可选)</div>
                <n-input v-model:value="commitScope" placeholder="e.g. auth, ui" size="medium" @update:value="updatePreview" />
              </n-gi>
            </n-grid>

            <!-- Subject -->
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">标题</div>
              <n-input v-model:value="commitMessage" placeholder="简要描述变更内容" size="medium" @update:value="updatePreview" />
            </div>

            <!-- Body -->
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">详细描述 (可选)</div>
              <n-input v-model:value="commitBody" type="textarea" :autosize="{ minRows: 3, maxRows: 6 }" placeholder="详细说明变更原因和内容..." />
            </div>

            <!-- Preview -->
            <div style="background: #0a0a0e; border-radius: 6px; padding: 12px; font-family: monospace; font-size: 13px;">
              <div style="color: #888; font-size: 11px; margin-bottom: 6px;">预览：</div>
              <div style="color: #70c0e8;">{{ commitPreview }}</div>
              <div v-if="commitBody" style="color: #a0a0a0; margin-top: 8px; white-space: pre-wrap;">{{ commitBody }}</div>
            </div>

            <!-- Actions -->
            <n-space>
              <n-button type="primary" size="large" style="border-radius: 8px;">
                <template #icon><n-icon><GitCommitOutline /></n-icon></template>
                Commit
              </n-button>
              <n-button type="info" size="large" style="border-radius: 8px;">
                <template #icon><n-icon><CloudUploadOutline /></n-icon></template>
                Commit & Push
              </n-button>
            </n-space>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
  </div>
</template>
