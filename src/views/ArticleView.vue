<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { store } from "../store";
import { readArticle, resolveFileUrl } from "../api";
import { renderMarkdown, hydrateMermaid } from "../markdown";
import { renderDocx } from "../docx";
import { loadProgress, saveProgress, prefs } from "../prefs";
import { articleKindOf } from "../types";
import AppTopbar from "../components/AppTopbar.vue";

const props = defineProps<{ slug: string; filename: string }>();
const router = useRouter();

const sidebarOpen = ref(false);
const html = ref("");
const pdfUrl = ref("");
const pdfZoom = ref(125);
const loading = ref(false);
const error = ref("");
const progressWidth = ref(0);
const linkView = ref<{ url: string; label: string } | null>(null);
const articleEl = ref<HTMLElement | null>(null);

const PDF_ZOOM_MIN = 50;
const PDF_ZOOM_MAX = 300;
const PDF_ZOOM_STEP = 25;

function setPdfZoom(next: number) {
  pdfZoom.value = Math.min(PDF_ZOOM_MAX, Math.max(PDF_ZOOM_MIN, Math.round(next)));
}

function zoomPdfIn() {
  setPdfZoom(pdfZoom.value + PDF_ZOOM_STEP);
}

function zoomPdfOut() {
  setPdfZoom(pdfZoom.value - PDF_ZOOM_STEP);
}

function resetPdfZoom() {
  setPdfZoom(125);
}

function onPdfWheel(e: WheelEvent) {
  if (!(e.ctrlKey || e.metaKey)) return;
  e.preventDefault();
  if (e.deltaY < 0) zoomPdfIn();
  else zoomPdfOut();
}

const col = computed(() =>
  store.columns.find((c) => c.slug === decodeURIComponent(props.slug))
);

const filename = computed(() => decodeURIComponent(props.filename));

const idx = computed(() => col.value?.articles.findIndex((a) => a.filename === filename.value) ?? -1);
const article = computed(() => (idx.value >= 0 ? col.value!.articles[idx.value] : null));
const kind = computed(() => (article.value ? articleKindOf(article.value) : "markdown"));
const prev = computed(() => (idx.value > 0 ? col.value!.articles[idx.value - 1] : null));
const next = computed(() =>
  col.value && idx.value >= 0 && idx.value < col.value.articles.length - 1
    ? col.value.articles[idx.value + 1]
    : null
);

let onScroll: (() => void) | null = null;

