import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_book_playground import check_book, check_local_book, main, parse_inventory, playground_blocks


MANIFEST = '''[package]
name = "stillwater"
version = "2.0.0"
[dev-dependencies]
tokio-test = "0.4"
'''


def rendered(code, code_class="language-rust", pre_class="playground"):
    return f'<pre class="{pre_class}"><code class="{code_class}">{code}</code></pre>'


class PlaygroundTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.manifest = self.root / "Cargo.toml"
        self.manifest.write_text(MANIFEST, encoding="utf-8")
        self.book = self.root / "book"
        self.book.mkdir()
        self.page = self.book / "index.html"

    def check(self, code, inventory=None, **classes):
        self.page.write_text(rendered(code, **classes), encoding="utf-8")
        return check_book(self.book, self.manifest, inventory or {})

    def test_semigroup_regression_without_extern_crate(self):
        report = self.check('''use stillwater::Semigroup;
let v1 = vec![1, 2, 3];
let v2 = vec![4, 5, 6];
assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);
let empty: Vec&lt;i32&gt; = vec![];
let values = vec![1, 2, 3];
assert_eq!(empty.combine(values), vec![1, 2, 3]);''')
        self.assertEqual(report.blocks, 1)
        self.assertEqual(len(report.errors), 1)
        self.assertIn("index.html:1", report.errors[0])
        self.assertIn("stillwater is unavailable", report.errors[0])

    def test_published_old_version_does_not_validate_new_docs(self):
        report = self.check("use stillwater::Semigroup;", {"stillwater": "1.1.1"})
        self.assertIn("requires stillwater 2.0.0", report.errors[0])

    def test_matching_version_passes_dependency_check(self):
        report = self.check("use stillwater::Semigroup;", {"stillwater": "2.0.0"})
        self.assertEqual(report.errors, [])

    def test_hidden_lines_and_highlighted_paths_are_checked(self):
        code = '<span class="boring">use <span>stillwater</span>::Semigroup;</span>'
        self.assertIn("stillwater is unavailable", self.check(code).errors[0])

    def test_fully_qualified_and_aliased_paths_are_checked(self):
        for code in ["stillwater::pure(1);", "use stillwater :: {Semigroup as S};",
                     "extern crate stillwater as sw;"]:
            with self.subTest(code=code):
                self.assertEqual(len(self.check(code).errors), 1)

    def test_helpers_need_their_own_dependencies(self):
        report = self.check("tokio_test::block_on(async {});", {"stillwater": "2.0.0"})
        self.assertIn("tokio_test is unavailable", report.errors[0])

    def test_non_runnable_blocks_are_not_playground_promises(self):
        for code_class in ["language-rust no_run", "language-rust compile_fail",
                           "language-rust ignore"]:
            with self.subTest(code_class=code_class):
                report = self.check("use stillwater::Semigroup;", code_class=code_class)
                self.assertEqual((report.blocks, report.errors), (0, []))
        report = self.check("use stillwater::Semigroup;", pre_class="")
        self.assertEqual((report.blocks, report.errors), (0, []))

    def test_std_only_example_passes(self):
        self.assertEqual(self.check("assert_eq!(2 + 2, 4);").errors, [])

    def test_local_policy_rejects_any_reintroduced_run_controls(self):
        for code in ["use stillwater::Semigroup;", "assert_eq!(2 + 2, 4);"]:
            with self.subTest(code=code):
                self.page.write_text(rendered(code), encoding="utf-8")
                self.assertIn("browser Run control", check_local_book(self.book).errors[0])

    def test_local_policy_needs_no_network(self):
        self.page.write_text(rendered("use stillwater::Semigroup;", pre_class=""),
                             encoding="utf-8")
        with patch("sys.argv", ["checker", str(self.book), "--local-only"]), \
             patch("check_book_playground.load_inventory", side_effect=AssertionError("network")):
            self.assertEqual(main(), 0)

    def test_local_policy_rejects_missing_book(self):
        with self.assertRaises(ValueError):
            check_local_book(self.root / "missing")

    def test_missing_or_empty_book_fails(self):
        for path in [self.root / "missing", self.book]:
            with self.subTest(path=path), self.assertRaises(ValueError):
                check_book(path, self.manifest, {})

    def test_parser_preserves_hidden_text_entities_and_line_numbers(self):
        blocks = playground_blocks('\n' + rendered('<span class="boring">&amp;x</span>'))
        self.assertEqual([(block.line, block.code) for block in blocks], [(2, "&x")])

    def test_inventory_schema_is_checked(self):
        self.assertEqual(parse_inventory({"crates": [{"id": "tokio", "version": "1.0"}]}),
                         {"tokio": "1.0"})
        for invalid in [{}, {"crates": []}, {"crates": [{}]},
                        {"crates": [{"id": "tokio", "version": None}]}]:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                parse_inventory(invalid)

    def test_cli_returns_failure_and_actionable_location(self):
        self.page.write_text(rendered("use stillwater::Semigroup;"), encoding="utf-8")
        inventory = self.root / "inventory.json"
        inventory.write_text(json.dumps({"crates": [{"id": "tokio", "version": "1"}]}),
                             encoding="utf-8")
        script = Path(__file__).resolve().parents[1] / "check_book_playground.py"
        result = subprocess.run(
            [sys.executable, str(script), str(self.book), "--manifest", str(self.manifest),
             "--inventory", str(inventory)], capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("index.html:1", result.stderr)
        self.assertIn("stillwater is unavailable", result.stderr)


if __name__ == "__main__":
    unittest.main()
