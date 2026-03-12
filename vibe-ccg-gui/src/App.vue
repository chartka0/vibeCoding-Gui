<script setup lang="ts">
import { ref } from "vue";
import { darkTheme, NConfigProvider, NMessageProvider } from "naive-ui";

import Overview from "./views/Overview.vue";
import WorkspaceView from "./views/WorkspaceView.vue";

type PageState = 'overview' | 'workspace';

const activePage = ref<PageState>('overview');
const activeWorkspaceId = ref<string>('');

function handleEnterWorkspace(id: string) {
  activeWorkspaceId.value = id;
  activePage.value = 'workspace';
}

function handleBackToOverview() {
  activeWorkspaceId.value = '';
  activePage.value = 'overview';
}
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <transition name="fade" mode="out-in">
        <Overview 
          v-if="activePage === 'overview'" 
          @enter-workspace="handleEnterWorkspace" 
        />
        <WorkspaceView 
          v-else-if="activePage === 'workspace' && activeWorkspaceId"
          :workspace-id="activeWorkspaceId"
          @back-to-overview="handleBackToOverview"
        />
      </transition>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: #101014;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
