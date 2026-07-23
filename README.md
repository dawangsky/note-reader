# Note Reader

Vue 3 + Tauri local desktop reader. Release builds load Markdown columns from `~/NoteReader/columns/`; development uses the repo `content/columns/`.

## Layout

```
note-reader/
  content/            # Dev columns
    columns/
    meta.json
  src/
  src-tauri/
```

Release data directory (ASCII paths only):

| OS | Default columns path |
|------|----------------|
| macOS / Linux | `~/NoteReader/columns` |
| Windows | `%USERPROFILE%\NoteReader\columns` |

## Add columns

Put folders under `columns/`, for example:

```
NoteReader/columns/MyColumn/
  01 intro.md
  02 body.md
```

First launch confirms the read path; you can change it anytime in Admin. On Windows, the installer chooses the **install path** separately from the **read path**.

## Dev / run

```bash
npm install
npm run tauri:dev
```

Build:

```bash
npm run tauri:build
```

- macOS: `.app` / `.dmg`; content defaults to `~/NoteReader`
- Windows: NSIS can pick per-user/machine and install directory; post-install creates `NoteReader\columns` under the user profile
