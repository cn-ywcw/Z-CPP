# 贡献与版本发布规则

## 默认版本规则

每个会进入 `main` 的代码提交，若没有特殊说明，默认将补丁版本号加 `0.0.1`：

```bash
python3 scripts/version.py bump
```

例如 `0.1.0` 会变为 `0.1.1`。脚本会同步更新 `VERSION`、Tauri/Rust、前端和安装包元数据；提交前可用下面的命令检查是否一致：

```bash
python3 scripts/version.py check
```

需要发布次版本或主版本时，可以明确执行：

```bash
python3 scripts/version.py bump minor
python3 scripts/version.py bump major
```

`[no-version-bump]` 仅用于文档、CI 或其他明确不影响软件版本的提交，并应在提交信息或 PR 描述中说明原因。没有这个标记的 `main` 代码提交如果没有更新 `VERSION`，CI 会直接提示失败。

## 构建与发布

- 推送到 `main` 且提交包含版本变化时，CI 默认构建 Windows（NSIS/MSI）和 Linux（deb/AppImage）安装包。
- 两个平台都构建成功后，CI 自动创建 `v{version}` 标签和标题为 `Z-CPP {version}` 的 GitHub Release，并将安装包作为 Assets 上传。
- Release 内容由本次版本包含的提交信息自动生成。
- 只改文档等不在构建路径中的提交不会触发构建。
- 对虽命中构建路径但不需要构建的提交，在提交信息中加入 `[skip build]` 或 `[no-build]`；`[skip ci]` 也会跳过。
- 也可以在 GitHub Actions 的 `workflow_dispatch` 中手动选择是否构建；`publish` 默认关闭，只有明确选择时才发布 Release。

## 提交示例

```bash
python3 scripts/version.py bump
git add .
git commit -m "feat: add compiler option"
git push origin main
```
