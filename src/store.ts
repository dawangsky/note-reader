import { reactive } from "vue";
import {
  listColumns,
  loadMeta,
  saveMeta as saveMetaApi,
  refreshColumns,
  getPathInfo,
  setColumnsPath as setColumnsPathApi,
  setSessionColumnsPath as setSessionColumnsPathApi,
  completePathOnboarding as completePathOnboardingApi,
} from "./api";
import type { ColumnMeta, MetaStore, PathInfo, TagDef } from "./types";
import { EMPTY_META } from "./types";

export const store = reactive({
  columns: [] as ColumnMeta[],
  meta: { ...EMPTY_META, columnTags: {} } as MetaStore,
  pathInfo: null as PathInfo | null,
  loading: false,
  error: "",
  ready: false,
  needsOnboarding: false,
});

async function applyPathResult(pathInfo: PathInfo) {
  store.pathInfo = pathInfo;
  const [cols, meta] = await Promise.all([refreshColumns(), loadMeta()]);
  store.columns = cols;
  store.meta = {
    tags: meta.tags || [],
    columnTags: meta.columnTags || {},
  };
}

export async function initStore() {
  store.loading = true;
  store.error = "";
  try {
    const [cols, meta, pathInfo] = await Promise.all([
      listColumns(),
      loadMeta(),
      getPathInfo(),
    ]);
    store.columns = cols;
    store.meta = {
      tags: meta.tags || [],
      columnTags: meta.columnTags || {},
    };
    store.pathInfo = pathInfo;
    // First launch (esp. Windows): confirm content/read path.
    // Dev builds use repo content/ and skip forcing the dialog every time if already onboarded.
    store.needsOnboarding = !pathInfo.pathOnboarded;
    store.ready = true;
  } catch (e) {
    store.error = String(e);
  } finally {
    store.loading = false;
  }
}

export async function reloadColumns() {
  store.columns = await refreshColumns();
}

export async function persistMeta() {
  await saveMetaApi(JSON.parse(JSON.stringify(store.meta)) as MetaStore);
}

export async function applyColumnsPath(path: string | null) {
  await applyPathResult(await setColumnsPathApi(path));
}

/** Enter temporary browse (`path`) or return to fixed path (`null`). Does not change settings. */
export async function applySessionBrowse(path: string | null) {
  await applyPathResult(await setSessionColumnsPathApi(path));
}

export async function finishOnboarding(path: string | null) {
  store.pathInfo = await completePathOnboardingApi(path);
  store.needsOnboarding = false;
  const [cols, meta] = await Promise.all([refreshColumns(), loadMeta()]);
  store.columns = cols;
  store.meta = {
    tags: meta.tags || [],
    columnTags: meta.columnTags || {},
  };
}

export function columnTagIds(slug: string): string[] {
  return store.meta.columnTags[slug] || [];
}

export function resolveTags(ids: string[]): TagDef[] {
  const map = new Map(store.meta.tags.map((t) => [t.id, t]));
  return ids.map((id) => map.get(id)).filter(Boolean) as TagDef[];
}
