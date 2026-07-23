<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { applySessionBrowse, store } from "../store";

const router = useRouter();
const busy = ref(false);

const shortPath = computed(() => {
  const p = store.pathInfo?.columnsPath || "";
  if (p.length <= 56) return p;
  return `…${p.slice(-54)}`;
});

async function changeFolder() {
  if (busy.value) return;
  const selected = await open({ directory: true, multiple: false });
  if (!selected || Array.isArray(selected)) return;
  busy.value = true;
  try {
    await applySessionBrowse(selected);
    await router.push("/");
  } finally {
    busy.value = false;
  }
}

async function exitBrowse() {
  if (busy.value) return;
  busy.value = true;
  try {
    await applySessionBrowse(null);
    await router.push("/");
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div
    v-if="store.pathInfo?.isSessionBrowse"
    class="session-banner"
    role="status"
  >
    <span class="session-banner-label">临时浏览</span>
    <code class="session-banner-path" :title="store.pathInfo?.columnsPath">{{ shortPath }}</code>
    <div class="session-banner-actions">
      <button type="button" :disabled="busy" @click="changeFolder">换目录</button>
      <button type="button" class="primary" :disabled="busy" @click="exitBrowse">返回固定路径</button>
    </div>
  </div>
</template>
