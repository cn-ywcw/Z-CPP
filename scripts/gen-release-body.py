#!/usr/bin/env python3
"""Generate Z-CPP release body for GitHub Releases."""
import os

version = os.environ.get("GITHUB_REF_NAME", "latest")
body = f"""## 🚀 Z-CPP 轻量级 C/C++ IDE (v{version})

> 面向算法竞赛选手的单文件 C/C++ 集成开发环境。

### 📦 安装包说明

| 文件 | 平台 | 用法 |
|------|------|------|
| `z-cpp-linux-x86_64.tar.gz` | Linux | 解压 → `./start.sh` |
| `z-cpp-windows-x86_64.zip`  | Windows | 解压 → 双击 `start.bat` |

### 🛠 系统要求

- **编译器**：系统需预装 **GCC (g++)** 或 **Clang (clang++)**
  - Linux: `sudo apt install g++` 或 `sudo apt install clang`
  - Windows MinGW: 下载 [MinGW-w64](https://www.mingw-w64.org/) 或使用 WSL
- **浏览器**：Chrome / Edge / Firefox 最新版

### 💡 快速开始

```bash
# Linux
tar xzf z-cpp-linux-x86_64.tar.gz
cd z-cpp-linux-x86_64
./start.sh

# Windows — 解压后双击 start.bat
```

浏览器打开 **http://localhost:3000** 即可开始写代码！

### ✨ 功能

- Monaco Editor 代码编辑（语法高亮、行号）
- GCC / Clang 编译器切换
- 单文件编译运行 + 仅编译模式
- 运行时间显示
- 暗色主题，竞赛风格

---

*自动发布 by GitHub Actions*
"""

with open("release-body.md", "w", encoding="utf-8") as f:
    f.write(body)

print(f"Release body generated for {version}")
print(f"Written to release-body.md ({len(body)} bytes)")
