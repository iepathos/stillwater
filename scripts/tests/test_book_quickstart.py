from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_book_quickstart import one_fence


class QuickstartTests(unittest.TestCase):
    def test_extracts_exact_manifest_and_program(self):
        markdown = '```sh\ncargo run\n```\n```toml\n[package]\n```\n```rust\nfn main() {}\n```\n'
        self.assertEqual(one_fence(markdown, "toml"), "[package]\n")
        self.assertEqual(one_fence(markdown, "rust"), "fn main() {}\n")

    def test_missing_or_ambiguous_blocks_fail(self):
        for markdown in ["no snippet", "```rust\nfirst\n```\n```rust\nsecond\n```\n"]:
            with self.subTest(markdown=markdown), self.assertRaises(ValueError):
                one_fence(markdown, "rust")


if __name__ == "__main__":
    unittest.main()
