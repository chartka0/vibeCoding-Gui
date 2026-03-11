<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NGrid, NGi, NButton, NIcon, NTag, NSpace,
  NAlert, NScrollbar
} from "naive-ui";
import {
  CheckmarkOutline, CloseOutline, ChatbubblesOutline
} from "@vicons/ionicons5";

// Mock diff data
const diffBlocks = ref([
  {
    file: 'src/services/auth.service.ts',
    status: 'modified',
    severity: 'warning',
    comment: '建议：密码哈希应使用 bcrypt 而非 MD5，避免安全风险。',
    oldCode: `export async function login(email: string, password: string) {
  const hash = md5(password);
  const user = await db.users.findOne({ email, password: hash });
  if (!user) throw new Error('Invalid credentials');
  return generateToken(user.id);
}`,
    newCode: `export async function login(email: string, password: string) {
  const user = await db.users.findOne({ email });
  if (!user) throw new Error('Invalid credentials');
  const valid = await bcrypt.compare(password, user.passwordHash);
  if (!valid) throw new Error('Invalid credentials');
  return generateToken(user.id);
}`,
  },
  {
    file: 'src/components/LoginForm.vue',
    status: 'modified',
    severity: 'info',
    comment: '样式优化：表单宽度应限制在 400px 以内，居中展示。',
    oldCode: `<n-form ref="formRef" :model="formData" :rules="rules">
  <n-form-item label="邮箱" path="email">
    <n-input v-model:value="formData.email" />
  </n-form-item>
  <n-form-item label="密码" path="password">
    <n-input v-model:value="formData.password" type="password" />
  </n-form-item>
</n-form>`,
    newCode: `<n-form ref="formRef" :model="formData" :rules="rules"
  style="max-width: 400px; margin: 0 auto;">
  <n-form-item label="邮箱" path="email">
    <n-input v-model:value="formData.email" placeholder="请输入邮箱" />
  </n-form-item>
  <n-form-item label="密码" path="password">
    <n-input v-model:value="formData.password" type="password"
      show-password-on="click" placeholder="请输入密码" />
  </n-form-item>
</n-form>`,
  },
]);

// Mock summary
const reviewSummary = ref({
  total: 5,
  passed: 3,
  warnings: 1,
  errors: 1,
});
</script>

<template>
  <div>
    <!-- Review Summary -->
    <n-card size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <div style="display: flex; gap: 24px;">
          <div style="text-align: center;">
            <div style="font-size: 24px; font-weight: 700; color: #e0e0e0;">{{ reviewSummary.total }}</div>
            <div style="font-size: 12px; color: #888;">文件总数</div>
          </div>
          <div style="text-align: center;">
            <div style="font-size: 24px; font-weight: 700; color: #63e2b7;">{{ reviewSummary.passed }}</div>
            <div style="font-size: 12px; color: #888;">已通过</div>
          </div>
          <div style="text-align: center;">
            <div style="font-size: 24px; font-weight: 700; color: #f2c97d;">{{ reviewSummary.warnings }}</div>
            <div style="font-size: 12px; color: #888;">警告</div>
          </div>
          <div style="text-align: center;">
            <div style="font-size: 24px; font-weight: 700; color: #e88080;">{{ reviewSummary.errors }}</div>
            <div style="font-size: 12px; color: #888;">错误</div>
          </div>
        </div>
        <n-space>
          <n-button type="success" size="small">全部接受</n-button>
          <n-button type="error" size="small" secondary>全部拒绝</n-button>
        </n-space>
      </div>
    </n-card>

    <!-- Diff Blocks -->
    <n-space vertical :size="16">
      <n-card
        v-for="(diff, index) in diffBlocks"
        :key="index"
        :title="diff.file"
        size="small"
        style="border-radius: 12px;"
      >
        <template #header-extra>
          <n-space :size="8">
            <n-tag :type="diff.severity" size="small" :bordered="false">
              {{ diff.severity === 'warning' ? 'Warning' : 'Info' }}
            </n-tag>
            <n-button type="success" size="tiny" secondary>
              <template #icon><n-icon><CheckmarkOutline /></n-icon></template>
              接受
            </n-button>
            <n-button type="error" size="tiny" secondary>
              <template #icon><n-icon><CloseOutline /></n-icon></template>
              拒绝
            </n-button>
          </n-space>
        </template>

        <!-- Claude Comment -->
        <n-alert :type="diff.severity" style="margin-bottom: 12px; border-radius: 6px;" :show-icon="true">
          <template #icon><n-icon><ChatbubblesOutline /></n-icon></template>
          <span style="font-size: 13px;">Claude: {{ diff.comment }}</span>
        </n-alert>

        <!-- Diff View -->
        <n-grid :x-gap="12" :cols="2">
          <n-gi>
            <div style="background: #1a1215; border-radius: 6px; padding: 12px; border: 1px solid #3a2020;">
              <div style="font-size: 11px; color: #e88080; margin-bottom: 8px; font-weight: 600;">- 原始代码</div>
              <pre style="margin: 0; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.6; color: #c0a0a0; white-space: pre-wrap; word-break: break-all;">{{ diff.oldCode }}</pre>
            </div>
          </n-gi>
          <n-gi>
            <div style="background: #121a15; border-radius: 6px; padding: 12px; border: 1px solid #203a20;">
              <div style="font-size: 11px; color: #63e2b7; margin-bottom: 8px; font-weight: 600;">+ 建议代码</div>
              <pre style="margin: 0; font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px; line-height: 1.6; color: #a0c0a0; white-space: pre-wrap; word-break: break-all;">{{ diff.newCode }}</pre>
            </div>
          </n-gi>
        </n-grid>
      </n-card>
    </n-space>
  </div>
</template>
