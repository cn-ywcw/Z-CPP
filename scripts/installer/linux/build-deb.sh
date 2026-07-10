#!/usr/bin/env bash
# Z-CPP .deb 包构建脚本
# 用法: ./build-deb.sh <backend-binary> <frontend-dist> <output-dir>
set -euo pipefail

BACKEND="$1"       # 编译好的 z-cpp-backend 二进制
FRONTEND="$2"      # frontend/dist 目录
OUTPUT_DIR="$3"    # 输出目录
PKG_NAME="z-cpp"
PKG_VERSION="${PKG_VERSION:-0.1.0}"
ARCH="amd64"
DEB_NAME="${PKG_NAME}_${PKG_VERSION}_${ARCH}.deb"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$(mktemp -d)"
trap "rm -rf $BUILD_DIR" EXIT

echo "构建 .deb 包..."
echo "  后端: $BACKEND"
echo "  前端: $FRONTEND"
echo "  输出: $OUTPUT_DIR/$DEB_NAME"

# 创建 .deb 目录结构
INSTALL_DIR="$BUILD_DIR/opt/z-cpp"
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/frontend/dist"
mkdir -p "$INSTALL_DIR/workspace"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/local/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"

# 复制文件
cp "$BACKEND" "$INSTALL_DIR/z-cpp-backend"
cp -r "$FRONTEND"/* "$INSTALL_DIR/frontend/dist/"
cp "$SCRIPT_DIR/start.sh" "$INSTALL_DIR/"
cp "$SCRIPT_DIR/../../README.md" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/start.sh"
chmod +x "$INSTALL_DIR/z-cpp-backend"

# 复制 DEBIAN 控制文件
cp "$SCRIPT_DIR/DEBIAN/control" "$BUILD_DIR/DEBIAN/"
cp "$SCRIPT_DIR/DEBIAN/postinst" "$BUILD_DIR/DEBIAN/"
cp "$SCRIPT_DIR/DEBIAN/prerm" "$BUILD_DIR/DEBIAN/"
chmod 755 "$BUILD_DIR/DEBIAN/postinst"
chmod 755 "$BUILD_DIR/DEBIAN/prerm"

# 符号链接 (供 /usr/local/bin/z-cpp)
ln -sf /opt/z-cpp/start.sh "$BUILD_DIR/usr/local/bin/z-cpp"

# 桌面入口
cp "$SCRIPT_DIR/../z-cpp.desktop" "$BUILD_DIR/usr/share/applications/"

# 生成 .deb
mkdir -p "$OUTPUT_DIR"
dpkg-deb --build "$BUILD_DIR" "$OUTPUT_DIR/$DEB_NAME"

echo "✓ .deb 包构建完成: $OUTPUT_DIR/$DEB_NAME"
echo "  大小: $(du -h "$OUTPUT_DIR/$DEB_NAME" | cut -f1)"
