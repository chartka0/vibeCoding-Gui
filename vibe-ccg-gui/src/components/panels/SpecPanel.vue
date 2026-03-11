<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NTag, NSpace, NSwitch, NGrid, NGi,
  NInput, NDynamicTags, NAlert, NDescriptions, NDescriptionsItem
} from "naive-ui";
import {
  AddOutline, ShieldCheckmarkOutline, CheckmarkCircleOutline, CloseCircleOutline
} from "@vicons/ionicons5";

const opsxEnabled = ref(false);

// Mock constraints
const constraints = ref([
  '所有 API 必须返回统一 ResponseDTO 格式',
  '密码最少 8 位，包含大小写和数字',
  '登录失败超过 5 次锁定 30 分钟',
  '所有数据库操作必须在事务中执行',
  '前端表单必须有客户端 + 服务端双重校验',
]);

// Mock validation results
const validations = ref([
  { rule: 'ResponseDTO 格式', status: 'pass', detail: '所有 3 个 API 端点均符合' },
  { rule: '密码复杂度', status: 'pass', detail: 'bcrypt 哈希 + 前端正则校验' },
  { rule: '登录失败锁定', status: 'fail', detail: '缺少锁定计数器实现' },
  { rule: '事务保护', status: 'pass', detail: '已使用 @Transactional 装饰器' },
  { rule: '双重校验', status: 'warn', detail: '服务端校验存在，客户端部分缺失' },
]);
</script>

<template>
  <div>
    <!-- OPSX Mode Toggle -->
    <n-card size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <div style="display: flex; align-items: center; gap: 12px;">
          <n-icon :size="20" color="#63e2b7"><ShieldCheckmarkOutline /></n-icon>
          <span style="font-weight: 600;">OPSX 强约束模式</span>
          <n-tag v-if="opsxEnabled" type="success" size="small" :bordered="false">已启用</n-tag>
          <n-tag v-else type="default" size="small" :bordered="false">未启用</n-tag>
        </div>
        <n-switch v-model:value="opsxEnabled" />
      </div>
      <div style="color: #888; font-size: 12px; margin-top: 8px;">
        开启后，必须所有约束校验通过才能进入编码阶段。对应 /ccg:spec-plan 和 /ccg:spec-impl 命令。
      </div>
    </n-card>

    <n-grid :x-gap="20" :cols="2">
      <!-- Left: Constraint Definition -->
      <n-gi>
        <n-card title="约束规则定义" size="medium" style="border-radius: 12px;">
          <template #header-extra>
            <n-tag type="info" size="small" :bordered="false">/ccg:spec-research</n-tag>
          </template>

          <n-space vertical :size="12">
            <n-dynamic-tags v-model:value="constraints" type="info" />

            <n-alert type="info" :bordered="false" style="border-radius: 6px; margin-top: 8px;">
              约束规则从需求文档中自动提取，也可手动添加、删除。每条约束将作为代码生成和审查的硬性校验标准。
            </n-alert>

            <n-button type="info" block style="border-radius: 8px; margin-top: 8px;">
              <template #icon><n-icon><AddOutline /></n-icon></template>
              从需求重新提取约束
            </n-button>
          </n-space>
        </n-card>
      </n-gi>

      <!-- Right: Validation Status -->
      <n-gi>
        <n-card title="约束校验状态" size="medium" style="border-radius: 12px;">
          <template #header-extra>
            <n-tag type="warning" size="small" :bordered="false">4/5 通过</n-tag>
          </template>

          <n-space vertical :size="12">
            <div
              v-for="(v, i) in validations"
              :key="i"
              style="display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; background: #18181c; border-radius: 8px;"
            >
              <n-icon
                :size="18"
                :color="v.status === 'pass' ? '#63e2b7' : v.status === 'fail' ? '#e88080' : '#f2c97d'"
                style="margin-top: 2px; flex-shrink: 0;"
              >
                <CheckmarkCircleOutline v-if="v.status === 'pass'" />
                <CloseCircleOutline v-else />
              </n-icon>
              <div>
                <div style="font-weight: 500; font-size: 13px;">{{ v.rule }}</div>
                <div style="color: #888; font-size: 12px; margin-top: 2px;">{{ v.detail }}</div>
              </div>
              <n-tag
                :type="v.status === 'pass' ? 'success' : v.status === 'fail' ? 'error' : 'warning'"
                size="tiny"
                :bordered="false"
                style="margin-left: auto; flex-shrink: 0;"
              >
                {{ v.status === 'pass' ? 'PASS' : v.status === 'fail' ? 'FAIL' : 'WARN' }}
              </n-tag>
            </div>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
  </div>
</template>
