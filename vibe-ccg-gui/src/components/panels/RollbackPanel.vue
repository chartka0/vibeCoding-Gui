<script setup lang="ts">
import { ref } from "vue";
import {
  NCard, NButton, NIcon, NTag, NSpace, NTimeline, NTimelineItem,
  NPopconfirm
} from "naive-ui";
import {
  GitCommitOutline, ArrowUndoOutline, GitBranchOutline
} from "@vicons/ionicons5";

// Mock git history
const gitHistory = ref([
  {
    hash: '9f7c4d1',
    message: 'feat(gui): 重构侧边栏导航与路由框架',
    author: 'developer',
    time: '2 分钟前',
    type: 'info' as const,
    tags: ['HEAD', 'main'],
  },
  {
    hash: '81a65eb',
    message: 'feat: 项目组件化',
    author: 'developer',
    time: '1 小时前',
    type: 'success' as const,
    tags: [],
  },
  {
    hash: '1bc018e',
    message: 'feat: 项目初始化',
    author: 'developer',
    time: '3 小时前',
    type: 'success' as const,
    tags: [],
  },
  {
    hash: '2022f01',
    message: 'feat: 命令行与UI映射关系',
    author: 'developer',
    time: '5 小时前',
    type: 'success' as const,
    tags: [],
  },
  {
    hash: '5045b70',
    message: 'v0.5',
    author: 'developer',
    time: '1 天前',
    type: 'default' as const,
    tags: ['v0.5'],
  },
]);
</script>

<template>
  <div>
    <!-- Info Card -->
    <n-card size="small" style="border-radius: 12px; margin-bottom: 20px;">
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <div style="display: flex; align-items: center; gap: 10px;">
          <n-icon :size="20" color="#70c0e8"><GitBranchOutline /></n-icon>
          <span style="font-weight: 600;">当前分支：main</span>
          <n-tag type="info" size="small" :bordered="false">{{ gitHistory.length }} commits</n-tag>
        </div>
        <n-space>
          <n-button size="small" secondary>刷新历史</n-button>
        </n-space>
      </div>
    </n-card>

    <!-- Timeline -->
    <n-card title="提交历史" size="medium" style="border-radius: 12px;">
      <n-timeline size="large" style="padding: 8px 0;">
        <n-timeline-item
          v-for="commit in gitHistory"
          :key="commit.hash"
          :type="commit.type"
        >
          <template #header>
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
              <span style="font-weight: 600; font-size: 14px;">{{ commit.message }}</span>
              <n-tag v-for="tag in commit.tags" :key="tag" size="tiny" :bordered="false"
                :type="tag === 'HEAD' ? 'info' : tag === 'main' ? 'success' : 'warning'">
                {{ tag }}
              </n-tag>
            </div>
          </template>
          <template #default>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 4px;">
              <div style="display: flex; align-items: center; gap: 16px;">
                <span style="font-family: monospace; font-size: 12px; color: #f2c97d;">{{ commit.hash }}</span>
                <span style="font-size: 12px; color: #888;">{{ commit.author }}</span>
                <span style="font-size: 12px; color: #666;">{{ commit.time }}</span>
              </div>
              <n-popconfirm
                :positive-text="'确认回滚'"
                :negative-text="'取消'"
              >
                <template #trigger>
                  <n-button size="tiny" secondary type="warning">
                    <template #icon><n-icon><ArrowUndoOutline /></n-icon></template>
                    回滚到此版本
                  </n-button>
                </template>
                确认回滚到 {{ commit.hash }}？此操作将撤销之后的所有提交。
              </n-popconfirm>
            </div>
          </template>
        </n-timeline-item>
      </n-timeline>
    </n-card>
  </div>
</template>
