<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  NCard, NButton, NIcon, NGrid, NGi, NSpace, NTag,
  NAlert, NSpin, NTooltip, NGradientText, NDivider
} from "naive-ui";
import {
  CheckmarkCircleOutline, CloseCircleOutline, RefreshOutline,
  AlertCircleOutline, CopyOutline, OpenOutline, BookOutline,
  DownloadOutline
} from "@vicons/ionicons5";
import { invoke } from "@tauri-apps/api/core";

interface ToolCheckResult {
  id: string;
  display_name: string;
  candidates: string[];
  status: string;
  resolved_path: string | null;
  version: string | null;
  source: string | null;
  latency_ms: number;
  error: string | null;
  install_url: string | null;
  install_hint: string | null;
}

interface EnvironmentCheckResult {
  checked_at: string;
  platform: string;
  tools: ToolCheckResult[];
}

const loading = ref(false);
const result = ref<EnvironmentCheckResult | null>(null);
const checkError = ref<string | null>(null);

const installedTools = computed(() =>
  result.value?.tools.filter(t => t.status === 'installed') || []
);
const missingTools = computed(() =>
  result.value?.tools.filter(t => t.status === 'not_found') || []
);
const errorTools = computed(() =>
  result.value?.tools.filter(t => t.status === 'error') || []
);
const totalCount = computed(() => result.value?.tools.length || 0);
const allInstalled = computed(() => installedTools.value.length === totalCount.value);

async function checkEnvironment() {
  loading.value = true;
  checkError.value = null;
  try {
    result.value = await invoke<EnvironmentCheckResult>('check_environment');
  } catch (e: any) {
    checkError.value = String(e);
  } finally {
    loading.value = false;
  }
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'installed': return '#63e2b7';
    case 'not_found': return '#f85149';
    case 'error': return '#e2a727';
    default: return '#888';
  }
}

function getStatusLabel(status: string): string {
  switch (status) {
    case 'installed': return '已安装';
    case 'not_found': return '未安装';
    case 'error': return '检测异常';
    default: return '未知';
  }
}

function getStatusType(status: string): 'success' | 'error' | 'warning' | 'default' {
  switch (status) {
    case 'installed': return 'success';
    case 'not_found': return 'error';
    case 'error': return 'warning';
    default: return 'default';
  }
}

const toolIcons: Record<string, string> = {
  node: '\u2B22',
  git: '\u2387',
  claude: '\u2728',
  ccg: '\uD83D\uDE80',
  opsx: '\uD83D\uDCCB',
  codex: '\uD83E\uDDE0',
  gemini: '\u2666',
};

function getToolIcon(id: string): string {
  return toolIcons[id] || '\uD83D\uDD27';
}

onMounted(() => {
  checkEnvironment();
});
</script>

