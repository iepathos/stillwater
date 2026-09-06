from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_book_playground import check_local_book


def rendered(code="use stillwater::Semigroup;", code_class="language-rust",
             container_class="playground", tag="pre"):
    return (f'<{tag} class="{container_class}">'
            f'<code class="{code_class}">{code}</code></{tag}>')


class PlaygroundTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.book = Path(self.temporary.name) / "book"
        self.book.mkdir()
        self.page = self.book / "index.html"

    def check(self, html):
        self.page.write_text(html, encoding="utf-8")
        return check_local_book(self.book)

    def run_cli(self, *args):
        script = Path(__file__).resolve().parents[1] / "check_book_playground.py"
        return subprocess.run([sys.executable, "-B", str(script), str(self.book), *args],
                              capture_output=True, text=True, timeout=10)

    def test_semigroup_without_extern_crate_is_rejected(self):
        report = self.check(rendered())
        self.assertEqual((report.blocks, len(report.errors)), (1, 1))
        self.assertIn("index.html:1", report.errors[0])

    def test_any_playground_container_is_rejected(self):
        for tag in ["pre", "div", "section", "code"]:
            with self.subTest(tag=tag):
                self.assertEqual(self.check(rendered(tag=tag)).blocks, 1)

    def test_doctest_classes_cannot_bypass_local_policy(self):
        for code_class in ["language-rust no_run", "language-rust compile_fail",
                           "language-rust ignore"]:
            with self.subTest(code_class=code_class):
                self.assertEqual(self.check(rendered(code_class=code_class)).blocks, 1)

    def test_even_std_only_playground_blocks_are_rejected(self):
        self.assertEqual(self.check(rendered("assert_eq!(2 + 2, 4);")).blocks, 1)

    def test_class_tokens_and_entities_are_recognized(self):
        for classes in ["example playground extra", "playground\textra", "play&#103;round"]:
            with self.subTest(classes=classes):
                self.assertEqual(self.check(rendered(container_class=classes)).blocks, 1)

    def test_non_playground_code_is_allowed(self):
        for classes in ["", "example", "not-playground", "playground-extra"]:
            with self.subTest(classes=classes):
                report = self.check(rendered(container_class=classes))
                self.assertEqual((report.blocks, report.errors), (0, []))

    def test_comments_and_escaped_markup_are_not_elements(self):
        html = '<!-- <div class="playground"> --> &lt;pre class="playground"&gt;'
        self.assertEqual(self.check(html).errors, [])

    def test_nested_and_empty_elements_are_counted_with_locations(self):
        report = self.check('<div class="playground">\n<pre class="playground"></pre>\n</div>')
        self.assertEqual(report.blocks, 2)
        self.assertIn("index.html:1", report.errors[0])
        self.assertIn("index.html:2", report.errors[1])

    def test_self_closing_and_unclosed_elements_are_rejected(self):
        for html in ['<div class="playground"/>', '<pre class="playground">']:
            with self.subTest(html=html):
                self.assertEqual(self.check(html).blocks, 1)

    def test_nested_pages_and_print_output_are_checked(self):
        (self.book / "guide").mkdir()
        (self.book / "guide/chapter.html").write_text(rendered(), encoding="utf-8")
        (self.book / "print.html").write_text(rendered(), encoding="utf-8")
        report = self.check(rendered(container_class=""))
        self.assertEqual(report.blocks, 2)
        self.assertTrue(any("chapter.html:1" in error for error in report.errors))
        self.assertTrue(any("print.html:1" in error for error in report.errors))

    def test_missing_or_empty_book_fails(self):
        for path in [self.book / "missing", self.book]:
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, "No rendered HTML"):
                check_local_book(path)

    def test_cli_defaults_to_local_policy_with_actionable_failure(self):
        self.check(rendered())
        result = self.run_cli()
        self.assertEqual(result.returncode, 1)
        self.assertIn("index.html:1", result.stderr)
        self.assertIn("runnable = false", result.stderr)

    def test_cli_passes_local_book(self):
        self.check(rendered(container_class=""))
        result = self.run_cli()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("0 rendered playground", result.stdout)

    def test_cli_missing_output_fails(self):
        result = self.run_cli()
        self.assertEqual(result.returncode, 1)
        self.assertIn("No rendered HTML", result.stderr)

    def test_removed_inventory_options_are_rejected(self):
        for args in [("--inventory", "inventory.json"), ("--manifest", "Cargo.toml")]:
            with self.subTest(args=args):
                result = self.run_cli(*args)
                self.assertEqual(result.returncode, 2)
                self.assertIn("unrecognized arguments", result.stderr)


if __name__ == "__main__":
    unittest.main()
