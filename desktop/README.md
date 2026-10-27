# Desktop development alpha (macOS + Windows)

This is the first **working local feature slice** of desktop issue [#44](https://github.com/savisaluwadana/Pain-Point-Analyzer/issues/44): a native Tauri 2 window, SQLite storage under the signed-in operating-system account, and the **same Rust `keywords-v2` analyzer** as the server edition via `shared/scoring.rs`.

It is **not** a feature-complete port of the Docker application. The desktop edition currently does **not** implement permitted external source collectors, PostgreSQL data migration, browser OAuth, the 58 MCP workflows, native import/restore, unattended updates, cloud sync, or a signed public release. The current operating scope remains **individual, local, non-commercial research**. The desktop app must not be exposed as a web server.

## Current functionality

1. Launch native window without Node, Docker, PostgreSQL or an HTTP listener in the installed app.
2. Add your own short, authorized evidence summaries with optional HTTP(S) reference links; direct Reddit URL import is rejected.
3. Seed six **fictional** evidence items for an offline demo (idempotent).
4. Run the exact shared Rust `keywords-v2` grouping and relative priority scoring over **up to 1,000 latest** local items. Topic-aware lexical filtering can discard unrelated items. The result is not evidence of demand.
5. Review historical candidate themes, view still-present local evidence, and save user-authored alternative-solution notes plus five explicit decision stages. Deleted evidence is not copied into saved reports.
6. Delete an evidence record and export the current bounded workspace snapshot as a JSON file in the OS's app-data `exports` directory.

**Storage model:** `research.sqlite3` in `AppHandle.path().app_data_dir()`; the schema uses `PRAGMA user_version=1` and refuses databases with a newer version. The local SQLite app is intentionally independent from web PostgreSQL data; it does not silently import or delete PostgreSQL records. The export includes evidence and personal notes; it is **not** a lossless SQLite backup or an implemented restore function.

**Local security:** single operating-system user identity (OS permissions protect the files). Do not store confidential or unlicensed third-party content. No generic filesystem/shell plugins are enabled, and the app only exposes explicit local Rust IPC commands. Static evidence is rendered as escaped React text; source URLs are shown as text, not launched inside the webview.

## Development

Prerequisites: Node 22+, npm, Rust stable, and [Tauri 2 OS build prerequisites](https://v2.tauri.app/start/prerequisites/). On macOS install Xcode Command Line Tools; on Windows install Microsoft C++ Build Tools and ensure WebView2 is available.

```bash
cd desktop
npm install
npm run desktop:dev     # creates icons and starts the native app
```

```bash
npm run build           # typecheck + Vite bundle
npm run icons           # generate platform .ico/.icns and PNG icons from committed SVG
npm run desktop:build   # compile and bundle on your own target operating system
```

Direct `npm run dev` serves only the Vite preview, and cannot perform native IPC without Tauri.

Desktop Rust tests reuse the shared analyzer tests:

```bash
cargo test --manifest-path desktop/src-tauri/Cargo.toml
```

Run from the repository root. CI's desktop job runs these tests with a frontend bundle and generated icons already in place. No release signer/notarizer is configured yet; unsigned macOS/Windows artifacts should be treated as **internal test builds**, not ready-for-public downloads.

## Repository layout

- `desktop/src/App.tsx`: native React workspace and form states.
- `desktop/src-tauri/src/lib.rs`: explicit Tauri IPC commands, initialization and private file path.
- `desktop/src-tauri/src/db.rs`: SQLite schema/queries, transactions, validation and regression tests.
- `desktop/assets/app-icon.svg`: app icon source; generated binaries are build artifacts.
- `shared/scoring.rs`: dependency-light analyzer imported by the Rust API and Tauri desktop.
- `desktop/src-tauri/tauri.conf.json`: secure native window, static assets and installer configuration.
- `.github/workflows/desktop.yml`: native build and test jobs for macOS/Windows.

## Next work

- Replace the limited desktop-specific data model with parity-tested service/repository traits.
- Implement permitted source connectors with rate limits and no provider-policy bypass.
- Add secure credential storage, a backed-up import/restore format, and opt-in migration from the web edition.
- Restore desktop-side access to explicitly authorized MCP hosts.
- Add platform-specific installer smoke tests, accessibility and security reviews.
- Configure macOS signing/notarization and Windows signing before publishing general availability.
