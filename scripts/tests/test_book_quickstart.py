import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_book_quickstart import active_toolchain, check_quickstart, one_fence


class QuickstartTests(unittest.TestCase):
    def test_extracts_exact_manifest_and_program(self):
        markdown = '```sh\ncargo run\n```\n```toml\n[package]\n```\n```rust\nfn main() {}\n```\n'
        self.assertEqual(one_fence(markdown, "toml"), "[package]\n")
        self.assertEqual(one_fence(markdown, "rust"), "fn main() {}\n")

    def test_missing_or_ambiguous_blocks_fail(self):
        for markdown in ["no snippet", "```rust\nfirst\n```\n```rust\nsecond\n```\n"]:
            with self.subTest(markdown=markdown), self.assertRaises(ValueError):
                one_fence(markdown, "rust")

    def make_checkout(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        (root / "docs").mkdir()
        (root / "docs/running-examples.md").write_text(
            '```toml\n[dependencies]\nstillwater = { path = "../stillwater" }\n```\n'
            '```rust\nfn main() {}\n```\n', encoding="utf-8")
        return root

    def test_temporary_project_uses_checkout_toolchain_and_exact_snippets(self):
        root = self.make_checkout()
        toolchain = "1.89.0-aarch64-apple-darwin"

        def run(command, **kwargs):
            if command == ["rustup", "show", "active-toolchain"]:
                self.assertEqual(kwargs["cwd"], root)
                self.assertTrue(kwargs["check"])
                return subprocess.CompletedProcess(command, 0, f"{toolchain} (directory override)\n")
            self.assertEqual(command, ["cargo", "run", "--quiet"])
            self.assertEqual(kwargs["env"]["RUSTUP_TOOLCHAIN"], toolchain)
            self.assertEqual(kwargs["env"]["CARGO_TARGET_DIR"], str(root / "target/book-quickstart"))
            self.assertTrue(kwargs["check"])
            project = kwargs["cwd"]
            self.assertNotEqual(project.parent, root)
            self.assertEqual((project.parent / "stillwater").resolve(), root.resolve())
            self.assertEqual((project / "Cargo.toml").read_text(),
                             '[dependencies]\nstillwater = { path = "../stillwater" }\n')
            self.assertEqual((project / "src/main.rs").read_text(), "fn main() {}\n")
            return subprocess.CompletedProcess(command, 0)

        with patch.dict(os.environ, {}, clear=True), \
             patch("check_book_quickstart.subprocess.run", side_effect=run) as mocked:
            check_quickstart(root)
        self.assertEqual(mocked.call_count, 2)

    def test_toolchain_resolution_failure_stops_before_cargo(self):
        root = self.make_checkout()
        error = subprocess.CalledProcessError(1, ["rustup", "show", "active-toolchain"])
        with patch("check_book_quickstart.subprocess.run", side_effect=error) as mocked:
            with self.assertRaises(subprocess.CalledProcessError):
                check_quickstart(root)
        self.assertEqual(mocked.call_args.args[0], ["rustup", "show", "active-toolchain"])
        mocked.assert_called_once()

    def test_empty_toolchain_response_fails(self):
        result = subprocess.CompletedProcess([], 0, "\n")
        with patch("check_book_quickstart.subprocess.run", return_value=result):
            with self.assertRaisesRegex(ValueError, "no active toolchain"):
                active_toolchain(Path("checkout"))

    def test_explicit_toolchain_override_is_left_for_rustup_to_resolve(self):
        result = subprocess.CompletedProcess([], 0, "nightly-test (environment override)\n")
        with patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "nightly-test"}), \
             patch("check_book_quickstart.subprocess.run", return_value=result) as mocked:
            self.assertEqual(active_toolchain(Path("checkout")), "nightly-test")
        self.assertNotIn("env", mocked.call_args.kwargs)

    def test_cargo_failure_is_not_swallowed(self):
        root = self.make_checkout()
        selected = subprocess.CompletedProcess([], 0, "1.89.0 (directory override)\n")
        failure = subprocess.CalledProcessError(1, ["cargo", "run", "--quiet"])
        with patch("check_book_quickstart.subprocess.run", side_effect=[selected, failure]):
            with self.assertRaises(subprocess.CalledProcessError):
                check_quickstart(root)


if __name__ == "__main__":
    unittest.main()
