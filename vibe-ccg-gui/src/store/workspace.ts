import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface Workspace {
  id: string;
  name: string;
  path: string;
  created_at: string;
}

export interface WorkflowRun {
  id: string;
  workspace_id: string;
  status: 'Running' | 'Success' | 'Failed' | 'Cancelled';
  logs_path?: string;
  start_time: string;
  end_time?: string;
}

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([]);
  const currentWorkspaceId = ref<string | null>(null);

  // Load all workspaces from DB
  const fetchWorkspaces = async () => {
    // In actual implementation, we'll invoke the Rust db select command
    // const data = await invoke<Workspace[]>('get_workspaces');
    // workspaces.value = data;
    // if (!currentWorkspaceId.value && data.length > 0) {
    //   currentWorkspaceId.value = data[0].id;
    // }
  };

  const currentWorkspace = computed(() => 
    workspaces.value.find(w => w.id === currentWorkspaceId.value)
  );

  return {
    workspaces,
    currentWorkspaceId,
    currentWorkspace,
    fetchWorkspaces
  };
});
