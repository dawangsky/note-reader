<script setup lang="ts">
import type { TagDef } from "../types";

defineProps<{ tags: TagDef[]; linkable?: boolean }>();
</script>

<template>
  <span v-if="tags.length" class="tag-row">
    <template v-for="t in tags" :key="t.id">
      <router-link
        v-if="linkable !== false"
        class="tag"
        :to="{ path: '/', query: { tag: t.id } }"
        :style="{ '--tag': t.color }"
      >
        {{ t.name }}
      </router-link>
      <span v-else class="tag" :style="{ '--tag': t.color }">{{ t.name }}</span>
    </template>
  </span>
</template>
