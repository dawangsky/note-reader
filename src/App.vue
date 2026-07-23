<script setup lang="ts">
import { onMounted } from "vue";
import { initStore, store } from "./store";
import OnboardDialog from "./components/OnboardDialog.vue";
import CloseConfirmDialog from "./components/CloseConfirmDialog.vue";

onMounted(() => {
  void initStore();
});
</script>

<template>
  <div v-if="store.loading && !store.ready" class="empty">正在加载本地专栏…</div>
  <div v-else-if="store.error" class="empty">
    <p>加载失败</p>
    <p>{{ store.error }}</p>
  </div>
  <template v-else>
    <OnboardDialog v-if="store.needsOnboarding" />
    <router-view />
  </template>
  <CloseConfirmDialog />
</template>
