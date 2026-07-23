use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleMeta {
    pub title: String,
    pub filename: String,
    /// Absolute path to the article file
    pub path: String,
    pub chars: usize,
    /// markdown | docx | pdf
    #[serde(default = "default_article_kind")]
    pub kind: String,
}

fn default_article_kind() -> String {
    "markdown".into()
}

fn article_kind(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("md") | Some("markdown") => Some("markdown"),
        Some("docx") => Some("docx"),
        Some("pdf") => Some("pdf"),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMeta {
    pub title: String,
    pub slug: String,
    /// Absolute path to the column directory
    pub dir: String,
    pub article_count: usize,
    pub articles: Vec<ArticleMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDef {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaStore {
    pub tags: Vec<TagDef>,
    pub column_tags: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Absolute path to the columns directory. Empty / missing → use default.
    #[serde(default)]
    pub columns_path: Option<String>,
    /// User has confirmed content path (first-run / installer flow).
    #[serde(default)]
    pub path_onboarded: bool,
    /// Remembered close-window action: "tray" | "quit". None → ask each time.
    #[serde(default)]
    pub close_behavior: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathInfo {
    /// Currently active columns path (session browse overrides fixed).
    pub columns_path: String,
    /// Persisted / default fixed path (what Admin edits).
    pub fixed_columns_path: String,
    pub default_path: String,
    pub is_custom: bool,
    /// True when temporarily browsing another folder (not saved to settings).
    pub is_session_browse: bool,
    pub content_root: String,
    pub path_onboarded: bool,
    pub os: String,
    /// Human-readable data home, e.g. ~/NoteReader
    pub data_home: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub home_dir: String,
    pub data_home: String,
    pub default_columns_path: String,
    pub install_hint: String,
}

pub struct ContentState {
    settings: RwLock<AppSettings>,
    /// In-memory only: temporary browse root (does not touch settings).
    session_columns: RwLock<Option<PathBuf>>,
}

impl ContentState {
    pub fn load(app: &AppHandle) -> Self {
        let _ = ensure_default_dirs(app);
        let settings = load_settings(app).unwrap_or_default();
        Self {
            settings: RwLock::new(settings),
            session_columns: RwLock::new(None),
        }
    }
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_file(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_file(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, text + "\n").map_err(|e| e.to_string())
}

/// User-facing data home: `~/NoteReader` (created on demand).
pub fn data_home(app: &AppHandle) -> PathBuf {
    let home = app
        .path()
        .home_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join("NoteReader")
}

/// Default columns directory by platform.
/// - Release: `~/NoteReader/columns` (macOS / Windows / Linux)
/// - Dev: `<repo>/content/columns` for easier local scraping
pub fn default_columns_path(app: &AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
        return repo.join("content").join("columns");
    }

    #[cfg(not(debug_assertions))]
    {
        data_home(app).join("columns")
    }
}

pub fn ensure_default_dirs(app: &AppHandle) -> Result<(), String> {
    let columns = default_columns_path(app);
    fs::create_dir_all(&columns).map_err(|e| format!("无法创建专栏目录: {e}"))?;
    let readme = columns.parent().map(|p| p.join("README.txt"));
    if let Some(readme) = readme {
        if !readme.exists() {
            let text = concat!(
                "Note Reader — local columns folder\n\n",
                "Put each column as a subfolder under columns/, e.g.:\n",
                "  columns/MySQL/01 intro.md\n\n",
                "You can change the read path in the app Admin page.\n"
            );
            let _ = fs::write(readme, text);
        }
    }
    Ok(())
}

pub fn platform_info(app: &AppHandle) -> PlatformInfo {
    let os = std::env::consts::OS.to_string();
    let home = app
        .path()
        .home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~".into());
    let data = data_home(app);
    let default_cols = default_columns_path(app);
    let install_hint = match os.as_str() {
        "macos" => {
            "macOS：应用通常装在「应用程序」；专栏内容默认在用户目录 NoteReader/columns。"
                .into()
        }
        "windows" => {
            "Windows：安装程序可自选安装路径；专栏读取路径默认在用户目录 NoteReader\\columns。"
                .into()
        }
        _ => "专栏内容默认在用户目录 NoteReader/columns。".into(),
    };
    PlatformInfo {
        os,
        home_dir: home,
        data_home: data.to_string_lossy().to_string(),
        default_columns_path: default_cols.to_string_lossy().to_string(),
        install_hint,
    }
}

pub fn fixed_columns_root(app: &AppHandle, state: &ContentState) -> PathBuf {
    let settings = state.settings.read().unwrap();
    if let Some(custom) = settings
        .columns_path
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return PathBuf::from(custom);
    }
    default_columns_path(app)
}

pub fn columns_root(app: &AppHandle, state: &ContentState) -> PathBuf {
    if let Some(session) = state.session_columns.read().unwrap().clone() {
        return session;
    }
    fixed_columns_root(app, state)
}

/// Parent of columns dir when named `columns`, otherwise the columns dir itself.
/// Used for `meta.json` so default layout keeps `content/meta.json`.
pub fn content_root(app: &AppHandle, state: &ContentState) -> PathBuf {
    let columns = columns_root(app, state);
    let is_named_columns = columns
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("columns"))
        .unwrap_or(false);
    if is_named_columns {
        columns
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| columns.clone())
    } else {
        columns
    }
}

pub fn meta_path(app: &AppHandle, state: &ContentState) -> PathBuf {
    let columns = columns_root(app, state);
    let is_named_columns = columns
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("columns"))
        .unwrap_or(false);
    if is_named_columns {
        content_root(app, state).join("meta.json")
    } else {
        columns.join("meta.json")
    }
}

