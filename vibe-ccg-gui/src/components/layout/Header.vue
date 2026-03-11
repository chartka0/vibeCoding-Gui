<script setup lang="ts">
import { NLayoutHeader, NSpace, NButton } from "naive-ui";

defineProps<{
  title: string;
}>();
</script>

<template>
  <n-layout-header bordered style="padding: 16px 24px; display: flex; justify-content: space-between; align-items: center;">
    <div style="font-size: 16px; font-weight: 500; font-family: monospace;">工作区 / {{ title }}</div>
    <n-space>
      <n-button strong secondary type="info" size="small">极速前端 (Gemini Vue)</n-button>
      <n-button strong secondary type="tertiary" size="small">极速后端 (Codex Algo)</n-button>
    </n-space>
  </n-layout-header>
</template>
