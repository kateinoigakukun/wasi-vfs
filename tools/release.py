#!/usr/bin/env python3
"""Automate wasi-vfs releases: bump versions, update lockfile, commit, tag, push."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


def run(cmd, *, check=True, capture_output=False, text=True, cwd=None):
    return subprocess.run(
        cmd,
        check=check,
        capture_output=capture_output,
        text=text,
        cwd=cwd,
    )


def git_root() -> Path:
    out = run(["git", "rev-parse", "--show-toplevel"], capture_output=True).stdout.strip()
    return Path(out)


def assert_clean_tree():
    staged = run(
        ["git", "diff", "--quiet", "--ignore-submodules", "--cached"], check=False
    ).returncode
    if staged != 0:
        sys.exit("Index has staged changes. Please commit or unstage them first.")

    unstaged = run(
        ["git", "diff", "--quiet", "--ignore-submodules"], check=False
    ).returncode
    if unstaged != 0:
        sys.exit("Working tree has uncommitted changes. Please commit or stash them first.")


def read_version(manifest: Path) -> str:
    import tomllib

    with manifest.open("rb") as f:
        data = tomllib.load(f)
    try:
        return data["package"]["version"]
    except KeyError as exc:
        raise SystemExit(f"No [package].version found in {manifest}") from exc


def gather_manifests() -> list[Path]:
    import json

    meta = json.loads(
        run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            capture_output=True,
        ).stdout
    )
    paths = {Path(p["manifest_path"]) for p in meta["packages"]}
    return sorted(paths)


def bump_version(current: str, mode: str) -> str:
    major, minor, patch, *_ = (list(map(int, (current.split(".") + ["0", "0"])))[:3])
    if mode == "major":
        major, minor, patch = major + 1, 0, 0
    elif mode == "minor":
        minor, patch = minor + 1, 0
    elif mode == "patch":
        patch += 1
    else:
        raise SystemExit(f"Unknown bump mode: {mode}")
    return f"{major}.{minor}.{patch}"


def tag_exists(version: str) -> bool:
    return (
        run(["git", "rev-parse", "-q", "--verify", f"refs/tags/v{version}"], check=False).returncode
        == 0
    )


@dataclass
class ReleasePlan:
    old_version: str
    new_version: str
    manifests: list[Path]
    remote: str
    branch: str


def make_plan(args) -> ReleasePlan:
    root = git_root()
    old_version = read_version(root / "Cargo.toml")

    if args.bump:
        new_version = bump_version(old_version, args.bump)
    elif args.version:
        new_version = args.version
    else:
        sys.exit("A new version is required (pass an explicit version or --bump).")

    if new_version == old_version:
        sys.exit(f"New version matches current version ({old_version}); nothing to do.")

    if tag_exists(new_version):
        sys.exit(f"Tag v{new_version} already exists; aborting.")

    manifests = gather_manifests()
    branch = (
        run(["git", "symbolic-ref", "--short", "HEAD"], capture_output=True)
        .stdout.strip()
    )
    remote = os.environ.get("REMOTE", "origin")
    return ReleasePlan(old_version, new_version, manifests, remote, branch)


def update_versions(plan: ReleasePlan) -> list[Path]:
    updated: list[Path] = []
    for manifest in plan.manifests:
        text = manifest.read_text()
        import tomllib

        pkg = tomllib.loads(text).get("package")
        if not pkg or pkg.get("version") != plan.old_version:
            continue

        import re

        replaced, count = re.subn(
            rf'(?m)^version\s*=\s*"{re.escape(plan.old_version)}"',
            f'version = "{plan.new_version}"',
            text,
            count=1,
        )
        if count == 0:
            continue
        manifest.write_text(replaced)
        updated.append(manifest)
    if not updated:
        sys.exit(f"No manifests were updated; expected to find {plan.old_version}.")
    return updated


def regenerate_lock():
    run(["cargo", "generate-lockfile"])


def commit_and_tag(new_version: str, files: list[Path]):
    run(["git", "add", "Cargo.lock", *map(str, files)])
    run(["git", "commit", "-m", f"Release v{new_version}"])
    run(["git", "tag", "-a", f"v{new_version}", "-m", f"Release v{new_version}"])


def confirm_and_push(plan: ReleasePlan, dry_run: bool):
    print("\nReady to push:")
    print(f"  - Branch: {plan.branch} -> {plan.remote}")
    print(f"  - Tag:    v{plan.new_version} -> {plan.remote}")
    resp = input("Push now? [y/N] ").strip()
    if resp.lower() != "y":
        print("Push skipped. Commit and tag exist locally.")
        return
    if dry_run:
        print("[dry-run] Skipping push.")
        return
    run(["git", "push", plan.remote, plan.branch])
    run(["git", "push", plan.remote, f"v{plan.new_version}"])


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Automate wasi-vfs release (version bump, lock, commit, tag, push)."
    )
    parser.add_argument(
        "--bump", choices=["major", "minor", "patch"], help="derive new version from current"
    )
    parser.add_argument("--dry-run", action="store_true", help="perform everything except the push")
    parser.add_argument("version", nargs="?", help="explicit new version (if not using --bump)")
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv)
    assert_clean_tree()
    plan = make_plan(args)
    print(f"Bumping version: {plan.old_version} -> {plan.new_version}")
    updated_manifests = update_versions(plan)
    regenerate_lock()
    commit_and_tag(plan.new_version, updated_manifests)
    confirm_and_push(plan, args.dry_run)


if __name__ == "__main__":
    main()
