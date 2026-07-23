export type ArticleKind = "markdown" | "docx" | "pdf";

export type ArticleMeta = {
  title: string;
  filename: string;
  path: string;
  chars: number;
  kind?: ArticleKind | string;
};

export function articleKindOf(a: Pick<ArticleMeta, "kind" | "filename">): ArticleKind {
  const k = String(a.kind || "").toLowerCase();
  if (k === "docx" || k === "pdf" || k === "markdown") return k;
  const name = a.filename.toLowerCase();
  if (name.endsWith(".docx")) return "docx";
  if (name.endsWith(".pdf")) return "pdf";
  return "markdown";
}

export type ColumnMeta = {
  title: string;
  slug: string;
  dir: string;
  articleCount: number;
  articles: ArticleMeta[];
};

export type TagDef = {
  id: string;
  name: string;
  color: string;
};

export type MetaStore = {
  tags: TagDef[];
  columnTags: Record<string, string[]>;
};

export type PathInfo = {
  columnsPath: string;
  fixedColumnsPath: string;
  defaultPath: string;
  isCustom: boolean;
  isSessionBrowse: boolean;
  contentRoot: string;
  pathOnboarded: boolean;
  os: string;
  dataHome: string;
};

export type PlatformInfo = {
  os: string;
  homeDir: string;
  dataHome: string;
  defaultColumnsPath: string;
  installHint: string;
};

export type LayoutMode = "grid" | "list";

export type ThemeMode = "light" | "ink";

export type Prefs = {
  theme: ThemeMode;
  fontSize: "sm" | "md" | "lg";
  layout: LayoutMode;
  pageSize: number;
};

export const LAYOUT_OPTIONS: { id: LayoutMode; label: string; desc: string }[] = [
  { id: "grid", label: "卡片", desc: "卡片网格" },
  { id: "list", label: "列表", desc: "列表视图，含篇数与标签列" },
];

export const PAGE_SIZE_OPTIONS = [12, 24, 36, 48];

export const FONT_SIZES: Record<Prefs["fontSize"], string> = {
  sm: "0.95rem",
  md: "1.05rem",
  lg: "1.2rem",
};

export const DEFAULT_PREFS: Prefs = {
  theme: "light",
  fontSize: "md",
  layout: "grid",
  pageSize: 24,
};

export const EMPTY_META: MetaStore = { tags: [], columnTags: {} };
