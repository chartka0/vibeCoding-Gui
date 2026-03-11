<script setup lang="ts">
import { h } from "vue";
import { NLayoutSider, NGradientText, NIcon, NMenu } from "naive-ui";
import { 
  RocketOutline, TerminalOutline, GitCommitOutline, SettingsOutline, PlayOutline, BugOutline 
} from "@vicons/ionicons5";

const props = defineProps<{
  selectedKey: string;
}>();

const emit = defineEmits<{
  (e: 'update:selectedKey', key: string): void
}>();

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const menuOptions = [
  { label: "执行引擎 (Execute)", key: "execute", icon: renderIcon(PlayOutline) },
  { label: "质量门禁 (Quality)", key: "quality", icon: renderIcon(BugOutline) },
  { label: "交付版本 (Release)", key: "release", icon: renderIcon(GitCommitOutline) },
  { label: "终端控制 (Terminal)", key: "terminal", icon: renderIcon(TerminalOutline) },
  { label: "控制面板 (Settings)", key: "settings", icon: renderIcon(SettingsOutline) }
];

function handleUpdateValue(key: string) {
  emit('update:selectedKey', key);
}
</script>

<template>
  <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="260" show-trigger>
    <div style="padding: 24px; text-align: center;">
      <n-gradient-text type="info" :size="24" style="font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px;">
        <n-icon><RocketOutline/></n-icon> vibeCoding
      </n-gradient-text>
      <div style="font-size: 12px; color: #888; margin-top: 6px; letter-spacing: 0.5px;">CCG Workflow Engine</div>
    </div>
    <n-menu 
      :value="selectedKey" 
      @update:value="handleUpdateValue"
      :collapsed-width="64" 
      :collapsed-icon-size="22" 
      :options="menuOptions" 
      style="margin-top: 10px;" 
    />
  </n-layout-sider>
</template>
