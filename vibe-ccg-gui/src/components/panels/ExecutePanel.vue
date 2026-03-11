<script setup lang="ts">
import { ref } from "vue";
import { NGrid, NGi, NCard, NTag, NSpace, NInput, NButton, NIcon, NTimeline, NTimelineItem } from "naive-ui";
import { RocketOutline } from "@vicons/ionicons5";

const reqText = ref('');
</script>

<template>
  <div style="height: 100%">
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
</template>
