<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const open = ref(false);
const remember = ref(false);
const busy = ref(false);

let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  unlisten = await listen("close-prompt", () => {
    remember.value = false;
    open.value = true;
  });
});

onUnmounted(() => {
  unlisten?.();
  unlisten = null;
});

async function choose(action: "tray" | "quit" | "cancel") {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke("resolve_window_close", {
      action,
      remember: action !== "cancel" && remember.value,
    });
    open.value = false;
  } catch {
    open.value = false;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div
    v-if="open"
    class="close-mask"
    role="dialog"
    aria-modal="true"
    aria-label="关闭窗口"
  >
    <div class="close-card">
      <h1>关闭窗口后如何处理？</h1>
      <p class="close-lead">
        可最小化到系统托盘在后台运行，或直接退出程序。
      </p>
      <label class="close-remember">
        <input v-model="remember" type="checkbox" />
        <span>不再提示，记住本次选择</span>
      </label>
      <div class="close-actions">
        <button type="button" class="primary" :disabled="busy" @click="choose('tray')">
          最小化到托盘
        </button>
        <button type="button" :disabled="busy" @click="choose('quit')">退出程序</button>
        <button type="button" :disabled="busy" @click="choose('cancel')">取消</button>
      </div>
    </div>
  </div>
</template>
