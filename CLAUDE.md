# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Z-CPP is a lightweight Tauri v2 desktop IDE for C/C++ (algorithm-competition oriented). The app is a **Rust backend shell** (`src-tauri/`) driving a **React + Vite + Ant Design + Monaco** single-page frontend (`frontend/`). There is a legacy Axum HTTP backend in `backend/` that is **no longer built or run** — leave it alone.

## Architecture (big picture)

- **No HTTP between frontend and backend.** The frontend calls Rust via `invoke()` from `@tauri-apps/api/core` (`frontend/src/services/api.ts` is the typed wrapper). The `/api` proxy in `vite.config.ts` is legacy and inert under Tauri.
- **12 IPC commands** are registered in `src-tauri/src/lib.rs` (e.g. `compile_code`, `list_files`, `save_settings`, `save_session`, `get_system_fonts`). Each maps 1:1 to a function in `api.ts`. The two algorithm-IDE commands — `run_testcases` (compile once, run N cases, compare to expected via `normalize_output`) and `stress_test` (compile solution/reference/generator, loop generating input and diffing outputs) — build on the reusable `compile::compile_program` + `compile::run_capture` helpers in `compile.rs`.
- **App state** is `Mutex<Settings>` in `AppState` (`lib.rs`), loaded from `z-cpp-settings.json` (next to the exe) and re-saved on `save_settings`.
- **Frontend is a single page** (`App.tsx`, no router). Theme + appearance (including the background scrim) and Monaco setup all live in `App.tsx`; `styles.css` only holds the dark base + scrollbar + transparent editor background.
- **Build orchestration** is in `src-tauri/tauri.conf.json`: `beforeDevCommand` / `beforeBuildCommand` run from the project root via `npm --prefix frontend run ...`. Tauri invokes these itself — do **not** add a separate frontend build step in CI (it would run twice).

## Common commands

```bash
npx tauri dev                 # Vite HMR + Rust incremental compile + native window (root dir)
npx tauri build               # release bundles in src-tauri/target/release/bundle/
npm --prefix frontend run build:renderer   # tsc -b && vite build (also the beforeBuildCommand)
npx tsc -b --noEmit           # frontend typecheck only (run from frontend/)
cargo check                   # backend typecheck (from src-tauri/)
```

There is **no test framework, no lint script, and no formatter config** (Rust uses default `cargo fmt`; frontend has no prettier/eslint). `tsc -b` only runs as part of `build:renderer`.

## Settings — backward compatibility is a hard rule

Settings are defined in **two places that must stay in sync**: `src-tauri/src/models.rs` (`Settings` / `AppearanceSettings` / `EditorSettings` / `CompileOptions`) and `frontend/src/services/api.ts`. When you add or change a setting field:

1. Add it to **both** the Rust struct and the TS interface.
2. Give every new Rust field a `#[serde(default = "...")]` + a `default_*` fn, and include it in the `Default` impl. Old `z-cpp-settings.json` files lacking the field must deserialize without error.
3. Wire the UI: add an `edit*` state + load/init + save in `App.tsx` (see how `scrim_auto` / `scrim_opacity` are threaded).

Appearance model: `AppearanceSettings` holds `background_image`, `background_opacity`, `opacity` (window), `frosted_glass`, `blur_amount`, `scrim_auto`, `scrim_opacity`. The background **scrim layer** is a single full-screen overlay rendered above the background image and below the UI in `App.tsx`; its color follows the editor theme (dark→black, light→white) and its strength is auto-derived from background-image luminance (canvas sampling) unless `scrim_auto` is off. There is deliberately **no per-panel hardcoded overlay** — panels go transparent when a background is active so the unified scrim provides contrast everywhere, including Monaco.

## CI / release

`.github/workflows/build.yml` uses `tauri-apps/tauri-action@v0` on a matrix of `windows-latest` (MSVC) + `ubuntu-22.04` (Linux). Push to `main` builds + uploads artifacts; pushing a `v*` tag creates a GitHub Release. A `concurrency` group cancels superseded branch/PR runs but **never** cancels tag runs. npm deps are cached via `setup-node` and installed with `npm ci`. Linux needs `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf` (installed in the workflow).

## Gotchas

- C/C++ compilation at runtime depends on `g++`/`clang++` in `PATH`, or paths set via settings. The IDE does not bundle a compiler.
- Workspace location: `workspace/` by default; overridable by settings or the `ZCPP_WORKSPACE` env var.
- If `crates.io` is unreachable, the Rust mirror is configured in `~/.cargo/config.toml` (tuna sparse index) — see `AGENTS.md`.
- Git remote is `ssh://git@ssh.github.com:443/cn-ywcw/Z-CPP.git` (SSH on port 443).
- See `AGENTS.md` for the full architecture table, Rust mirror config, and capability/permission file locations (`src-tauri/capabilities/default.json`).