pub fn path_info(app: &AppHandle, state: &ContentState) -> PathInfo {
    let default = default_columns_path(app);
    let fixed = fixed_columns_root(app, state);
    let columns = columns_root(app, state);
    let content = content_root(app, state);
    let is_session = state.session_columns.read().unwrap().is_some();
    let (is_custom, path_onboarded) = {
        let settings = state.settings.read().unwrap();
        let is_custom = settings
            .columns_path
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        (is_custom, settings.path_onboarded)
    };
    PathInfo {
        columns_path: columns.to_string_lossy().to_string(),
        fixed_columns_path: fixed.to_string_lossy().to_string(),
        default_path: default.to_string_lossy().to_string(),
        is_custom,
        is_session_browse: is_session,
        content_root: content.to_string_lossy().to_string(),
        path_onboarded,
        os: std::env::consts::OS.to_string(),
        data_home: data_home(app).to_string_lossy().to_string(),
    }
}

pub fn set_columns_path(
    app: &AppHandle,
    state: &ContentState,
    path: Option<String>,
) -> Result<PathInfo, String> {
    let normalized = path
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(ref p) = normalized {
        let pb = PathBuf::from(p);
        if pb.exists() && !pb.is_dir() {
            return Err("路径不是文件夹".into());
        }
        fs::create_dir_all(&pb).map_err(|e| format!("无法创建目录: {e}"))?;
    }

    {
        let mut settings = state.settings.write().unwrap();
        settings.columns_path = normalized;
        settings.path_onboarded = true;
        save_settings(app, &settings)?;
    }
    // Changing fixed path exits temporary browse.
    *state.session_columns.write().unwrap() = None;

    Ok(path_info(app, state))
}

/// Temporary browse: `Some(path)` enters session mode; `None` returns to fixed path.
/// Does not write settings.json.
pub fn set_session_columns_path(
    app: &AppHandle,
    state: &ContentState,
    path: Option<String>,
) -> Result<PathInfo, String> {
    match path {
        None => {
            *state.session_columns.write().unwrap() = None;
        }
        Some(p) => {
            let pb = PathBuf::from(p.trim());
            if !pb.exists() {
                return Err("目录不存在".into());
            }
            if !pb.is_dir() {
                return Err("路径不是文件夹".into());
            }
            *state.session_columns.write().unwrap() = Some(pb);
        }
    }
    Ok(path_info(app, state))
}

pub fn close_behavior(state: &ContentState) -> Option<String> {
    state
        .settings
        .read()
        .unwrap()
        .close_behavior
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| s == "tray" || s == "quit")
}

pub fn set_close_behavior(
    app: &AppHandle,
    state: &ContentState,
    behavior: Option<String>,
) -> Result<(), String> {
    let normalized = behavior
        .map(|s| s.trim().to_string())
        .filter(|s| s == "tray" || s == "quit");
    let mut settings = state.settings.write().unwrap();
    settings.close_behavior = normalized;
    save_settings(app, &settings)
}

pub fn complete_path_onboarding(
    app: &AppHandle,
    state: &ContentState,
    path: Option<String>,
) -> Result<PathInfo, String> {
    if let Some(p) = path {
        set_columns_path(app, state, Some(p))
    } else {
        // Keep default path; just mark onboarded and ensure dirs exist
        ensure_default_dirs(app)?;
        {
            let mut settings = state.settings.write().unwrap();
            settings.path_onboarded = true;
            // Clear custom so default is used
            if settings
                .columns_path
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                settings.columns_path = None;
            }
            save_settings(app, &settings)?;
        }
        Ok(path_info(app, state))
    }
}

fn title_from_md(text: &str, fallback: &str) -> String {
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
        if !t.is_empty() {
            break;
        }
    }
    fallback.to_string()
}

