<script setup lang="ts">
import { NLayoutHeader, NSpace, NButton, NIcon, NTag } from "naive-ui";
import { FlashOutline, LogoVue, CodeSlashOutline } from "@vicons/ionicons5";

defineProps<{
  title: string;
  subtitle?: string;
}>();
</script>

<template>
  <n-layout-header bordered style="padding: 12px 24px; display: flex; justify-content: space-between; align-items: center; background: #18181c;">
    <div style="display: flex; align-items: center; gap: 12px;">
      <span style="font-size: 15px; font-weight: 600; font-family: monospace; color: #e0e0e0;">{{ title }}</span>
      <n-tag v-if="subtitle" size="small" :bordered="false" type="info">{{ subtitle }}</n-tag>
    </div>
    <n-space :size="8">
      <n-button strong secondary type="info" size="small">
        <template #icon><n-icon><LogoVue /></n-icon></template>
        极速前端 (Gemini)
      </n-button>
      <n-button strong secondary type="warning" size="small">
        <template #icon><n-icon><CodeSlashOutline /></n-icon></template>
        极速后端 (Codex)
      </n-button>
    </n-space>
  </n-layout-header>
</template>
