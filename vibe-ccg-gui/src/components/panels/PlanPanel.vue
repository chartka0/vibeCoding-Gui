<script setup lang="ts">
import { ref } from "vue";
import {
  NGrid, NGi, NCard, NInput, NButton, NIcon, NSpace, NTag,
  NUpload, NTimeline, NTimelineItem, NCollapse, NCollapseItem, NBadge
} from "naive-ui";
import {
  RocketOutline, CloudUploadOutline, DocumentTextOutline,
  CheckmarkCircleOutline
} from "@vicons/ionicons5";

const reqText = ref('');

// Mock plan data
const mockPlanTasks = [
  {
    title: 'UI_Spec.md - 前端组件规范',
    status: 'success' as const,
    items: [
      '登录表单组件 (NForm + NInput + NButton)',
      '密码强度指示器 (NProgress)',
      '邮箱验证弹窗 (NModal + NInputNumber)',
    ]
  },
  {
    title: 'Logic_Spec.md - 后端逻辑规范',
    status: 'info' as const,
    items: [
      '用户认证 API (/api/auth/login)',
      '邮箱验证码发送服务',
      '密码重置 Token 管理',
    ]
  },
  {
    title: 'Test_Spec.md - 验收标准',
    status: 'warning' as const,
    items: [
      '正常登录流程 E2E 测试',
      '密码错误重试限制测试',
      '验证码过期场景测试',
    ]
  }
];
</script>

<template>
  <div style="height: 100%;">
    <n-grid :x-gap="20" :cols="2" style="height: 100%;">
      <!-- Left: Requirement Input -->
      <n-gi>
        <n-card title="需求输入" size="medium" style="border-radius: 12px; height: 100%;">
          <template #header-extra>
            <n-tag type="info" size="small" :bordered="false">Phase 1 - Context</n-tag>
          </template>
          <n-space vertical :size="16">
            <n-input
              v-model:value="reqText"
              type="textarea"
              placeholder="描述您的需求，例如：增加登录界面的找回密码功能，需要邮箱验证..."
              :autosize="{ minRows: 8, maxRows: 14 }"
              style="font-family: inherit;"
            />

            <!-- Upload Zone -->
            <n-upload
              action="#"
              :default-upload="false"
              multiple
              directory-dnd
              style="border-radius: 8px;"
            >
              <n-card
                style="cursor: pointer; border: 1px dashed #444; border-radius: 8px; text-align: center; padding: 16px 0;"
                :bordered="false"
                content-style="padding: 12px;"
              >
                <n-icon :size="32" color="#666"><CloudUploadOutline /></n-icon>
                <div style="color: #888; font-size: 13px; margin-top: 6px;">
                  拖拽设计草图 / 参考截图到此处
                </div>
                <div style="color: #555; font-size: 11px; margin-top: 4px;">
                  支持 PNG / JPG / Markdown / PDF
                </div>
              </n-card>
            </n-upload>

            <n-button type="info" block size="large" style="border-radius: 8px;">
              <template #icon><n-icon><RocketOutline /></n-icon></template>
              生成 CCG 执行计划
            </n-button>
          </n-space>
        </n-card>
      </n-gi>

      <!-- Right: Generated Plan -->
      <n-gi>
        <n-card title="实施计划预览" size="medium" style="border-radius: 12px; height: 100%;">
          <template #header-extra>
            <n-tag type="success" size="small" :bordered="false">Phase 2 - Creation</n-tag>
          </template>

          <n-collapse default-expanded-names="0" style="margin-bottom: 16px;">
            <n-collapse-item
              v-for="(spec, index) in mockPlanTasks"
              :key="index"
              :title="spec.title"
              :name="String(index)"
            >
              <template #header-extra>
                <n-badge :value="spec.items.length" :type="spec.status" />
              </template>
              <n-timeline style="padding-left: 4px;">
                <n-timeline-item
                  v-for="(item, i) in spec.items"
                  :key="i"
                  :type="spec.status"
                  :title="item"
                />
              </n-timeline>
            </n-collapse-item>
          </n-collapse>

          <!-- Approve Button -->
          <div style="border-top: 1px solid #2a2a2e; padding-top: 16px;">
            <n-button type="primary" block size="large" style="border-radius: 8px; font-size: 15px; height: 44px;">
              <template #icon><n-icon><CheckmarkCircleOutline /></n-icon></template>
              批准并开始构建 (Approve & Build)
            </n-button>
          </div>
        </n-card>
      </n-gi>
    </n-grid>
  </div>
</template>
