<script setup lang="ts">
import { nextTick, onMounted, onBeforeUnmount, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { prefs } from "../prefs";
import { applySessionBrowse, reloadColumns, store } from "../store";
import type { ThemeMode } from "../types";
import SessionBanner from "./SessionBanner.vue";

type HistState = {
  back?: unknown;
  forward?: unknown;
  position?: number;
};

const THEME_OPTIONS: { id: ThemeMode; label: string; desc: string; swatch: string }[] = [
  { id: "light", label: "浅色", desc: "默认阅读", swatch: "#f3efe6" },
  { id: "ink", label: "水墨", desc: "类墨水屏灰阶", swatch: "#e6e4df" },
];

const route = useRoute();
const router = useRouter();
const refreshing = ref(false);
const canGoBack = ref(false);
const canGoForward = ref(false);
const themeOpen = ref(false);
const loadOpen = ref(false);
const loadBusy = ref(false);

const q = ref(String(route.query.q || ""));

function themeButtonLabel() {
  return THEME_OPTIONS.find((t) => t.id === prefs.theme)?.label || "浅色";
}

function loadButtonLabel() {
  return store.pathInfo?.isSessionBrowse ? "临时" : "固定";
}

function syncNav() {
  const state = (window.history.state || {}) as HistState;
  canGoBack.value = !!state.back || (route.name !== "home" && window.history.length > 1);
  canGoForward.value = !!state.forward;
  q.value = String(route.query.q || "");
}

function onSearch(e: KeyboardEvent) {
  if (e.key !== "Enter") return;
  const value = (e.target as HTMLInputElement).value.trim();
  const query = { ...route.query } as Record<string, string>;
  if (value) query.q = value;
  else delete query.q;
  delete query.page;
  if (route.name === "settings") {
    router.push({ name: "home", query });
  } else if (route.name === "home") {
    router.replace({ query });
  } else {
    router.push({ name: "home", query });
  }
}

function toggleThemeMenu() {
  loadOpen.value = false;
  themeOpen.value = !themeOpen.value;
}

function toggleLoadMenu() {
  themeOpen.value = false;
  loadOpen.value = !loadOpen.value;
}

function setTheme(id: ThemeMode) {
  prefs.theme = id;
  themeOpen.value = false;
}

function onDocClick(e: MouseEvent) {
  const el = e.target as HTMLElement | null;
  if (!el?.closest?.(".theme-menu")) themeOpen.value = false;
  if (!el?.closest?.(".load-menu")) loadOpen.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    themeOpen.value = false;
    loadOpen.value = false;
  }
}

function setFont(size: "sm" | "md" | "lg") {
  prefs.fontSize = size;
}

function goBack() {
  const ev = new CustomEvent("app-back", { cancelable: true });
  window.dispatchEvent(ev);
  if (ev.defaultPrevented) {
    syncNav();
    return;
  }

  const state = (window.history.state || {}) as HistState;
  if (state.back) {
    router.back();
    return;
  }

  if (route.name === "article" && typeof route.params.slug === "string") {
    router.push(`/c/${encodeURIComponent(route.params.slug)}`);
  } else if (route.name !== "home") {
    router.push("/");
  }
}

function goForward() {
  if (!canGoForward.value) return;
  router.forward();
}

async function onRefresh() {
  if (refreshing.value) return;
  refreshing.value = true;
  try {
    await reloadColumns();
  } finally {
    refreshing.value = false;
  }
}

async function useFixedPath() {
  if (loadBusy.value) return;
  loadOpen.value = false;
  if (!store.pathInfo?.isSessionBrowse) return;
  loadBusy.value = true;
  try {
    await applySessionBrowse(null);
    await router.push("/");
  } finally {
    loadBusy.value = false;
  }
}

async function pickBrowseFolder() {
  if (loadBusy.value) return;
  loadOpen.value = false;
  const selected = await open({ directory: true, multiple: false });
  if (!selected || Array.isArray(selected)) return;
  loadBusy.value = true;
  try {
    await applySessionBrowse(selected);
    await router.push("/");
  } finally {
    loadBusy.value = false;
  }
}

watch(
  () => route.fullPath,
  () => {
    themeOpen.value = false;
    loadOpen.value = false;
    nextTick(syncNav);
  }
);

onMounted(() => {
  syncNav();
  window.addEventListener("popstate", syncNav);
  document.addEventListener("click", onDocClick);
  document.addEventListener("keydown", onKeydown);
});
onBeforeUnmount(() => {
  window.removeEventListener("popstate", syncNav);
  document.removeEventListener("click", onDocClick);
  document.removeEventListener("keydown", onKeydown);
});

defineProps<{ showMenu?: boolean }>();
const emit = defineEmits<{ toggleSidebar: [] }>();
</script>

