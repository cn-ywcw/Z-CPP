#!/usr/bin/env bash
# Z-CPP .rpm 包构建脚本
# 用法: ./build-rpm.sh <backend-binary> <frontend-dist> <output-dir>
set -euo pipefail

BACKEND="$1"
FRONTEND="$2"
OUTPUT_DIR="$3"
PKG_NAME="z-cpp"
PKG_VERSION="${PKG_VERSION:-0.1.0}"
ARCH="x86_64"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RPM_BUILD_DIR="$(mktemp -d)"
trap "rm -rf $RPM_BUILD_DIR" EXIT

echo "构建 .rpm 包..."
echo "  后端: $BACKEND"
echo "  前端: $FRONTEND"
echo "  输出: $OUTPUT_DIR"

# RPM 需要特定目录结构
mkdir -p "$RPM_BUILD_DIR/BUILD"
mkdir -p "$RPM_BUILD_DIR/RPMS/$ARCH"
mkdir -p "$RPM_BUILD_DIR/SOURCES"
mkdir -p "$RPM_BUILD_DIR/SPECS"
mkdir -p "$RPM_BUILD_DIR/SRPMS"

# 源码打包
SOURCES_DIR="$RPM_BUILD_DIR/SOURCES/${PKG_NAME}-${PKG_VERSION}"
mkdir -p "$SOURCES_DIR"
cp "$BACKEND" "$SOURCES_DIR/z-cpp-backend"
mkdir -p "$SOURCES_DIR/frontend/dist"
cp -r "$FRONTEND"/* "$SOURCES_DIR/frontend/dist/"
cp "$SCRIPT_DIR/../../start.sh" "$SOURCES_DIR/"
cp "$SCRIPT_DIR/../../README.md" "$SOURCES_DIR/"
cp "$SCRIPT_DIR/z-cpp.desktop" "$SOURCES_DIR/"
chmod +x "$SOURCES_DIR/start.sh"
chmod +x "$SOURCES_DIR/z-cpp-backend"

# 生成 spec 文件
cat > "$RPM_BUILD_DIR/SPECS/${PKG_NAME}.spec" << EOF
Name: ${PKG_NAME}
Version: ${PKG_VERSION}
Release: 1
Summary: 轻量级 C/C++ IDE — 面向算法竞赛选手

Group: Development/Tools
License: MIT
URL: https://github.com/cn-ywcw/Z-CPP
BuildArch: ${ARCH}
Requires: gcc-c++ >= 8, glibc >= 2.28, libstdc++ >= 8

%description
Z-CPP 是一个专为算法竞赛选手设计的轻量级单文件 C/C++ 集成开发环境。

特性:
* Monaco Editor 代码编辑（语法高亮、行号、自动补全）
* 多文件支持（文件浏览器 + 标签页）
* GCC / Clang 双编译器切换
* 编译选项面板（优化等级、C++ 标准、警告级别）
* MinGW 路径配置
* 运行耗时显示
* 暗色主题

%install
mkdir -p %{buildroot}/opt/z-cpp/frontend/dist
mkdir -p %{buildroot}/opt/z-cpp/workspace
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/usr/share/applications

cp %{_sourcedir}/z-cpp-backend %{buildroot}/opt/z-cpp/
cp -r %{_sourcedir}/frontend/dist/* %{buildroot}/opt/z-cpp/frontend/dist/
cp %{_sourcedir}/start.sh %{buildroot}/opt/z-cpp/
cp %{_sourcedir}/README.md %{buildroot}/opt/z-cpp/
cp %{_sourcedir}/z-cpp.desktop %{buildroot}/usr/share/applications/
ln -sf /opt/z-cpp/start.sh %{buildroot}/usr/local/bin/z-cpp
chmod +x %{buildroot}/opt/z-cpp/start.sh
chmod +x %{buildroot}/opt/z-cpp/z-cpp-backend

%files
%defattr(-,root,root,-)
/opt/z-cpp/
/usr/local/bin/z-cpp
/usr/share/applications/z-cpp.desktop

%post
if [ ! -e /opt/z-cpp/workspace ]; then
    mkdir -p /opt/z-cpp/workspace
fi

%preun
if [ -L /usr/local/bin/z-cpp ]; then
    rm -f /usr/local/bin/z-cpp
fi
EOF

# 执行 rpmbuild
rpmbuild --define "_topdir $RPM_BUILD_DIR" \
         -bb "$RPM_BUILD_DIR/SPECS/${PKG_NAME}.spec"

# 复制结果
mkdir -p "$OUTPUT_DIR"
cp "$RPM_BUILD_DIR/RPMS/$ARCH/${PKG_NAME}-${PKG_VERSION}-1.${ARCH}.rpm" "$OUTPUT_DIR/"

echo "✓ .rpm 包构建完成:"
ls -lh "$OUTPUT_DIR/"*.rpm
