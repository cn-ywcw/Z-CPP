# Z-CPP — Agent Guide

## Architecture

**双架构共存** — 主入口是 Tauri v2 桌面壳 (`src-tauri/`)，旧版 Axum HTTP 后端 (`backend/`) 保留但不再使用。

| 层次 | 位置 | 关键事实 |
|------|------|----------|
| 前端 | `frontend/` | React + Vite + Ant Design + Monaco Editor |
| 桌面壳 | `src-tauri/` | Tauri v2, `build.rs` 用 `tauri-build`, `lib.rs` 注册 12 个 IPC 命令 |
| 旧后端 | `backend/` | 遗留 Axum 服务，代码保留但不再构建或运行 |

**通信**: 前端通过 `@tauri-apps/api/core` → `invoke()` 调用 Rust 命令，不走 HTTP（`vite.config.ts` 中 `/api` proxy 是 legacy，Tauri 模式下不生效）。

**入口点**:
- Rust: `src-tauri/src/main.rs` → `z_cpp_lib::run()` → `src-tauri/src/lib.rs`:run()
- 前端: `frontend/src/main.tsx` → `App.tsx`（单页无路由）

## 关键命令

```bash
# 开发（项目根目录）
npx tauri dev               # Vite dev server + Rust 增量编译 + 原生窗口

# 生产构建
npx tauri build             # 产物: src-tauri/target/release/bundle/

# 前端单独构建
npm --prefix frontend run build:renderer   # tsc -b && vite build
```

`beforeDevCommand` / `beforeBuildCommand` 在项目根目录执行，完整路径用 `npm --prefix frontend run ...`。

## CI

`.github/workflows/build.yml` 使用 `tauri-apps/tauri-action@v0` 构建 Tauri 产物，构建矩阵：`windows-latest` (MSVC) + `ubuntu-22.04` (Linux)。打 tag `v*` 自动创建 Release。

## 配置与持久化

- 运行时设置写入 `z-cpp-settings.json`（exe 同目录）
- 用户代码存放 `workspace/`（可在设置中修改路径；环境变量 `ZCPP_WORKSPACE` 可覆写）
- Tauri 配置: `src-tauri/tauri.conf.json`
- Tauri 权限: `src-tauri/capabilities/default.json`

## Rust 依赖

```bash
# 若 crates.io 网络不可达，配置镜像源：
# ~/.cargo/config.toml
[source.crates-io]
replace-with = "tuna"

[source.tuna]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
```

## 事项

- 无测试框架、无测试文件
- 无 lint/typecheck 脚本（`tsc -b` 是 `build:renderer` 的一部分）
- 无格式化配置 — Rust 用默认 fmt，前端无 prettier/eslint
- `backend/` 的 `Cargo.toml` 独立于 `src-tauri/`，无 cargo workspace
- 编译依赖系统 PATH 中的 `g++`/`clang++`，或通过设置指定自定义路径
- GitHub 远程: `ssh://git@ssh.github.com:443/cn-ywcw/Z-CPP.git`（SSH 端口 443）
