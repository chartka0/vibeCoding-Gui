<script setup lang="ts">
import { h } from "vue";
import { NLayoutSider, NGradientText, NIcon, NMenu } from "naive-ui";
import type { MenuOption, MenuGroupOption } from "naive-ui";
import {
  RocketOutline, PlayOutline, BugOutline,
  GitCommitOutline, SettingsOutline, TerminalOutline,
  FlashOutline, CodeSlashOutline, SearchOutline,
  CheckmarkCircleOutline, ShieldCheckmarkOutline,
  TimerOutline, GitBranchOutline
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

const menuOptions: (MenuOption | MenuGroupOption)[] = [
  {
    type: 'group',
    label: '工作流',
    key: 'group-workflow',
    children: [
      { label: "完整心流", key: "workflow", icon: renderIcon(RocketOutline) },
      { label: "规划面板", key: "plan", icon: renderIcon(SearchOutline) },
      { label: "构建监控", key: "build", icon: renderIcon(FlashOutline) },
    ]
  },
  {
    type: 'group',
    label: '质量门禁',
    key: 'group-quality',
    children: [
      { label: "代码审查", key: "review", icon: renderIcon(CodeSlashOutline) },
      { label: "诊断优化", key: "diagnostics", icon: renderIcon(BugOutline) },
      { label: "测试面板", key: "test", icon: renderIcon(CheckmarkCircleOutline) },
    ]
  },
  {
    type: 'group',
    label: '规范驱动',
    key: 'group-spec',
    children: [
      { label: "约束画板", key: "spec", icon: renderIcon(ShieldCheckmarkOutline) },
    ]
  },
  {
    type: 'group',
    label: '版本控制',
    key: 'group-git',
    children: [
      { label: "智能提交", key: "commit", icon: renderIcon(GitCommitOutline) },
      { label: "时光机", key: "rollback", icon: renderIcon(TimerOutline) },
    ]
  },
  {
    type: 'group',
    label: '系统',
    key: 'group-system',
    children: [
      { label: "设置", key: "settings", icon: renderIcon(SettingsOutline) },
    ]
  }
];

function handleUpdateValue(key: string) {
  emit('update:selectedKey', key);
}
</script>

<template>
  <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="220" show-trigger>
    <div style="padding: 20px 16px 12px; text-align: center;">
      <n-gradient-text type="info" :size="22" style="font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 6px;">
        <n-icon :size="20"><RocketOutline/></n-icon> vibeCoding
      </n-gradient-text>
      <div style="font-size: 11px; color: #666; margin-top: 4px; letter-spacing: 0.5px;">CCG Workflow Engine</div>
    </div>
    <n-menu
      :value="selectedKey"
      @update:value="handleUpdateValue"
      :collapsed-width="64"
      :collapsed-icon-size="20"
      :options="menuOptions"
    />
  </n-layout-sider>
</template>