<template>
  <div>
    <!-- Summary Header -->
    <n-card
      style="border-radius: 12px; margin-bottom: 20px; background: linear-gradient(135deg, #18181c 0%, #1a2332 100%);"
    >
      <div style="display: flex; align-items: center; justify-content: space-between;">
        <div style="display: flex; align-items: center; gap: 16px;">
          <n-gradient-text type="info" :size="20" style="font-weight: 700;">
            环境配置
          </n-gradient-text>
          <n-tag v-if="result && !loading" :type="allInstalled ? 'success' : 'warning'" size="medium" round>
            {{ installedTools.length }} / {{ totalCount }} 已安装
          </n-tag>
        </div>
        <div style="display: flex; align-items: center; gap: 12px;">
          <span v-if="result && !loading" style="font-size: 11px; color: #555;">
            {{ result.platform }} · {{ new Date(result.checked_at).toLocaleTimeString() }}
          </span>
          <n-button
            type="info"
            :loading="loading"
            size="small"
            style="border-radius: 8px;"
            @click="checkEnvironment"
          >
            <template #icon><n-icon><RefreshOutline /></n-icon></template>
            刷新检测
          </n-button>
        </div>
      </div>
      <div style="font-size: 12px; color: #666; margin-top: 8px;">
        检测本地工具链安装状态，确保工作流可正常执行
      </div>
    </n-card>

    <!-- Error Alert -->
    <n-alert v-if="checkError" type="error" style="margin-bottom: 20px; border-radius: 12px;">
      检测失败：{{ checkError }}
    </n-alert>

    <!-- Loading State -->
    <div v-if="loading && !result" style="display: flex; justify-content: center; padding: 60px 0;">
      <n-space vertical align="center" :size="12">
        <n-spin :size="32" />
        <span style="color: #888; font-size: 13px;">正在检测本地工具环境...</span>
      </n-space>
    </div>

    <template v-if="result">
      <!-- Installed Tools Section -->
      <div v-if="installedTools.length > 0" style="margin-bottom: 24px;">
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px;">
          <n-icon :size="16" color="#63e2b7"><CheckmarkCircleOutline /></n-icon>
          <span style="font-size: 13px; font-weight: 600; color: #63e2b7;">已安装 ({{ installedTools.length }})</span>
        </div>
        <n-grid :x-gap="16" :y-gap="16" :cols="2">
          <n-gi v-for="tool in installedTools" :key="tool.id">
            <n-card size="small" style="border-radius: 12px; border-left: 3px solid #63e2b7;">
              <div style="display: flex; align-items: center; gap: 12px;">
                <div style="width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 18px; background: rgba(99, 226, 183, 0.1); flex-shrink: 0;">
                  {{ getToolIcon(tool.id) }}
                </div>
                <div style="flex: 1; min-width: 0;">
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 14px; font-weight: 600; color: #eee;">{{ tool.display_name }}</span>
                    <n-tag type="success" size="tiny" round :bordered="false">
                      <template #icon><n-icon><CheckmarkCircleOutline /></n-icon></template>
                      已安装
                    </n-tag>
                    <span style="font-size: 11px; color: #555; margin-left: auto;">{{ tool.latency_ms }}ms</span>
                  </div>
                  <div style="font-size: 11px; color: #aaa; margin-top: 2px;">
                    {{ tool.version }}
                    <span v-if="tool.resolved_path" style="color: #555; margin-left: 8px;" :title="tool.resolved_path">{{ tool.resolved_path }}</span>
                  </div>
                </div>
              </div>
            </n-card>
          </n-gi>
        </n-grid>
      </div>

      <!-- Missing Tools Section -->
      <div v-if="missingTools.length > 0" style="margin-bottom: 24px;">
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px;">
          <n-icon :size="16" color="#f85149"><CloseCircleOutline /></n-icon>
          <span style="font-size: 13px; font-weight: 600; color: #f85149;">未安装 ({{ missingTools.length }})</span>
        </div>
        <n-grid :x-gap="16" :y-gap="16" :cols="2">
          <n-gi v-for="tool in missingTools" :key="tool.id">
            <n-card size="small" style="border-radius: 12px; border-left: 3px solid #f85149;">
              <div style="display: flex; align-items: flex-start; gap: 12px;">
                <div style="width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 18px; background: rgba(248, 81, 73, 0.1); flex-shrink: 0;">
                  {{ getToolIcon(tool.id) }}
                </div>
                <div style="flex: 1; min-width: 0;">
                  <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                    <span style="font-size: 14px; font-weight: 600; color: #eee;">{{ tool.display_name }}</span>
                    <n-tag type="error" size="tiny" round :bordered="false">
                      <template #icon><n-icon><CloseCircleOutline /></n-icon></template>
                      未安装
                    </n-tag>
                  </div>
                  <div v-if="tool.error" style="font-size: 11px; color: #f85149; margin-bottom: 6px;">{{ tool.error }}</div>
                  <div style="padding-top: 6px; border-top: 1px solid rgba(255,255,255,0.06);">
                    <div v-if="tool.install_hint" style="font-size: 11px; font-family: 'Cascadia Code', Consolas, monospace; background: rgba(0,0,0,0.3); padding: 5px 8px; border-radius: 6px; color: #70c0e8;">
                      {{ tool.install_hint }}
                    </div>
                    <div v-if="tool.install_url" style="margin-top: 4px;">
                      <n-button text size="tiny" tag="a" :href="tool.install_url" target="_blank" type="info">
                        <template #icon><n-icon :size="12"><OpenOutline /></n-icon></template>
                        官方文档
                      </n-button>
                    </div>
                  </div>
                </div>
              </div>
            </n-card>
          </n-gi>
        </n-grid>
      </div>

      <!-- Error Tools Section -->
      <div v-if="errorTools.length > 0" style="margin-bottom: 24px;">
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px;">
          <n-icon :size="16" color="#e2a727"><AlertCircleOutline /></n-icon>
          <span style="font-size: 13px; font-weight: 600; color: #e2a727;">检测异常 ({{ errorTools.length }})</span>
        </div>
        <n-grid :x-gap="16" :y-gap="16" :cols="2">
          <n-gi v-for="tool in errorTools" :key="tool.id">
            <n-card size="small" style="border-radius: 12px; border-left: 3px solid #e2a727;">
              <div style="display: flex; align-items: flex-start; gap: 12px;">
                <div style="width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 18px; background: rgba(226, 167, 39, 0.1); flex-shrink: 0;">
                  {{ getToolIcon(tool.id) }}
                </div>
                <div style="flex: 1; min-width: 0;">
                  <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                    <span style="font-size: 14px; font-weight: 600; color: #eee;">{{ tool.display_name }}</span>
                    <n-tag type="warning" size="tiny" round :bordered="false">
                      <template #icon><n-icon><AlertCircleOutline /></n-icon></template>
                      检测异常
                    </n-tag>
                  </div>
                  <div v-if="tool.error" style="font-size: 11px; color: #e2a727;">{{ tool.error }}</div>
                  <div style="padding-top: 6px; border-top: 1px solid rgba(255,255,255,0.06); margin-top: 4px;">
                    <div v-if="tool.install_hint" style="font-size: 11px; font-family: 'Cascadia Code', Consolas, monospace; background: rgba(0,0,0,0.3); padding: 5px 8px; border-radius: 6px; color: #70c0e8;">
                      {{ tool.install_hint }}
                    </div>
                    <div v-if="tool.install_url" style="margin-top: 4px;">
                      <n-button text size="tiny" tag="a" :href="tool.install_url" target="_blank" type="info">
                        <template #icon><n-icon :size="12"><OpenOutline /></n-icon></template>
                        官方文档
                      </n-button>
                    </div>
                  </div>
                </div>
              </div>
            </n-card>
          </n-gi>
        </n-grid>
      </div>

      <!-- Reference Links -->
      <n-card
        size="small"
        style="border-radius: 12px; background: rgba(112, 192, 232, 0.04); border: 1px solid rgba(112, 192, 232, 0.15);"
      >
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 10px;">
          <n-icon :size="16" color="#70c0e8"><BookOutline /></n-icon>
          <span style="font-size: 13px; font-weight: 600; color: #aaa;">参考文档</span>
        </div>
        <div style="display: flex; flex-wrap: wrap; gap: 16px;">
          <n-button
            text size="small" tag="a" type="info"
            href="https://github.com/fengshao1227/ccg-workflow/blob/main/README.zh-CN.md"
            target="_blank"
          >
            <template #icon><n-icon :size="14"><OpenOutline /></n-icon></template>
            CCG Workflow 文档
          </n-button>
          <n-button
            text size="small" tag="a" type="info"
            href="https://github.com/MrNine-666/claude-code-quickstart"
            target="_blank"
          >
            <template #icon><n-icon :size="14"><DownloadOutline /></n-icon></template>
            CCQ 一键安装脚本
          </n-button>
        </div>
      </n-card>
    </template>
  </div>
</template>
