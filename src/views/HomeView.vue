<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { store, columnTagIds, resolveTags } from "../store";
import { prefs } from "../prefs";
import { LAYOUT_OPTIONS, PAGE_SIZE_OPTIONS, type ColumnMeta, type LayoutMode } from "../types";
import TagChips from "../components/TagChips.vue";
import AppTopbar from "../components/AppTopbar.vue";

const route = useRoute();
const router = useRouter();

const q = computed(() => String(route.query.q || ""));
const tag = computed(() => String(route.query.tag || "") || undefined);
const page = computed(() => Math.max(1, Number(route.query.page || 1) || 1));

function filterColumns(cols: ColumnMeta[]): ColumnMeta[] {
  const needle = q.value.trim().toLowerCase();
  return cols
    .map((col) => {
      if (tag.value && !columnTagIds(col.slug).includes(tag.value)) return null;
      if (!needle) return col;
      const titleHit = col.title.toLowerCase().includes(needle);
      const tagHit = resolveTags(columnTagIds(col.slug)).some((t) =>
        t.name.toLowerCase().includes(needle)
      );
      const arts = col.articles.filter((a) => a.title.toLowerCase().includes(needle));
      if (titleHit || tagHit) return col;
      if (arts.length) return { ...col, articles: arts, articleCount: arts.length };
      return null;
    })
    .filter(Boolean) as ColumnMeta[];
}

const filtered = computed(() => filterColumns(store.columns));

const pageData = computed(() => {
  const items = filtered.value;
  const total = items.length;
  const pages = Math.max(1, Math.ceil(total / prefs.pageSize));
  const safePage = Math.min(Math.max(1, page.value), pages);
  const start = (safePage - 1) * prefs.pageSize;
  return {
    items: items.slice(start, start + prefs.pageSize),
    page: safePage,
    pages,
    total,
    start: total ? start + 1 : 0,
    end: Math.min(start + prefs.pageSize, total),
  };
});

const activeTagName = computed(
  () => store.meta.tags.find((t) => t.id === tag.value)?.name
);

function setLayout(id: LayoutMode) {
  prefs.layout = id;
}

function setPageSize(e: Event) {
  prefs.pageSize = Number((e.target as HTMLSelectElement).value);
  goPage(1);
}

function goPage(p: number) {
  const query: Record<string, string> = {};
  if (q.value) query.q = q.value;
  if (tag.value) query.tag = tag.value;
  if (p > 1) query.page = String(p);
  router.replace({ query });
}

function pageButtons() {
  const { page: cur, pages } = pageData.value;
  const buttons: { p: number; label: string; kind: "num" | "nav" | "ellipsis" }[] = [];
  buttons.push({ p: cur - 1, label: "上一页", kind: "nav" });
  const windowSize = 5;
  let from = Math.max(1, cur - Math.floor(windowSize / 2));
  let to = Math.min(pages, from + windowSize - 1);
  from = Math.max(1, to - windowSize + 1);
  if (from > 1) {
    buttons.push({ p: 1, label: "1", kind: "num" });
    if (from > 2) buttons.push({ p: 0, label: "…", kind: "ellipsis" });
  }
  for (let p = from; p <= to; p++) buttons.push({ p, label: String(p), kind: "num" });
  if (to < pages) {
    if (to < pages - 1) buttons.push({ p: 0, label: "…", kind: "ellipsis" });
    buttons.push({ p: pages, label: String(pages), kind: "num" });
  }
  buttons.push({ p: cur + 1, label: "下一页", kind: "nav" });
  return buttons;
}
</script>

<template>
  <AppTopbar />
  <main class="main">
    <section class="hero">
      <h1>Note Reader</h1>
    </section>

    <div class="home-controls">
      <div class="control-group">
        <span class="control-label">布局</span>
        <div class="seg">
          <button
            v-for="l in LAYOUT_OPTIONS"
            :key="l.id"
            type="button"
            :class="{ active: prefs.layout === l.id }"
            :title="l.desc"
            @click="setLayout(l.id)"
          >
            {{ l.label }}
          </button>
        </div>
      </div>
      <div class="control-group">
        <span class="control-label">每页</span>
        <select :value="prefs.pageSize" @change="setPageSize">
          <option v-for="n in PAGE_SIZE_OPTIONS" :key="n" :value="n">{{ n }}</option>
        </select>
      </div>
      <div class="control-group tag-filter-group">
        <span class="control-label">标签</span>
        <div class="tag-filter">
          <router-link class="tag" :class="{ active: !tag }" to="/">全部</router-link>
          <router-link
            v-for="t in store.meta.tags"
            :key="t.id"
            class="tag"
            :class="{ active: tag === t.id }"
            :to="{ path: '/', query: { tag: t.id } }"
            :style="{ '--tag': t.color }"
          >
            {{ t.name }}
          </router-link>
        </div>
      </div>
      <div v-if="activeTagName" class="filter-hint">
        正在筛选标签：<strong>{{ activeTagName }}</strong>
        <router-link to="/">清除</router-link>
      </div>
    </div>

    <div v-if="!pageData.items.length" class="empty">没有匹配的专栏</div>

    <div v-else-if="prefs.layout === 'grid'" class="grid">
      <router-link
        v-for="c in pageData.items"
        :key="c.slug"
        class="card"
        :to="`/c/${encodeURIComponent(c.slug)}`"
      >
        <h3>{{ c.title }}</h3>
        <div class="meta">{{ c.articleCount }} 篇</div>
      </router-link>
    </div>

    <div v-else class="table-wrap">
      <table class="col-table">
        <thead>
          <tr>
            <th>专栏</th>
            <th>篇数</th>
            <th>标签</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in pageData.items" :key="c.slug">
            <td>
              <router-link :to="`/c/${encodeURIComponent(c.slug)}`">{{ c.title }}</router-link>
            </td>
            <td>{{ c.articleCount }}</td>
            <td>
              <TagChips :tags="resolveTags(columnTagIds(c.slug))" />
              <span v-if="!columnTagIds(c.slug).length">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="pager-bar">
      <span class="pager-info">
        第 {{ pageData.start }}–{{ pageData.end }} 项，共 {{ pageData.total }} 个
      </span>
      <div v-if="pageData.pages > 1" class="page-btns">
        <template v-for="(b, i) in pageButtons()" :key="i">
          <span v-if="b.kind === 'ellipsis'" class="page-ellipsis">…</span>
          <span
            v-else-if="
              (b.kind === 'nav' && (b.p < 1 || b.p > pageData.pages)) ||
              (b.kind === 'num' && b.p === pageData.page)
            "
            class="page-btn"
            :class="{ active: b.p === pageData.page, disabled: b.kind === 'nav' }"
          >
            {{ b.label }}
          </span>
          <button v-else type="button" class="page-btn" @click="goPage(b.p)">{{ b.label }}</button>
        </template>
      </div>
    </div>
  </main>
</template>
