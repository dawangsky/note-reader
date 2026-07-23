import { reactive, watch } from "vue";
import type { Prefs, ThemeMode } from "./types";
import { DEFAULT_PREFS, FONT_SIZES } from "./types";

const PREFS_KEY = "note-reader-prefs";
const PROGRESS_KEY = "note-reader-progress";

function normalizeTheme(raw: unknown): ThemeMode {
  if (raw === "ink" || raw === "eink") return "ink";
  return "light";
}

function load(): Prefs {
  try {
    const raw = localStorage.getItem(PREFS_KEY);
    if (raw) {
      const parsed = { ...DEFAULT_PREFS, ...JSON.parse(raw) } as Prefs & {
        layout?: string;
      };
      const layout = String(parsed.layout || "");
      let nextLayout: Prefs["layout"] = DEFAULT_PREFS.layout;
      if (layout === "list" || layout === "table") nextLayout = "list";
      else if (layout === "grid" || layout === "compact" || layout === "magazine") nextLayout = "grid";
      return {
        theme: normalizeTheme(parsed.theme),
        fontSize: parsed.fontSize === "sm" || parsed.fontSize === "lg" ? parsed.fontSize : "md",
        layout: nextLayout,
        pageSize: Number(parsed.pageSize) || DEFAULT_PREFS.pageSize,
      };
    }
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_PREFS };
}

export const prefs = reactive<Prefs>(load());

export function applyPrefs() {
  document.documentElement.dataset.theme = prefs.theme;
  document.documentElement.style.setProperty("--font-size", FONT_SIZES[prefs.fontSize]);
}

export function cycleTheme() {
  prefs.theme = prefs.theme === "light" ? "ink" : "light";
}

export function themeLabel(theme: ThemeMode = prefs.theme): string {
  return theme === "ink" ? "水墨" : "浅色";
}

watch(
  prefs,
  () => {
    localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
    applyPrefs();
  },
  { deep: true }
);

applyPrefs();

export type ProgressMap = Record<string, { scroll: number; at: number }>;

export function loadProgress(): ProgressMap {
  try {
    return JSON.parse(localStorage.getItem(PROGRESS_KEY) || "{}");
  } catch {
    return {};
  }
}

export function saveProgress(map: ProgressMap) {
  localStorage.setItem(PROGRESS_KEY, JSON.stringify(map));
}

export function slugifyTag(name: string): string {
  const base = name
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "-")
    .replace(/[^\w\u4e00-\u9fff-]/g, "");
  return base || `tag-${Date.now()}`;
}

export function tagColor(seed: string): string {
  const colors = [
    "#0f5c4c",
    "#0369a1",
    "#b45309",
    "#b91c1c",
    "#6d28d9",
    "#4d7c0f",
    "#9f1239",
    "#0e7490",
    "#c2410c",
    "#1d4ed8",
  ];
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return colors[h % colors.length];
}
