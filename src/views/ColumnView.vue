<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { store, columnTagIds, resolveTags } from "../store";
import { articleKindOf } from "../types";
import AppTopbar from "../components/AppTopbar.vue";
import TagChips from "../components/TagChips.vue";

const props = defineProps<{ slug: string }>();
const route = useRoute();
const q = computed(() => String(route.query.q || ""));

const col = computed(() =>
  store.columns.find((c) => c.slug === decodeURIComponent(props.slug))
);

const arts = computed(() => {
  if (!col.value) return [];
  if (!q.value) return col.value.articles;
  const needle = q.value.toLowerCase();
  return col.value.articles.filter((a) => a.title.toLowerCase().includes(needle));
});

function metaLabel(a: { kind?: string; filename: string; chars: number }) {
  const k = articleKindOf(a);
  if (k === "pdf") return "PDF";
  if (k === "docx") return "Word";
  return `${Math.round(a.chars / 100) / 10}k`;
}
</script>

<template>
  <AppTopbar />
  <main v-if="col" class="main">
    <div class="breadcrumb">
      <router-link to="/">全部专栏</router-link> / {{ col.title }}
    </div>
    <div class="column-head">
      <h1>{{ col.title }}</h1>
      <div class="meta">
        {{ col.articleCount }} 篇文章
        <TagChips :tags="resolveTags(columnTagIds(col.slug))" />
      </div>
    </div>
    <ul class="article-list">
      <li v-for="a in arts" :key="a.filename">
        <router-link
          :to="`/c/${encodeURIComponent(col.slug)}/${encodeURIComponent(a.filename)}`"
        >
          <span>{{ a.title }}</span>
          <span class="chars">{{ metaLabel(a) }}</span>
        </router-link>
      </li>
    </ul>
  </main>
  <div v-else class="empty">专栏不存在</div>
</template>
