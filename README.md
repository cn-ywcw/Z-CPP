# Z-CPP — 轻量级 C/C++ IDE

> 面向算法竞赛选手的轻量级单文件 C/C++ 集成开发环境。
>
> **前端**：React + TypeScript + Ant Design + Monaco Editor
> **后端**：Rust (Tauri v2)
> **桌面框架**：Tauri v2（系统 WebView）
> **编译器支持**：GCC / Clang

---

## 📖 开发手册

### 🚨 Git 提交规范

**每次修改代码后，必须使用 SSH 推送至 GitHub。** 禁止使用 HTTPS 方式推送。

```bash
# 查看当前变更
git status

# 添加所有变更
git add .

# 提交（使用有意义的提交信息）
git commit -m "feat: 修改说明"

# 推送到 GitHub（SSH 方式）
git push origin main
```

> 远程仓库已配置为 `ssh://git@ssh.github.com:443/cn-ywcw/Z-CPP.git`

### 📁 项目结构

```
F:/Z-CPP/
├── README.md               # 本手册
├── frontend/               # React 前端
│   ├── src/
│   │   ├── App.tsx         # 主应用组件
│   │   ├── components/     # 通用组件
│   │   ├── pages/          # 页面组件
│   │   └── services/       # API 调用封装（Tauri IPC）
│   ├── package.json
│   └── vite.config.ts
├── src-tauri/              # Tauri v2 桌面壳
│   ├── src/
│   │   ├── main.rs         # 入口（隐藏控制台窗口）
│   │   ├── lib.rs          # Tauri 应用 + 命令注册
│   │   ├── compile.rs      # 编译逻辑
│   │   └── models.rs       # 数据模型
│   ├── tauri.conf.json     # Tauri 配置
│   ├── capabilities/       # 权限声明
│   ├── icons/              # 应用图标
│   └── Cargo.toml
├── backend/                # （旧）独立 HTTP 后端
└── workspace/              # 用户代码工作目录（存放待编译的 .c/.cpp 文件）
```

### 🛠 本地开发

#### 前置依赖

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Rust | 1.70+ | 后端编译 |
| Node.js | 18+ | 前端构建 |
| GCC (g++) | 任意 | C/C++ 编译（默认） |
| Clang (clang++) | 任意 | C/C++ 编译（可选） |
| WebView2 | Win10+ 自带 | Windows 桌面渲染（Tauri） |

#### 启动开发模式（热更新）

```bash
# 1. 安装前端依赖
cd frontend && npm install

# 2. 启动 Tauri 开发模式（自动启动 Vite + Rust 热重载）
cd .. && npm run tauri dev
# 或使用 npx:
npx tauri dev
```

Tauri 会自动：
1. 执行 `beforeDevCommand` 启动 Vite 开发服务器（端口 5173）
2. 编译 Rust 后端（增量编译）
3. 打开系统原生窗口加载前端页面
4. Rust 代码更改自动重启，前端代码实时热更新

#### 仅启动前端（用于 UI 调试）

```bash
cd frontend && npm run dev
# 浏览器打开 http://localhost:5173
# 需要同时启动旧版 HTTP 后端提供 API
```

#### 生产构建

```bash
# 一次性构建桌面安装包
cd frontend && npm install && cd ..
npx tauri build
```

产物位于 `src-tauri/target/release/bundle/`：
- Windows: `.msi` / `.exe`
- macOS: `.dmg` / `.app`
- Linux: `.deb` / `.AppImage`

> 构建流程：`beforeBuildCommand` → Vite 编译前端 → `frontend/dist/` → Rust 编译 → Tauri 嵌入静态资源到二进制 → 打包

### 🚀 CI/CD 自动构建

每次推送到 `main` 分支或创建 `v*` 标签时，GitHub Actions 会自动执行：

1. **构建前端** — `npm ci && npm run build`
2. **编译 Linux 后端** — `cargo build --release` (Ubuntu runner)
3. **编译 Windows 后端** — `cargo build --release` (Windows runner)
4. **打包发布** — 生成平台专属压缩包（内含二进制 + 前端 + 启动脚本）

| 平台 | 包名 | 内容 |
|------|------|------|
| Linux | `z-cpp-linux-x86_64.tar.gz` | 后端二进制 + 前端静态文件 + `start.sh` |
| Windows | `z-cpp-windows-x86_64.zip` | 后端 exe + 前端静态文件 + `start.bat` |

打 `git tag v0.1.0 && git push origin v0.1.0` 时自动创建 GitHub Release。

#### 使用安装包

```bash
# Linux
tar xzf z-cpp-linux-x86_64.tar.gz
cd z-cpp-linux-x86_64
./start.sh

# Windows
# 解压 zip，双击 start.bat
```

> ⚠ 系统需要预装 GCC 或 Clang 编译器。

## 🎯 核心功能

### MVP 功能清单

- [x] 代码编辑器（语法高亮、行号、自动补全）
- [x] 单文件 C/C++ 编译
- [x] 运行并捕获程序输出
- [x] 编译器切换（GCC / Clang）
- [ ] 编译错误解析与跳转
- [ ] 多种输入测试用例
- [ ] 运行时间测量

---

## 📝 改动记录

| 日期 | 改动说明 | 提交者 |
|------|---------|--------|
| 2026-07-10 | 初始化项目 + GitHub Actions CI/CD (Windows/Linux 自动打包) | — |

> 每次修改后运行 `git add . && git commit -m "描述" && git push origin main` 提交。
