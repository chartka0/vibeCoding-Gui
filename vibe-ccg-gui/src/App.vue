<script setup lang="ts">
import { ref } from "vue";
import { darkTheme, NConfigProvider, NMessageProvider, NLayout, NLayoutContent } from "naive-ui";

import Sidebar from "./components/layout/Sidebar.vue";
import Header from "./components/layout/Header.vue";
import ExecutePanel from "./components/panels/ExecutePanel.vue";

const selectedKey = ref('execute');
</script>

<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-layout has-sider style="height: 100vh;">
        <!-- Left Sider -->
        <Sidebar v-model:selectedKey="selectedKey" />
        
        <!-- Right Content -->
        <n-layout>
          <!-- Header -->
          <Header :title="selectedKey.toUpperCase()" />
          
          <!-- Main Content -->
          <n-layout-content style="padding: 24px; background: #101014;">
            <ExecutePanel v-if="selectedKey === 'execute'" />
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
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: #101014;
}
</style>