#!/usr/bin/env bash
# Z-CPP 启动脚本 (Linux)
# 用法: ./start.sh [port]
set -euo pipefail

PORT="${1:-3000}"
DIR="$(cd "$(dirname "$0")" && pwd)"

cd "$DIR"
mkdir -p workspace

export ZCPP_MODE=production

echo "========================================="
echo "  Z-CPP 轻量级 C/C++ IDE"
echo "  端口: $PORT"
echo "  工作目录: $DIR/workspace"
echo "  浏览器打开: http://localhost:$PORT"
echo "========================================="
echo ""

exec "$DIR/z-cpp-backend" --port "$PORT"