async function load() {
  linkView.value = null;
  html.value = "";
  pdfUrl.value = "";
  pdfZoom.value = 125;
  if (!col.value || !article.value) {
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const k = articleKindOf(article.value);
    if (k === "pdf") {
      pdfUrl.value = await resolveFileUrl(article.value.path);
    } else if (k === "docx") {
      html.value = await renderDocx(article.value.path);
    } else {
      const md = await readArticle(article.value.path);
      html.value = await renderMarkdown(md, col.value.dir);
    }
    await nextTick();
    if (k === "markdown" && articleEl.value) {
      await hydrateMermaid(articleEl.value);
    }
    const key = `${col.value.slug}/${article.value.filename}`;
    const map = loadProgress();
    const saved = map[key];
    if (saved?.scroll && k !== "pdf") window.scrollTo(0, saved.scroll);

    onScroll = () => {
      const max = document.documentElement.scrollHeight - window.innerHeight;
      const ratio = max > 0 ? window.scrollY / max : 0;
      progressWidth.value = Math.min(100, ratio * 100);
      map[key] = { scroll: window.scrollY, at: Date.now() };
      saveProgress(map);
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function cleanup() {
  if (onScroll) {
    window.removeEventListener("scroll", onScroll);
    onScroll = null;
  }
}

function closeLinkView() {
  linkView.value = null;
}

async function openInBrowser() {
  if (!linkView.value) return;
  const url = linkView.value.url;
  if (!/^https?:\/\//i.test(url)) return;
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank");
  }
}

function findLocalArticle(href: string): string | null {
  if (!col.value) return null;
  let path = href.split("#")[0].split("?")[0];
  try {
    path = decodeURIComponent(path);
  } catch {
    /* keep */
  }
  path = path.replace(/^\.\//, "");
  const base = path.split("/").pop() || path;
  const stripExt = (s: string) => s.replace(/\.(md|markdown|docx|pdf)$/i, "");
  const hit = col.value.articles.find(
    (a) =>
      a.filename === base ||
      a.filename === path ||
      stripExt(a.filename) === stripExt(base) ||
      a.title === base
  );
  return hit ? hit.filename : null;
}

function onArticleClick(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  const anchor = target?.closest?.("a") as HTMLAnchorElement | null;
  if (!anchor || !col.value) return;

  const href = anchor.getAttribute("href");
  if (!href || href.startsWith("javascript:")) return;

  if (href.startsWith("#")) {
    const id = decodeURIComponent(href.slice(1));
    const el =
      document.getElementById(id) ||
      document.querySelector(`[name="${CSS.escape(id)}"]`);
    if (el) {
      e.preventDefault();
      el.scrollIntoView({ behavior: "smooth", block: "start" });
    }
    return;
  }

  e.preventDefault();
  e.stopPropagation();

  const local = findLocalArticle(href);
  if (local) {
    router.push(`/c/${encodeURIComponent(col.value.slug)}/${encodeURIComponent(local)}`);
    return;
  }

  const label = (anchor.textContent || href).trim().slice(0, 80) || href;
  linkView.value = { url: href, label };
}

watch(
  () => [props.slug, props.filename],
  async () => {
    cleanup();
    await load();
  }
);

watch(
  () => prefs.theme,
  async () => {
    if (kind.value !== "markdown" || !articleEl.value || !html.value) return;
    await nextTick();
    await hydrateMermaid(articleEl.value);
  }
);

onMounted(() => {
  load();
  window.addEventListener("app-back", onAppBack);
});
onBeforeUnmount(() => {
  cleanup();
  window.removeEventListener("app-back", onAppBack);
});

function onAppBack(e: Event) {
  if (linkView.value) {
    e.preventDefault();
    linkView.value = null;
  }
}
</script>

<template>
  <div class="progress-bar" :style="{ width: progressWidth + '%' }" />
  <AppTopbar show-menu @toggle-sidebar="sidebarOpen = !sidebarOpen" />
  <div v-if="col && article" class="reader" :class="{ 'reader-pdf': kind === 'pdf' }">
    <aside class="sidebar" :class="{ open: sidebarOpen }">
      <h2>{{ col.title }}</h2>
      <ul class="nav-list">
        <li v-for="a in col.articles" :key="a.filename">
          <router-link
            :class="{ active: a.filename === filename }"
            :to="`/c/${encodeURIComponent(col.slug)}/${encodeURIComponent(a.filename)}`"
          >
            {{ a.title }}
          </router-link>
        </li>
      </ul>
    </aside>
    <main class="main" :class="{ 'main-pdf': kind === 'pdf' }">
      <div class="breadcrumb">
        <router-link to="/">全部专栏</router-link> /
        <router-link :to="`/c/${encodeURIComponent(col.slug)}`">{{ col.title }}</router-link>
        <span v-if="kind !== 'markdown'" class="file-kind-tag">{{ kind === "pdf" ? "PDF" : "Word" }}</span>
      </div>
      <div v-if="loading" class="empty">加载中…</div>
      <div v-else-if="error" class="empty">{{ error }}</div>
      <template v-else-if="kind === 'pdf' && pdfUrl">
        <div class="pdf-toolbar" role="toolbar" aria-label="PDF 缩放">
          <button type="button" :disabled="pdfZoom <= PDF_ZOOM_MIN" @click="zoomPdfOut" title="缩小">−</button>
          <span class="pdf-zoom-label">{{ pdfZoom }}%</span>
          <button type="button" :disabled="pdfZoom >= PDF_ZOOM_MAX" @click="zoomPdfIn" title="放大">+</button>
          <button type="button" class="pdf-zoom-reset" @click="resetPdfZoom" title="恢复默认">默认</button>
          <span class="pdf-zoom-hint">Ctrl / ⌘ + 滚轮也可缩放</span>
        </div>
        <div class="pdf-viewport" @wheel="onPdfWheel">
          <div
            class="pdf-scale"
            :style="{
              width: 100 / (pdfZoom / 100) + '%',
              height: 100 / (pdfZoom / 100) + '%',
              transform: `scale(${pdfZoom / 100})`,
            }"
          >
            <iframe class="pdf-frame" :src="pdfUrl" title="PDF 预览" />
          </div>
        </div>
      </template>
      <article
        v-else
        ref="articleEl"
        class="article"
        :class="{ 'article-docx': kind === 'docx' }"
        v-html="html"
        @click="onArticleClick"
      />
      <nav class="pager" :class="{ 'pager-pdf': kind === 'pdf' }">
        <router-link
          v-if="prev"
          :to="`/c/${encodeURIComponent(col.slug)}/${encodeURIComponent(prev.filename)}`"
        >
          ← {{ prev.title }}
        </router-link>
        <span v-else />
        <router-link
          v-if="next"
          class="next"
          :to="`/c/${encodeURIComponent(col.slug)}/${encodeURIComponent(next.filename)}`"
        >
          {{ next.title }} →
        </router-link>
      </nav>
    </main>
  </div>
  <div v-else class="empty">文章不存在</div>

  <div v-if="linkView" class="link-view" role="dialog" aria-label="链接预览">
    <header class="link-view-bar">
      <button type="button" class="link-back" @click="closeLinkView">← 返回文章</button>
      <span class="link-view-title" :title="linkView.url">{{ linkView.label }}</span>
      <button type="button" @click="openInBrowser">浏览器打开</button>
    </header>
    <iframe
      v-if="/^https?:\/\//i.test(linkView.url)"
      class="link-view-frame"
      :src="linkView.url"
      referrerpolicy="no-referrer"
    />
    <div v-else class="link-view-fallback">
      <p>无法在应用内打开此链接：</p>
      <code>{{ linkView.url }}</code>
      <div class="link-view-actions">
        <button type="button" class="primary" @click="openInBrowser">浏览器打开</button>
        <button type="button" @click="closeLinkView">返回文章</button>
      </div>
    </div>
    <div class="link-view-hint">
      若页面为空白，多半是网站禁止内嵌，可点「浏览器打开」，或「返回文章」。
    </div>
  </div>
</template>
