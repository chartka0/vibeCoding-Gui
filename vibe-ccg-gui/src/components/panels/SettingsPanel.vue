<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NInput, NGrid, NGi, NSpace, NSwitch,
  NSelect, NInputGroup, NTag, NForm, NFormItem, NDivider
} from "naive-ui";
import {
  SaveOutline, RefreshOutline
} from "@vicons/ionicons5";

// Mock settings
const claudePath = ref('claude');
const codexPath = ref('codex');
const geminiPath = ref('gemini');

const claudeApiKey = ref('sk-ant-***...***');
const codexApiKey = ref('sk-***...***');
const geminiApiKey = ref('AIza***...***');

const language = ref('zh');
const languageOptions = [
  { label: '中文', value: 'zh' },
  { label: 'English', value: 'en' },
];

const autoCommit = ref(false);
const autoReview = ref(true);
const darkMode = ref(true);
</script>

<template>
  <div>
    <n-grid :x-gap="20" :y-gap="20" :cols="2">
      <!-- Model Configuration -->
      <n-gi>
        <n-card title="模型配置" size="medium" style="border-radius: 12px;">
          <template #header-extra>
            <n-tag type="info" size="small" :bordered="false">CLI Paths</n-tag>
          </template>
          <n-space vertical :size="16">
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">Claude CLI 路径 (Tech Lead)</div>
              <n-input v-model:value="claudePath" placeholder="claude" />
            </div>
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">Codex CLI 路径 (Backend)</div>
              <n-input v-model:value="codexPath" placeholder="codex" />
            </div>
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">Gemini CLI 路径 (Frontend)</div>
              <n-input v-model:value="geminiPath" placeholder="gemini" />
            </div>
            <n-button type="info" block style="border-radius: 8px;">
              <template #icon><n-icon><RefreshOutline /></n-icon></template>
              检测 CLI 可用性
            </n-button>
          </n-space>
        </n-card>
      </n-gi>

      <!-- API Keys -->
      <n-gi>
        <n-card title="API 密钥" size="medium" style="border-radius: 12px;">
          <template #header-extra>
            <n-tag type="warning" size="small" :bordered="false">Sensitive</n-tag>
          </template>
          <n-space vertical :size="16">
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">Anthropic API Key</div>
              <n-input v-model:value="claudeApiKey" type="password" show-password-on="click" placeholder="sk-ant-..." />
            </div>
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">OpenAI API Key (Codex)</div>
              <n-input v-model:value="codexApiKey" type="password" show-password-on="click" placeholder="sk-..." />
            </div>
            <div>
              <div style="font-size: 12px; color: #888; margin-bottom: 4px;">Google AI API Key (Gemini)</div>
              <n-input v-model:value="geminiApiKey" type="password" show-password-on="click" placeholder="AIza..." />
            </div>
            <n-button type="success" block style="border-radius: 8px;">
              <template #icon><n-icon><SaveOutline /></n-icon></template>
              保存密钥
            </n-button>
          </n-space>
        </n-card>
      </n-gi>

      <!-- Preferences -->
      <n-gi :span="2">
        <n-card title="偏好设置" size="medium" style="border-radius: 12px;">
          <n-grid :x-gap="40" :cols="3">
            <n-gi>
              <n-space vertical :size="20">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <div>
                    <div style="font-weight: 500;">界面语言</div>
                    <div style="font-size: 12px; color: #888;">选择界面显示语言</div>
                  </div>
                  <n-select v-model:value="language" :options="languageOptions" size="small" style="width: 120px;" />
                </div>
              </n-space>
            </n-gi>
            <n-gi>
              <n-space vertical :size="20">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <div>
                    <div style="font-weight: 500;">自动提交</div>
                    <div style="font-size: 12px; color: #888;">审查通过后自动 git commit</div>
                  </div>
                  <n-switch v-model:value="autoCommit" />
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <div>
                    <div style="font-weight: 500;">自动审查</div>
                    <div style="font-size: 12px; color: #888;">编码完成后自动触发 Claude Review</div>
                  </div>
                  <n-switch v-model:value="autoReview" />
                </div>
              </n-space>
            </n-gi>
            <n-gi>
              <n-space vertical :size="20">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <div>
                    <div style="font-weight: 500;">暗色模式</div>
                    <div style="font-size: 12px; color: #888;">切换界面主题</div>
                  </div>
                  <n-switch v-model:value="darkMode" />
                </div>
              </n-space>
            </n-gi>
          </n-grid>
        </n-card>
      </n-gi>
    </n-grid>
  </div>
</template>
