<script setup lang="ts">
import { ref, h } from "vue";
import { 
  darkTheme, NConfigProvider, NMessageProvider, NLayout, NLayoutHeader, NLayoutSider, 
  NLayoutContent, NMenu, NSpace, NButton, NCard, NGradientText, NIcon, NInput, 
  NTimeline, NTimelineItem, NGrid, NGi, NTag
} from "naive-ui";
import { 
  RocketOutline, TerminalOutline, GitCommitOutline, SettingsOutline, PlayOutline, BugOutline 
} from "@vicons/ionicons5";

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

const selectedKey = ref('execute');
const reqText = ref('');
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-layout has-sider style="height: 100vh;">
        <!-- Left Sider -->
        <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="260" show-trigger>
          <div style="padding: 24px; text-align: center;">
            <n-gradient-text type="info" :size="24" style="font-weight: 800; display: flex; align-items: center; justify-content: center; gap: 8px;">
              <n-icon><RocketOutline/></n-icon> vibeCoding
            </n-gradient-text>
            <div style="font-size: 12px; color: #888; margin-top: 6px; letter-spacing: 0.5px;">CCG Workflow Engine</div>
          </div>
          <n-menu v-model:value="selectedKey" :collapsed-width="64" :collapsed-icon-size="22" :options="menuOptions" style="margin-top: 10px;" />
        </n-layout-sider>
        
        <!-- Right Content -->
        <n-layout>
          <!-- Header -->
          <n-layout-header bordered style="padding: 16px 24px; display: flex; justify-content: space-between; align-items: center;">
            <div style="font-size: 16px; font-weight: 500; font-family: monospace;">工作区 / {{ selectedKey.toUpperCase() }}</div>
            <n-space>
              <n-button strong secondary type="info" size="small">极速前端 (Gemini Vue)</n-button>
              <n-button strong secondary type="tertiary" size="small">极速后端 (Codex Algo)</n-button>
            </n-space>
          </n-layout-header>
          
          <!-- Main Content -->
          <n-layout-content style="padding: 24px; background: #101014;">
            <div v-if="selectedKey === 'execute'" style="height: 100%">
              <n-grid :x-gap="24" :y-gap="24" :cols="2" style="height: 100%">
                <!-- Card 1: Requirement -->
                <n-gi>
                  <n-card title="需求解析与派发 (Phase 1)" size="medium" bordered style="height: 100%; border-radius: 12px;">
                    <template #header-extra>
                      <n-tag type="info" size="small">/ccg:plan</n-tag>
                    </template>
                    <n-space vertical :size="20">
                      <n-input
                        v-model:value="reqText"
                        type="textarea"
                        placeholder="描述您的需求，例如：增加登录界面的找回密码功能，需要邮箱验证..."
                        :autosize="{ minRows: 8, maxRows: 12 }"
                        style="font-family: inherit;"
                      />
                      <n-button type="primary" block size="large">
                        <template #icon>
                          <n-icon><RocketOutline /></n-icon>
                        </template>
                        生成 CCG 执行计划
                      </n-button>
                    </n-space>
                  </n-card>
                </n-gi>

                <!-- Card 2: Execution Tree -->
                <n-gi>
                  <n-card title="引擎执行树与进度跟踪 (Phase 2)" size="medium" bordered style="height: 100%; border-radius: 12px;">
                    <template #header-extra>
                      <n-tag type="success" size="small">/ccg:execute</n-tag>
                    </template>
                    <div style="padding: 10px 0;">
                      <n-timeline>
                        <n-timeline-item type="success" title="需求拆解完毕" content="已生成 UI_Spec.md 和 Logic_Spec.md" time="刚刚" />
                        <n-timeline-item type="info" title="等待用户确认" time="现在">
                          <template #default>
                             <div style="margin-top: 10px;">
                               <n-button size="medium" type="primary">批准并开始构建 (Approve & Build)</n-button>
                             </div>
                          </template>
                        </n-timeline-item>
                        <n-timeline-item type="default" title="Codex 编写核心逻辑" content="队列中挂起..." />
                        <n-timeline-item type="default" title="Gemini 构建前端视图" content="队列中挂起..." />
                        <n-timeline-item type="default" title="Claude 交叉审查" content="等待前置任务完毕..." />
                      </n-timeline>
                    </div>
                  </n-card>
                </n-gi>
              </n-grid>
            </div>
            
            <div v-else style="display:flex; justify-content:center; align-items:center; height: 100%; color: #666; font-size: 16px;">
              [ {{ selectedKey }} ] 模块规划中...
            </div>
          </n-layout-content>
        </n-layout>
      </n-layout>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
body {
  margin: 0;
  padding: 0;
  font-family: "Helvetica Neue", Helvetica, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "微软雅黑", Arial, sans-serif;
  background-color: #101014;
}
</style>