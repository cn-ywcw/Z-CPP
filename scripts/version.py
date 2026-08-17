#!/usr/bin/env python3
"""Read, update, and validate the version used by Z-CPP."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
VERSION_FILE = ROOT / "VERSION"
SEMVER = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")


def read_version() -> str:
    version = VERSION_FILE.read_text(encoding="utf-8").strip()
    if not SEMVER.fullmatch(version):
        raise SystemExit(f"Invalid version in VERSION: {version!r}")
    return version


def replace_once(path: Path, pattern: str, replacement: str, expected: int = 1) -> None:
    text = path.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, replacement, text, count=expected, flags=re.MULTILINE)
    if count != expected:
        raise SystemExit(f"Expected {expected} version occurrence(s) in {path}, found {count}")
    path.write_text(updated, encoding="utf-8")


def version_files(version: str) -> None:
    # Keep package metadata and the Tauri/Rust package metadata in lockstep.
    replace_once(ROOT / "package.json", r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(ROOT / "package-lock.json", r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{version}\g<2>", expected=2)
    replace_once(ROOT / "frontend/package.json", r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(ROOT / "frontend/package-lock.json", r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{version}\g<2>", expected=2)
    replace_once(ROOT / "src-tauri/tauri.conf.json", r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(ROOT / "src-tauri/Cargo.toml", r'(^version\s*=\s*")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(
        ROOT / "src-tauri/Cargo.lock",
        r'(^name\s*=\s*"z-cpp"\s*\nversion\s*=\s*")[^"]+(")',
        rf"\g<1>{version}\g<2>",
    )
    replace_once(ROOT / "backend/Cargo.toml", r'(^version\s*=\s*")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(
        ROOT / "backend/Cargo.lock",
        r'(^name\s*=\s*"z-cpp-backend"\s*\nversion\s*=\s*")[^"]+(")',
        rf"\g<1>{version}\g<2>",
    )
    replace_once(ROOT / "scripts/installer/linux/DEBIAN/control", r'(^Version:\s*)\S+', rf"\g<1>{version}")
    replace_once(ROOT / "scripts/installer/windows/setup.nsi", r'(^!define PRODUCT_VERSION\s+")[^"]+(")', rf"\g<1>{version}\g<2>")
    replace_once(ROOT / "scripts/installer/linux/build-deb.sh", r'(^PKG_VERSION=\"\$\{PKG_VERSION:-)[^}]+(\}\")', rf"\g<1>{version}\g<2>")
    replace_once(ROOT / "scripts/installer/linux/build-rpm.sh", r'(^PKG_VERSION=\"\$\{PKG_VERSION:-)[^}]+(\}\")', rf"\g<1>{version}\g<2>")


def check() -> None:
    version = read_version()
    checks = {
        "package.json": (r'"version"\s*:\s*"([^"]+)"', 1),
        "package-lock.json": (r'"version"\s*:\s*"([^"]+)"', 2),
        "frontend/package.json": (r'"version"\s*:\s*"([^"]+)"', 1),
        "frontend/package-lock.json": (r'"version"\s*:\s*"([^"]+)"', 2),
        "src-tauri/tauri.conf.json": (r'"version"\s*:\s*"([^"]+)"', 1),
        "src-tauri/Cargo.toml": (r'^version\s*=\s*"([^"]+)"', 1),
        "src-tauri/Cargo.lock": (r'^name\s*=\s*"z-cpp"\s*\nversion\s*=\s*"([^"]+)"', 1),
        "backend/Cargo.toml": (r'^version\s*=\s*"([^"]+)"', 1),
        "backend/Cargo.lock": (r'^name\s*=\s*"z-cpp-backend"\s*\nversion\s*=\s*"([^"]+)"', 1),
        "scripts/installer/linux/DEBIAN/control": (r'^Version:\s*(\S+)', 1),
        "scripts/installer/windows/setup.nsi": (r'^!define PRODUCT_VERSION\s+"([^"]+)"', 1),
        "scripts/installer/linux/build-deb.sh": (r'^PKG_VERSION="\$\{PKG_VERSION:-([^}]+)\}"', 1),
        "scripts/installer/linux/build-rpm.sh": (r'^PKG_VERSION="\$\{PKG_VERSION:-([^}]+)\}"', 1),
    }
    mismatches = []
    for relative_path, (pattern, expected_count) in checks.items():
        matches = re.findall(pattern, (ROOT / relative_path).read_text(encoding="utf-8"), re.MULTILINE)
        if len(matches) < expected_count or any(value != version for value in matches[:expected_count]):
            mismatches.append(f"{relative_path}={matches[:expected_count]}")
    if mismatches:
        raise SystemExit(f"Version mismatch: VERSION={version}; {', '.join(mismatches)}")
    print(version)


def main() -> None:
    command = sys.argv[1] if len(sys.argv) > 1 else "check"
    if command == "check":
        check()
        return

    if command == "set":
        if len(sys.argv) != 3 or not SEMVER.fullmatch(sys.argv[2]):
            raise SystemExit("Usage: scripts/version.py set MAJOR.MINOR.PATCH")
        version = sys.argv[2]
    elif command == "bump":
        level = sys.argv[2] if len(sys.argv) > 2 else "patch"
        if level not in {"major", "minor", "patch"}:
            raise SystemExit("Usage: scripts/version.py bump [major|minor|patch]")
        parts = [int(part) for part in read_version().split(".")]
        index = {"major": 0, "minor": 1, "patch": 2}[level]
        parts[index] += 1
        for reset in range(index + 1, 3):
            parts[reset] = 0
        version = ".".join(str(part) for part in parts)
    else:
        raise SystemExit("Usage: scripts/version.py [check|bump [major|minor|patch]|set VERSION]")

    VERSION_FILE.write_text(version + "\n", encoding="utf-8")
    version_files(version)
    check()
    print(f"Updated Z-CPP to {version}")


if __name__ == "__main__":
    main()
