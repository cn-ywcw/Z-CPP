# Z-CPP — 轻量级 C/C++ IDE

> 面向算法竞赛选手的轻量级单文件 C/C++ 集成开发环境。
>
> **前端**：React + TypeScript + Ant Design + Monaco Editor
> **后端**：Rust (Axum)
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
│   │   └── services/       # API 调用封装
│   ├── package.json
│   └── vite.config.ts
├── backend/                # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口 + HTTP 路由
│   │   ├── compile.rs      # 编译逻辑
│   │   └── models.rs       # 数据模型
│   └── Cargo.toml
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

#### 启动后端

```bash
# 首次需下载依赖
cd backend
cargo build

# 启动开发服务器（默认监听 http://127.0.0.1:3000）
cargo run
```

#### 启动前端

```bash
cd frontend
npm install
npm run dev
```

浏览器打开 `http://localhost:5173` 即可使用。

#### 生产模式构建

```bash
# 构建前端
cd frontend && npm run build

# 构建后端（release）
cd ../backend && cargo build --release

# 启动（生产模式）
ZCPP_MODE=production ./target/release/z-cpp-backend
# 浏览器打开 http://localhost:3000
```

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