<template>
  <header class="topbar">
    <div class="nav-btns">
      <button
        type="button"
        class="icon-btn"
        title="后退"
        aria-label="后退"
        :disabled="!canGoBack"
        @click="goBack"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path
            fill="currentColor"
            d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"
          />
        </svg>
      </button>
      <button
        type="button"
        class="icon-btn"
        title="前进"
        aria-label="前进"
        :disabled="!canGoForward"
        @click="goForward"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path
            fill="currentColor"
            d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8-8-8z"
          />
        </svg>
      </button>
    </div>
    <button v-if="showMenu" class="mobile-toggle" type="button" @click="emit('toggleSidebar')">
      目录
    </button>
    <router-link class="brand" to="/">Note Reader</router-link>
    <input
      class="search"
      type="search"
      placeholder="搜索专栏、文章或标签…"
      :value="q"
      @keydown="onSearch"
    />
    <div class="toolbar">
      <button type="button" :class="{ active: prefs.fontSize === 'sm' }" @click="setFont('sm')">小</button>
      <button type="button" :class="{ active: prefs.fontSize === 'md' }" @click="setFont('md')">中</button>
      <button type="button" :class="{ active: prefs.fontSize === 'lg' }" @click="setFont('lg')">大</button>
      <div class="theme-menu">
        <button
          type="button"
          class="theme-trigger"
          :class="{ open: themeOpen, active: prefs.theme === 'ink' }"
          :aria-expanded="themeOpen"
          aria-haspopup="listbox"
          title="选择主题"
          @click.stop="toggleThemeMenu"
        >
          {{ themeButtonLabel() }}
          <svg class="theme-caret" viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
            <path fill="currentColor" d="M7 10l5 5 5-5H7z" />
          </svg>
        </button>
        <div v-if="themeOpen" class="theme-dropdown" role="listbox" aria-label="主题">
          <button
            v-for="opt in THEME_OPTIONS"
            :key="opt.id"
            type="button"
            class="theme-option"
            role="option"
            :aria-selected="prefs.theme === opt.id"
            :class="{ selected: prefs.theme === opt.id }"
            @click.stop="setTheme(opt.id)"
          >
            <span class="theme-swatch" :style="{ background: opt.swatch }" />
            <span class="theme-option-text">
              <strong>{{ opt.label }}</strong>
              <small>{{ opt.desc }}</small>
            </span>
            <span v-if="prefs.theme === opt.id" class="theme-check" aria-hidden="true">✓</span>
          </button>
        </div>
      </div>
      <div class="load-menu theme-menu">
        <button
          type="button"
          class="theme-trigger"
          :class="{ open: loadOpen, active: store.pathInfo?.isSessionBrowse }"
          :aria-expanded="loadOpen"
          aria-haspopup="listbox"
          title="加载模式"
          :disabled="loadBusy"
          @click.stop="toggleLoadMenu"
        >
          {{ loadButtonLabel() }}
          <svg class="theme-caret" viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
            <path fill="currentColor" d="M7 10l5 5 5-5H7z" />
          </svg>
        </button>
        <div v-if="loadOpen" class="theme-dropdown load-dropdown" role="listbox" aria-label="加载模式">
          <button
            type="button"
            class="theme-option"
            role="option"
            :aria-selected="!store.pathInfo?.isSessionBrowse"
            :class="{ selected: !store.pathInfo?.isSessionBrowse }"
            @click.stop="useFixedPath"
          >
            <span class="theme-option-text">
              <strong>固定路径</strong>
              <small>使用设置中的固定目录</small>
            </span>
            <span v-if="!store.pathInfo?.isSessionBrowse" class="theme-check" aria-hidden="true">✓</span>
          </button>
          <button
            type="button"
            class="theme-option"
            role="option"
            :aria-selected="!!store.pathInfo?.isSessionBrowse"
            :class="{ selected: store.pathInfo?.isSessionBrowse }"
            @click.stop="pickBrowseFolder"
          >
            <span class="theme-option-text">
              <strong>临时浏览…</strong>
              <small>选文件夹，不改默认设置</small>
            </span>
            <span v-if="store.pathInfo?.isSessionBrowse" class="theme-check" aria-hidden="true">✓</span>
          </button>
        </div>
      </div>
      <button
        type="button"
        class="icon-btn"
        title="刷新目录"
        aria-label="刷新目录"
        :disabled="refreshing"
        :class="{ spinning: refreshing }"
        @click="onRefresh"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
          <path
            fill="currentColor"
            d="M17.65 6.35A7.95 7.95 0 0 0 12 4a8 8 0 1 0 7.75 10h-2.1A6 6 0 1 1 12 6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35Z"
          />
        </svg>
      </button>
      <router-link
        class="btn-link"
        :class="{ active: route.name === 'settings' }"
        to="/settings/path"
      >设置</router-link>
    </div>
  </header>
  <SessionBanner />
</template>
