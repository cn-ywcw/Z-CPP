#!/usr/bin/env python3
"""Generate Z-CPP release body for GitHub Releases."""
import os

version = os.environ.get("GITHUB_REF_NAME", "latest")
body = f"""## 🚀 Z-CPP 轻量级 C/C++ IDE (v{version})

> 面向算法竞赛选手的单文件 C/C++ 集成开发环境。

---

### 📥 下载

| 文件 | 平台 | 类型 | 用法 |
|------|------|------|------|
| `z-cpp-linux-x86_64.tar.gz` | Linux | 绿色版 | 解压 → `./start.sh` |
| `z-cpp_0.1.0_amd64.deb` | Debian/Ubuntu | 安装包 | `sudo dpkg -i *.deb` |
| `z-cpp-0.1.0-1.x86_64.rpm` | Fedora/CentOS | 安装包 | `sudo rpm -ivh *.rpm` |
| `z-cpp-windows-x86_64.zip` | Windows | 绿色版 | 解压 → 双击 `start.bat` |
| `Z-CPP-Setup-*.exe` | Windows | 安装包 | 双击安装 |

---

### 🚀 启动步骤

#### Linux 绿色版
```bash
tar xzf z-cpp-linux-x86_64.tar.gz
cd z-cpp-linux-x86_64
./start.sh        # 默认端口 3000
./start.sh 8080   # 自定义端口
```
浏览器打开 **http://localhost:3000**

#### Linux 安装包（.deb）
```bash
sudo dpkg -i z-cpp_0.1.0_amd64.deb
# 安装后直接运行：
z-cpp   # 通过 PATH 中的符号链接启动
```

#### Linux 安装包（.rpm）
```bash
sudo rpm -ivh z-cpp-0.1.0-1.x86_64.rpm
z-cpp
```

#### Windows 绿色版
```cmd
解压 z-cpp-windows-x86_64.zip → 双击 start.bat
```

#### Windows 安装包
```cmd
双击 Z-CPP-Setup-*.exe → 按提示安装
安装后可通过开始菜单或桌面快捷方式启动
```

---

### 🛠 系统要求

| 组件 | 要求 |
|------|------|
| **编译器** | GCC (g++) 或 Clang (clang++)，需自行安装 |
| **浏览器** | Chrome / Edge / Firefox 最新版 |

**安装编译器：**

- **Linux (Ubuntu/Debian):** `sudo apt install g++` 或 `sudo apt install clang`
- **Windows MinGW:** 下载 [MinGW-w64](https://www.mingw-w64.org/) 或使用 WSL
- **macOS:** `xcode-select --install`（自带 Clang）

首次启动后，在 IDE 设置页面（⚙️）配置编译器路径。

---

### ✨ 功能

- ✅ **Monaco Editor** — 语法高亮、行号、自动补全、暗色主题
- ✅ **多文件支持** — 文件浏览器 + 标签页切换 + 新建文件
- ✅ **GCC / Clang 双编译器** — 一键切换
- ✅ **编译选项面板** — 优化等级 (O0~O3/Os/Ofast)、C++ 标准 (11~23)、警告级别、自定义参数
- ✅ **MinGW 路径配置** — 自定义 GCC/Clang 路径 + 工作目录
- ✅ **单文件编译运行** — 编译 + 运行 + 仅编译模式
- ✅ **运行耗时显示** — 毫秒级精度
- ✅ **设置持久化** — 配置保存到 `z-cpp-settings.json`

---

### ⚙️ 自定义配置

启动后点击顶部 ⚙️ 按钮打开设置：

| 配置项 | 说明 |
|--------|------|
| GCC 路径 | 自定义 g++ 位置，留空使用 PATH |
| Clang 路径 | 自定义 clang++ 位置，留空使用 PATH |
| 工作目录 | 代码存放目录，默认 `workspace/` |
| 默认编译选项 | 优化等级、标准、警告的默认值 |

---

*📝 自动发布 by GitHub Actions — 完整提交历史见下方*
"""

with open("release-body.md", "w", encoding="utf-8") as f:
    f.write(body)

print(f"Release body generated for {version}")
print(f"Written to release-body.md ({len(body)} bytes)")
