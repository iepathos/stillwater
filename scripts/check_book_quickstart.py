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


def check_quickstart(root):
    guide = (root / "docs/running-examples.md").read_text(encoding="utf-8")
    with tempfile.TemporaryDirectory(prefix="stillwater-book-quickstart-") as temporary:
        directory = Path(temporary)
        (directory / "stillwater").symlink_to(root, target_is_directory=True)
        project = directory / "stillwater-book-example"
        (project / "src").mkdir(parents=True)
        (project / "Cargo.toml").write_text(one_fence(guide, "toml"), encoding="utf-8")
        (project / "src/main.rs").write_text(one_fence(guide, "rust"), encoding="utf-8")
        env = dict(os.environ, CARGO_TARGET_DIR=str(root / "target/book-quickstart"))
        # Allow dependency fetching just like the reader's documented `cargo run`.
        subprocess.run(["cargo", "run", "--quiet"], cwd=project, env=env, check=True, timeout=180)


if __name__ == "__main__":
    check_quickstart(Path(__file__).resolve().parents[1])
