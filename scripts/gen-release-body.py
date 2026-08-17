#!/usr/bin/env python3
"""Generate release notes from the commits included in this version."""
import os
import subprocess


version = os.environ.get("VERSION", "latest")
try:
    tags = subprocess.run(
        ["git", "tag", "--sort=-v:refname"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    previous_tag = next((tag for tag in tags if tag != f"v{version}"), "")
    revision_range = f"{previous_tag}..HEAD" if previous_tag else "HEAD"
    changes = subprocess.run(
        ["git", "log", "--no-merges", "--pretty=format:- %s (%h)", revision_range],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
except (OSError, subprocess.CalledProcessError):
    changes = ""

if not changes:
    changes = "- 本次版本的更新内容请参见提交历史。"

body = f"""## 更新内容

{changes}

## 下载

请从下方 Assets 下载对应平台的安装包：

- Windows：`.exe`（NSIS）或 `.msi`
- Linux：`.deb` 或 `.AppImage`

当前版本：`{version}`
"""

with open("release-body.md", "w", encoding="utf-8") as f:
    f.write(body)

print(f"Release body generated for {version}")
print(f"Written to release-body.md ({len(body)} bytes)")
