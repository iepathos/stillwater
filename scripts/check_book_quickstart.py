"""Run the exact documented Cargo manifest and main.rs in a fresh sibling project."""

import os
from pathlib import Path
import re
import subprocess
import tempfile


def one_fence(markdown, language):
    blocks = re.findall(rf"^```{language}\n(.*?)^```\s*$", markdown, re.MULTILINE | re.DOTALL)
    if len(blocks) != 1:
        raise ValueError(f"Quickstart needs exactly one complete {language} block, got {len(blocks)}")
    return blocks[0]


def active_toolchain(root):
    result = subprocess.run(["rustup", "show", "active-toolchain"], cwd=root,
                            stdout=subprocess.PIPE, text=True, check=True, timeout=30)
    fields = result.stdout.split()
    if not fields:
        raise ValueError(f"rustup reported no active toolchain for {root}")
    return fields[0]


def check_quickstart(root):
    toolchain = active_toolchain(root)
    print(f"Running standalone quickstart with Rust toolchain {toolchain}", flush=True)
    guide = (root / "docs/running-examples.md").read_text(encoding="utf-8")
    with tempfile.TemporaryDirectory(prefix="stillwater-book-quickstart-") as temporary:
        directory = Path(temporary)
        (directory / "stillwater").symlink_to(root, target_is_directory=True)
        project = directory / "stillwater-book-example"
        (project / "src").mkdir(parents=True)
        (project / "Cargo.toml").write_text(one_fence(guide, "toml"), encoding="utf-8")
        (project / "src/main.rs").write_text(one_fence(guide, "rust"), encoding="utf-8")
        # The checkout's directory override does not follow us into the temp project.
        env = dict(os.environ, RUSTUP_TOOLCHAIN=toolchain,
                   CARGO_TARGET_DIR=str(root / "target/book-quickstart"))
        # Allow dependency fetching just like the reader's documented `cargo run`.
        subprocess.run(["cargo", "run", "--quiet"], cwd=project, env=env, check=True, timeout=180)


if __name__ == "__main__":
    check_quickstart(Path(__file__).resolve().parents[1])
