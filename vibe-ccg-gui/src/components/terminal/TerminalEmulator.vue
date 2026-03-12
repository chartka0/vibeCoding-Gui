<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, defineExpose } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';

const terminalRef = ref<HTMLDivElement>();
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlistenPtyOutput: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;
const isReady = ref(false);

const emit = defineEmits<{
  (e: 'mode-change', mode: string): void;
}>();

async function initTerminal() {
  if (!terminalRef.value) return;

  term = new Terminal({
    theme: {
      background: '#0d1117',
      foreground: '#c9d1d9',
      cursor: '#58a6ff',
      cursorAccent: '#0d1117',
      selectionBackground: '#264f78',
      black: '#0d1117',
      red: '#ff7b72',
      green: '#3fb950',
      yellow: '#d29922',
      blue: '#58a6ff',
      magenta: '#bc8cff',
      cyan: '#39d353',
      white: '#c9d1d9',
    },
    fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
    fontSize: 14,
    lineHeight: 1.4,
    cursorBlink: true,
    scrollback: 5000,
    convertEol: true,
  });

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(terminalRef.value);

  await nextTick();
  fitAddon.fit();

  // Spawn PTY backend
  const dims = fitAddon.proposeDimensions();
  await invoke('spawn_pty', {
    cols: dims?.cols ?? 80,
    rows: dims?.rows ?? 24,
  });

  // Listen for PTY output from Rust
  unlistenPtyOutput = await listen<{ data: string }>('pty-output', (event) => {
    if (term) {
      term.write(event.payload.data);

      // Detect [模式：X] tags in the output to drive the step indicator
      const modeMatch = event.payload.data.match(/\[模式[：:]\s*(.+?)\]/);
      if (modeMatch) {
        emit('mode-change', modeMatch[1]);
      }
    }
  });

  // Forward user input to PTY
  term.onData(async (data) => {
    try {
      await invoke('write_to_pty', { data });
    } catch (e) {
      console.error('Failed to write to PTY:', e);
    }
  });

  // Handle resize
  resizeObserver = new ResizeObserver(() => {
    if (fitAddon && term) {
      fitAddon.fit();
      const dims = fitAddon.proposeDimensions();
      if (dims) {
        invoke('resize_pty', { cols: dims.cols, rows: dims.rows }).catch(() => {});
      }
    }
  });
  resizeObserver.observe(terminalRef.value);

  isReady.value = true;
}

/** Execute a command string in the terminal */
async function executeCommand(cmd: string) {
  if (!isReady.value) return;
  try {
    await invoke('write_to_pty', { data: cmd + '\r' });
  } catch (e) {
    console.error('Failed to execute command:', e);
  }
}

defineExpose({ executeCommand });

onMounted(() => {
  initTerminal();
});

onUnmounted(() => {
  if (unlistenPtyOutput) unlistenPtyOutput();
  if (resizeObserver) resizeObserver.disconnect();
  if (term) term.dispose();
});
</script>

<template>
  <div
    ref="terminalRef"
    style="
      width: 100%;
      height: 100%;
      min-height: 300px;
      border-radius: 8px;
      overflow: hidden;
      background: #0d1117;
    "
  />
</template>
