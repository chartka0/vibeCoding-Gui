<script setup lang="ts">
import { onMounted } from 'vue';
import { NCard, NGradientText, NButton, NIcon, NEmpty, NBadge, NSpace } from 'naive-ui';
import { FolderOpenOutline, RocketOutline, FlashOutline } from '@vicons/ionicons5';
import { useWorkspaceStore } from '../store/workspace';

import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';

const workspaceStore = useWorkspaceStore();

const emit = defineEmits<{
  (e: 'enter-workspace', id: string): void
}>();

onMounted(() => {
  workspaceStore.fetchWorkspaces();
});

function enterProject(id: string) {
  workspaceStore.currentWorkspaceId = id;
  emit('enter-workspace', id);
}

async function openExistingProject() {
  try {
    const selectedPath = await open({
      title: '选择已有项目文件夹',
      directory: true,
      multiple: false
    });
    
    if (selectedPath && typeof selectedPath === 'string') {
      const folderName = selectedPath.split('\\').pop() || selectedPath.split('/').pop() || 'Unknown Project';
      
      const newWorkspace = await workspaceStore.addWorkspace(folderName, selectedPath);
      
      if (newWorkspace) {
        enterProject(newWorkspace.id);
      }
    }
  } catch (err) {
    console.error("对话框打开失败(如果在非桌面环境会报错):", err);
  }
}

async function openFolder(path: string, event: MouseEvent) {
  event.stopPropagation();
  try {
    await openPath(path);
  } catch (err) {
    console.error('打开目录失败:', err);
  }
}

// Placeholder: running state will be driven by store in future
function isRunning(_id: string) {
  return false;
}
</script>

<template>
  <div style="padding: 40px; max-width: 1200px; margin: 0 auto;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 40px;">
      <div>
        <n-gradient-text type="info" :size="32" style="font-weight: 800; display: flex; align-items: center; gap: 12px;">
          <n-icon :size="28"><RocketOutline/></n-icon> vibeCoding
        </n-gradient-text>
        <div style="color: #888; margin-top: 8px;">选择一个项目进入工作流</div>
      </div>
      
      <n-space>
        <n-button type="default" size="large" @click="openExistingProject">
          <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
          打开本地项目
        </n-button>
      </n-space>
    </div>

    <div v-if="workspaceStore.workspaces.length === 0" style="padding: 60px 0;">
      <n-empty description="暂无项目记录。请点击右上角打开本地已有项目。">
        <template #icon>
          <n-icon><FolderOpenOutline /></n-icon>
        </template>
      </n-empty>
    </div>

    <div v-else style="display: flex; flex-wrap: wrap; gap: 24px;">
      <div v-for="ws in workspaceStore.workspaces" :key="ws.id" style="width: 360px; flex-shrink: 0;">
        <n-badge :show="isRunning(ws.id)" dot type="success" :offset="[-10, 10]" style="width: 100%; height: 100%;">
          <n-card 
            hoverable 
            style="border-radius: 12px; cursor: pointer; height: 100%; transition: transform 0.2s, box-shadow 0.2s;"
            @click="enterProject(ws.id)"
          >
            <div style="display: flex; align-items: flex-start; gap: 16px;">
              <div style="background: rgba(112, 192, 232, 0.1); padding: 14px; border-radius: 12px; display: flex; align-items: center; justify-content: center;">
                <n-icon :size="28" color="#70c0e8"><FolderOpenOutline /></n-icon>
              </div>
              <div style="flex: 1; min-width: 0;">
                <div style="font-size: 16px; font-weight: 600; color: #eee; margin-bottom: 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                  {{ ws.name }}
                </div>
                <div style="font-size: 12px; color: #888; margin-bottom: 16px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" :title="ws.path">
                  {{ ws.path }}
                </div>
                
                <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid rgba(255, 255, 255, 0.08); padding-top: 12px;">
                  <div style="font-size: 12px; color: #666;">
                    创建于: {{ new Date(ws.created_at).toLocaleDateString() }}
                  </div>
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <div v-if="isRunning(ws.id)" style="display: flex; align-items: center; gap: 4px; color: #63e2b7; font-size: 12px;">
                      <n-icon><FlashOutline /></n-icon> 运行中
                    </div>
                    <n-button size="tiny" quaternary @click="openFolder(ws.path, $event)" title="在资源管理器中打开">
                      <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
                    </n-button>
                  </div>
                </div>
              </div>
            </div>
          </n-card>
        </n-badge>
      </div>
    </div>
  </div>
</template>

<style scoped>
.n-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.2) !important;
  border-color: #70c0e8;
}
</style>
