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
    try {
      const data = await invoke<Workspace[]>('get_workspaces');
      workspaces.value = data;
    } catch (err) {
      console.error("Failed to fetch workspaces from Tauri backend:", err);
      // Fallback or leave as empty if no backend
    }
  };

  const addWorkspace = async (name: string, path: string): Promise<Workspace | null> => {
    try {
      const newWs = await invoke<Workspace>('add_workspace', { name, path });
      workspaces.value.unshift(newWs); // Add to top
      return newWs;
    } catch (err) {
      console.error("Failed to add workspace via Tauri", err);
      return null;
    }
  };

  const currentWorkspace = computed(() => 
    workspaces.value.find(w => w.id === currentWorkspaceId.value)
  );

  return {
    workspaces,
    currentWorkspaceId,
    currentWorkspace,
    fetchWorkspaces,
    addWorkspace
  };
});
