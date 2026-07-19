# Z-CPP — 轻量级 C/C++ IDE

> 面向算法竞赛选手的轻量级桌面 C/C++ 集成开发环境。
>
> **前端**：React 18 + TypeScript + Ant Design 5 + Monaco Editor  
> **后端**：Rust (Tauri v2)  
> **编译器**：GCC / Clang  

---

<img width="1920" height="1040" alt="image" src="https://github.com/user-attachments/assets/c0e072e3-0fe9-4540-a068-defb787665d9" />

## 核心功能

- Monaco Editor 代码编辑（语法高亮、行号、自动补全、多标签）
- 单文件 C/C++ 编译 & 运行，捕获 stdout/stderr
- 编译器切换（GCC / Clang）、编译选项面板（优化等级、警告、标准版本、额外参数）
- 程序 stdin 输入支持（输出面板底部输入框）
- 文件浏览器（可拖拽调整宽度、刷新）
- 多标签编辑、自动保存、快捷键
- 系统字体枚举（font-kit）、编辑器主题切换（Dark / Light / High Contrast）
- 背景图片、毛玻璃效果、窗口透明度
- 背景遮罩层（自动适配图片明度 / 手动调节强度，浅色背景也能看清代码）
- 设置持久化（JSON 文件，向后兼容）
- 跨平台：Windows (NSIS/MSI)、Linux (deb/AppImage)

---

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+S` | 保存当前文件 |
| `Ctrl+N` | 新建文件 |
| `Ctrl+W` | 关闭当前标签 |
| `F5` | 编译运行 |
| `Ctrl+Shift+B` | 仅编译 |
| `Ctrl+,` | 打开设置 |
| `Ctrl+Tab` | 下一个标签 |
| `Ctrl+Shift+Tab` | 上一个标签 |

---

## 项目结构

```
Z-CPP/
├── frontend/                # React 前端
│   ├── src/
│   │   ├── App.tsx          # 主应用（单页）
│   │   ├── main.tsx         # 入口
│   │   ├── styles.css       # 全局暗色主题 CSS
│   │   └── services/api.ts  # Tauri IPC 封装
│   └── package.json
├── src-tauri/               # Tauri v2 桌面壳
│   ├── src/
│   │   ├── main.rs          # 入口
│   │   ├── lib.rs           # IPC 命令注册（12 个）
│   │   ├── compile.rs       # 编译/运行逻辑
│   │   └── models.rs        # 数据模型
│   ├── tauri.conf.json      # 窗口 & 构建配置
│   ├── capabilities/        # 权限声明
│   └── Cargo.toml
├── package.json             # 根（@tauri-apps/cli）
└── .github/workflows/       # CI/CD
```

---

## 本地开发

### 前置依赖

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.70+ | 后端 |
| Node.js | 22+ | 前端 |
| GCC / Clang | 任意 | C/C++ 编译 |
| WebView2 | Win10+ 自带 | 桌面渲染 |

### 启动

```bash
# 安装依赖
npm install
cd frontend && npm install && cd ..

# 开发模式（Vite 热更新 + Rust 增量编译）
npx tauri dev
```

### 构建安装包

```bash
npx tauri build
```

产物位于 `src-tauri/target/release/bundle/`：
- Windows: `*.exe` (NSIS) / `*.msi`
- Linux: `*.deb` / `*.AppImage`

---

## CI/CD

推送到 `main` 自动构建并发布安装包（GitHub Actions Artifacts）。

打 tag 自动创建 Release：

```bash
git tag v0.1.0
git push origin v0.1.0
```

构建矩阵：
- `windows-latest` → `x86_64-pc-windows-msvc`
- `ubuntu-22.04` → `x86_64-unknown-linux-gnu`

---

## 设置项

| 分类 | 设置 | 说明 |
|------|------|------|
| 编译器 | GCC/Clang 路径、默认编译器、额外参数、默认仅编译 | 编译行为 |
| 工作目录 | 路径 | 用户代码存放位置 |
| 编辑器 | 字体（系统字体下拉）、字号、Tab 大小、主题、自动换行、自动保存 | Monaco 编辑器 |
| 外观 | 背景图片、窗口透明度、毛玻璃效果、模糊程度 | 视觉效果 |
| 关于 | 版本、许可证、编译器状态 | 只读 |

所有设置持久化到 `z-cpp-settings.json`（exe 同目录），缺失字段自动填充默认值。

---

## 贡献者

感谢所有为 Z-CPP 做出贡献的开发者！

<a href="https://github.com/cn-ywcw/Z-CPP/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=cn-ywcw/Z-CPP" />
</a>

---

## Git 规范

```bash
git add . && git commit -m "feat: 描述" && git push origin main
```

远程：`ssh://git@ssh.github.com:443/cn-ywcw/Z-CPP.git`（SSH 端口 443）
