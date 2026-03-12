<script setup lang="ts">
import { ref, onMounted, watch, h } from "vue";
import { NCard, NDataTable, NTag, NButton, NIcon, NEmpty } from "naive-ui";
import { DocumentTextOutline, RefreshOutline } from "@vicons/ionicons5";
import { useWorkspaceStore, type WorkflowRun } from "../../store/workspace";

const workspaceStore = useWorkspaceStore();
const historyData = ref<WorkflowRun[]>([]);
const loading = ref(false);

const columns = [
  { title: "运行 ID", key: "id", width: 120, ellipsis: { tooltip: true } },
  { 
    title: "状态", 
    key: "status",
    width: 100,
    render(row: WorkflowRun) {
      let type: 'default' | 'success' | 'warning' | 'error' | 'info' = 'default';
      if (row.status === 'Success') type = 'success';
      if (row.status === 'Running') type = 'info';
      if (row.status === 'Failed') type = 'error';
      return h(NTag, { type: type, size: 'small' }, { default: () => row.status });
    }
  },
  { title: "开始时间", key: "start_time" },
  { title: "结束时间", key: "end_time" },
  {
    title: "操作",
    key: "actions",
    width: 120,
    render() {
      return h(
        NButton,
        { size: 'small', secondary: true, type: 'info' },
        { default: () => '查看日志' }
      );
    }
  }
];

import { invoke } from '@tauri-apps/api/core';

async function loadHistory() {
  if (!workspaceStore.currentWorkspaceId) {
    historyData.value = [];
    return;
  }
  loading.value = true;
  
  try {
    const runs = await invoke<WorkflowRun[]>('get_workflow_runs', { 
      workspaceId: workspaceStore.currentWorkspaceId 
    });
    historyData.value = runs;
  } catch (err) {
    console.warn("Failed to fetch history (fallback to mock):", err);
    historyData.value = [
      { id: "run-001", workspace_id: workspaceStore.currentWorkspaceId!, status: 'Success', start_time: new Date(Date.now() - 3600000).toISOString(), end_time: new Date().toISOString() },
      { id: "run-002", workspace_id: workspaceStore.currentWorkspaceId!, status: 'Failed', start_time: new Date(Date.now() - 7200000).toISOString(), end_time: new Date(Date.now() - 7100000).toISOString() }
    ];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadHistory();
});

watch(() => workspaceStore.currentWorkspaceId, () => {
  loadHistory();
});

</script>

<template>
  <div>
    <n-card title="执行历史 (Execution History)" style="border-radius: 12px; margin-bottom: 20px;">
      <template #header-extra>
        <n-button circle size="small" @click="loadHistory" :loading="loading">
          <template #icon><n-icon><RefreshOutline/></n-icon></template>
        </n-button>
      </template>
      
      <div v-if="!workspaceStore.currentWorkspaceId" style="padding: 40px 0;">
        <n-empty description="请先选择项目以查看历史记录">
          <template #icon>
            <n-icon><DocumentTextOutline /></n-icon>
          </template>
        </n-empty>
      </div>
      
      <div v-else>
        <n-data-table
          :columns="columns"
          :data="historyData"
          :loading="loading"
          :bordered="false"
          :single-line="false"
          style="margin-top: 10px;"
        />
      </div>
    </n-card>
  </div>
</template>
