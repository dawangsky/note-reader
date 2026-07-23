import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ColumnMeta, MetaStore, PathInfo, PlatformInfo } from "./types";

export async function listColumns(): Promise<ColumnMeta[]> {
  return invoke<ColumnMeta[]>("list_columns");
}

export async function refreshColumns(): Promise<ColumnMeta[]> {
  return invoke<ColumnMeta[]>("refresh_columns");
}

export async function readArticle(path: string): Promise<string> {
  return invoke<string>("read_article", { path });
}

export async function readFileBytes(path: string): Promise<Uint8Array> {
  const bytes = await invoke<number[]>("read_file_bytes", { path });
  return Uint8Array.from(bytes);
}

export async function resolveFileUrl(path: string): Promise<string> {
  const abs = await invoke<string>("resolve_file_path", { path });
  return convertFileSrc(abs);
}

export async function loadMeta(): Promise<MetaStore> {
  return invoke<MetaStore>("load_meta");
}

export async function saveMeta(meta: MetaStore): Promise<void> {
  return invoke("save_meta", { meta });
}

export async function getContentRoot(): Promise<string> {
  return invoke<string>("get_content_root");
}

export async function getPathInfo(): Promise<PathInfo> {
  return invoke<PathInfo>("get_path_info");
}

export async function getPlatformInfo(): Promise<PlatformInfo> {
  return invoke<PlatformInfo>("get_platform_info");
}

export async function setColumnsPath(path: string | null): Promise<PathInfo> {
  return invoke<PathInfo>("set_columns_path", { path });
}

export async function setSessionColumnsPath(path: string | null): Promise<PathInfo> {
  return invoke<PathInfo>("set_session_columns_path", { path });
}

export async function completePathOnboarding(path: string | null): Promise<PathInfo> {
  return invoke<PathInfo>("complete_path_onboarding", { path });
}

export async function getCloseBehavior(): Promise<"tray" | "quit" | null> {
  return invoke<"tray" | "quit" | null>("get_close_behavior");
}

export async function setCloseBehavior(behavior: "tray" | "quit" | null): Promise<void> {
  return invoke("set_close_behavior", { behavior });
}

export async function resolveAssetUrl(columnDir: string, relative: string): Promise<string> {
  const abs = await invoke<string>("resolve_asset", { columnDir, relative });
  return convertFileSrc(abs);
}
