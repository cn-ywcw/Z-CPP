# Z-CPP — Agent Guide

## Architecture

**双架构共存** — 主入口是 Tauri v2 桌面壳 (`src-tauri/`)，旧版 Axum HTTP 后端 (`backend/`) 保留但不再作为主入口。

| 层次 | 位置 | 关键事实 |
|------|------|----------|
| 前端 | `frontend/` | React + Vite + Ant Design + Monaco Editor |
| 桌面壳 | `src-tauri/` | Tauri v2, `build.rs` 用 `tauri-build`, `lib.rs` 注册所有 IPC 命令 |
| 旧后 | `backend/` | 遗留 Axum 服务，CI 仍在构建但用户不用了 |

**通信**: 前端通过 `@tauri-apps/api/core` → `invoke()` 调用 Rust 命令（`src-tauri/src/lib.rs` 中全部 10 个），不再走 HTTP。

**入口点**:
- Rust: `src-tauri/src/main.rs` → `z_cpp_lib::run()` → `src-tauri/src/lib.rs`:run()
- 前端: `frontend/src/main.tsx` → `App.tsx`（单页应用，无路由）
- 业务逻辑: `src-tauri/src/compile.rs`（编译/运行），`models.rs`（数据结构）

## 关键命令

```bash
# 开发（项目根目录）
npx tauri dev           # 启动 Vite dev server + Rust 增量编译 + 原生窗口

# 生产构建
npx tauri build         # 产物: src-tauri/target/release/bundle/

# 前端单独构建
npm --prefix frontend run build:renderer
```

`beforeDevCommand` 和 `beforeBuildCommand` 在项目根目录执行，完整路径用 `npm --prefix frontend run ...`。

## CI 警告

`.github/workflows/build.yml` **仍在使用旧版 `backend/` 架构**（Axum HTTP + 独立前端）。它不执行 `src-tauri/` 的 Tauri 构建。如需 CI 构建 Tauri 产物，必须重写该 workflow。

## 配置与持久化

- 运行时设置写入 `z-cpp-settings.json`（exe 同目录）
- 用户代码存放 `workspace/`（可在设置中修改路径）
- Tauri 配置: `src-tauri/tauri.conf.json`（窗口大小、图标、frontendDist、beforeDevCommand）
- Tauri 权限: `src-tauri/capabilities/default.json`

## 杂项

- 设置持久化路径是 exe 所在目录，资源管理器右侧输出面板
- 编译依赖系统 PATH 中的 `g++`/`clang++`，或通过设置指定自定义路径
- `vite.config.ts` 仍保留 `/api` proxy（legacy），Tauri 模式下不生效，可忽略

## 不在此仓库中的

- **无测试** — 无测试框架、无测试文件
- **无 lint/typecheck 脚本** — `tsc -b` 是 `build:renderer` 的一部分
- **无 monorepo 工具** — 两个 Cargo 包独立（`src-tauri/Cargo.toml` + `backend/Cargo.toml`），无 workspace
- **无格式化配置** — Rust 用默认 fmt，前端无 prettier/eslint

## Git

```bash
git add . && git commit -m "feat: 描述" && git push origin main
```
远程: `ssh://git@ssh.github.com:443/cn-ywcw/Z-CPP.git`（SSH 端口 443）。
