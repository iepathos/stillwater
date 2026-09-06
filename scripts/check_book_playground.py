"""Enforce local-only example execution in freshly rendered mdBook HTML.

Reject every element with mdBook's playground class, independently of its tag,
contents, or doctest attributes. Cargo doctests check the examples themselves.
"""

import argparse
from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path
import sys


@dataclass
class Report:
    blocks: int = 0
    errors: list[str] = field(default_factory=list)


class PlaygroundParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.lines = []

    def handle_starttag(self, tag, attrs):
        classes = (dict(attrs).get("class") or "").split()
        # mdBook's JavaScript selects .playground, not just pre.playground.
        if "playground" in classes:
            self.lines.append(self.getpos()[0])


def playground_lines(html):
    parser = PlaygroundParser()
    parser.feed(html)
    parser.close()
    return parser.lines


def rendered_blocks(book):
    pages = sorted(book.rglob("*.html"))
    if not pages:
        raise ValueError(f"No rendered HTML in {book}; build the book before checking it")
    for page in pages:
        for line in playground_lines(page.read_text(encoding="utf-8")):
            yield page, line


def check_local_book(book):
    report = Report()
    for page, line in rendered_blocks(book):
        report.blocks += 1
        report.errors.append(f"{page}:{line}: playground element in a local-only book; "
                             "keep output.html.playground.runnable = false and remove "
                             "custom playground markup")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("book", type=Path, help="Freshly generated mdBook HTML directory")
    args = parser.parse_args()
    try:
        report = check_local_book(args.book)
    except (OSError, ValueError) as error:
        print(f"Local-execution check failed: {error}", file=sys.stderr)
        return 1
    print(f"Found {report.blocks} rendered playground elements")
    if report.errors:
        print("\n".join(report.errors), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