fn natural_key(name: &str) -> Vec<NaturalPart> {
    let mut parts = Vec::new();
    let mut num = String::new();
    let mut text = String::new();
    for ch in name.chars() {
        if ch.is_ascii_digit() {
            if !text.is_empty() {
                parts.push(NaturalPart::Text(text.to_lowercase()));
                text.clear();
            }
            num.push(ch);
        } else {
            if !num.is_empty() {
                parts.push(NaturalPart::Num(num.parse::<u64>().unwrap_or(0)));
                num.clear();
            }
            text.push(ch);
        }
    }
    if !num.is_empty() {
        parts.push(NaturalPart::Num(num.parse::<u64>().unwrap_or(0)));
    }
    if !text.is_empty() {
        parts.push(NaturalPart::Text(text.to_lowercase()));
    }
    parts
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NaturalPart {
    Num(u64),
    Text(String),
}

fn allowed_roots(app: &AppHandle, state: &ContentState) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    let columns = columns_root(app, state);
    if let Ok(c) = columns.canonicalize() {
        roots.push(c);
    } else {
        roots.push(columns);
    }
    let content = content_root(app, state);
    if let Ok(c) = content.canonicalize() {
        if !roots.iter().any(|r| r == &c) {
            roots.push(c);
        }
    }
    Ok(roots)
}

fn path_allowed(app: &AppHandle, state: &ContentState, path: &Path) -> Result<PathBuf, String> {
    let canon = path
        .canonicalize()
        .map_err(|e| format!("路径无效: {e}"))?;
    let roots = allowed_roots(app, state)?;
    if roots.iter().any(|root| canon.starts_with(root)) {
        return Ok(canon);
    }
    Err("路径不在当前专栏目录内".into())
}

pub fn scan_columns(app: &AppHandle, state: &ContentState) -> Result<Vec<ColumnMeta>, String> {
    let root = columns_root(app, state);
    if !root.exists() {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }

    let mut columns = Vec::new();
    let mut dirs: Vec<PathBuf> = fs::read_dir(&root)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    dirs.sort_by(|a, b| {
        let an = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let bn = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
        natural_key(an).cmp(&natural_key(bn))
    });

    for dir in dirs {
        let slug = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && article_kind(p).is_some())
            .collect();

        files.sort_by(|a, b| {
            let an = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let bn = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
            natural_key(an).cmp(&natural_key(bn))
        });

        let mut articles = Vec::new();
        for file in files {
            let kind = article_kind(&file).unwrap_or("markdown").to_string();
            let filename = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled");
            let (title, chars) = if kind == "markdown" {
                let text = fs::read_to_string(&file).unwrap_or_default();
                (title_from_md(&text, stem), text.chars().count())
            } else {
                let meta = fs::metadata(&file).ok();
                let size = meta.map(|m| m.len() as usize).unwrap_or(0);
                (stem.to_string(), size)
            };
            articles.push(ArticleMeta {
                title,
                filename,
                path: file.to_string_lossy().to_string(),
                chars,
                kind,
            });
        }

        columns.push(ColumnMeta {
            title: slug.clone(),
            slug,
            dir: dir.to_string_lossy().to_string(),
            article_count: articles.len(),
            articles,
        });
    }

    Ok(columns)
}

pub fn read_article(app: &AppHandle, state: &ContentState, path: &str) -> Result<String, String> {
    let canon = path_allowed(app, state, Path::new(path))?;
    match article_kind(&canon) {
        Some("markdown") | None => fs::read_to_string(&canon).map_err(|e| e.to_string()),
        Some(other) => Err(format!("请使用二进制读取接口打开 {other} 文件")),
    }
}

pub fn read_file_bytes(app: &AppHandle, state: &ContentState, path: &str) -> Result<Vec<u8>, String> {
    let canon = path_allowed(app, state, Path::new(path))?;
    fs::read(&canon).map_err(|e| e.to_string())
}

/// Absolute path suitable for `convertFileSrc` on the frontend (PDF preview).
pub fn resolve_file_path(app: &AppHandle, state: &ContentState, path: &str) -> Result<String, String> {
    let canon = path_allowed(app, state, Path::new(path))?;
    Ok(canon.to_string_lossy().to_string())
}

pub fn load_meta(app: &AppHandle, state: &ContentState) -> Result<MetaStore, String> {
    let path = meta_path(app, state);
    if !path.exists() {
        return Ok(MetaStore {
            tags: vec![],
            column_tags: Default::default(),
        });
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_meta(app: &AppHandle, state: &ContentState, meta: &MetaStore) -> Result<(), String> {
    let path = meta_path(app, state);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(&path, text + "\n").map_err(|e| e.to_string())
}

pub fn resolve_asset(
    app: &AppHandle,
    state: &ContentState,
    column_dir: &str,
    relative: &str,
) -> Result<String, String> {
    let rel = relative.trim().trim_start_matches("./");
    let full = Path::new(column_dir).join(rel);
    let canon = path_allowed(app, state, &full).map_err(|e| format!("asset not found: {rel} ({e})"))?;
    Ok(canon.to_string_lossy().to_string())
}
