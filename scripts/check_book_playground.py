"""Check rendered mdBook playground dependencies, not just Markdown doctests.

This is a conservative dependency check, not a Rust parser or execution test.
It checks references to the package and its declared dependencies, including
hidden lines. Cargo doctests remain responsible for code and feature coverage.
"""

import argparse
from dataclasses import dataclass, field
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
import tomllib
from urllib.error import URLError
from urllib.request import urlopen


INVENTORY_URL = "https://play.rust-lang.org/meta/crates"


@dataclass
class Block:
    line: int
    code: str


@dataclass
class Report:
    blocks: int = 0
    errors: list[str] = field(default_factory=list)


class PlaygroundParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.blocks = []
        self.in_pre = False
        self.in_code = False
        self.runnable = False
        self.parts = []
        self.line = 0

    def handle_starttag(self, tag, attrs):
        classes = dict(attrs).get("class", "").split()
        if tag == "pre":
            self.in_pre = "playground" in classes
            self.runnable = False
            self.parts = []
            self.line = self.getpos()[0]
        elif tag == "code" and self.in_pre:
            self.in_code = True
            self.runnable = not {"no_run", "ignore", "compile_fail"}.intersection(classes)

    def handle_data(self, data):
        if self.in_code:
            self.parts.append(data)

    def handle_endtag(self, tag):
        if tag == "code":
            self.in_code = False
        elif tag == "pre":
            if self.in_pre and self.runnable:
                self.blocks.append(Block(self.line, "".join(self.parts)))
            self.in_pre = False
            self.in_code = False


def playground_blocks(html):
    parser = PlaygroundParser()
    parser.feed(html)
    parser.close()
    return parser.blocks


def parse_inventory(payload):
    crates = payload.get("crates") if isinstance(payload, dict) else None
    if not isinstance(crates, list) or not crates:
        raise ValueError("Playground inventory must contain a nonempty crates list")
    inventory = {}
    for crate in crates:
        if not isinstance(crate, dict) or not all(
            isinstance(crate.get(key), str) and crate[key] for key in ("id", "version")
        ):
            raise ValueError("Playground inventory entries need string id and version fields")
        inventory[crate["id"]] = crate["version"]
    return inventory


def load_inventory(path):
    if path:
        return parse_inventory(json.loads(path.read_text(encoding="utf-8")))
    with urlopen(INVENTORY_URL, timeout=20) as response:
        return parse_inventory(json.load(response))


def dependency_names(manifest):
    names = {manifest["package"]["name"]}
    for section in ("dependencies", "dev-dependencies"):
        names.update(manifest.get(section, {}))
    return {name.replace("-", "_") for name in names}


def references(code, name):
    # Search code rather than only `extern crate`, which misses Rust 2018+ imports.
    name = re.escape(name)
    return re.search(rf"\b{name}\s*::|\bextern\s+crate\s+{name}\b", code) is not None


def dependency_errors(code, manifest, inventory):
    package = manifest["package"]
    package_name = package["name"].replace("-", "_")
    for name in sorted(dependency_names(manifest)):
        if not references(code, name):
            continue
        if name not in inventory:
            yield f"{name} is unavailable in the public Rust Playground"
        elif name == package_name and inventory[name] != package["version"]:
            yield (f"book requires {name} {package['version']}, "
                   f"but the public Rust Playground has {inventory[name]}")


def rendered_blocks(book):
    pages = sorted(path for path in book.rglob("*.html") if path.name != "print.html")
    if not pages:
        raise ValueError(f"No rendered HTML in {book}; build the book before checking it")
    for page in pages:
        for block in playground_blocks(page.read_text(encoding="utf-8")):
            yield page, block


def check_local_book(book):
    report = Report()
    for page, block in rendered_blocks(book):
        report.blocks += 1
        report.errors.append(f"{page}:{block.line}: browser Run control in a local-only book; "
                             "keep output.html.playground.runnable = false")
    return report


def check_book(book, manifest_path, inventory):
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    report = Report()
    for page, block in rendered_blocks(book):
        report.blocks += 1
        report.errors.extend(
            f"{page}:{block.line}: {error}"
            for error in dependency_errors(block.code, manifest, inventory)
        )
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("book", type=Path, help="Freshly generated mdBook HTML directory")
    parser.add_argument("--manifest", type=Path,
                        default=Path(__file__).resolve().parents[1] / "Cargo.toml")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--local-only", action="store_true",
                      help="Reject browser Run controls without contacting the Playground")
    mode.add_argument("--inventory", type=Path,
                      help="Saved /meta/crates JSON for offline regression checks")
    args = parser.parse_args()
    try:
        report = (check_local_book(args.book) if args.local_only else
                  check_book(args.book, args.manifest, load_inventory(args.inventory)))
    except (OSError, URLError, ValueError, KeyError) as error:
        print(f"Playground dependency check failed: {error}", file=sys.stderr)
        return 1
    print(f"Found {report.blocks} rendered playground blocks"
          if args.local_only else
          f"Checked dependencies of {report.blocks} rendered playground blocks")
    if report.errors:
        print("\n".join(report.errors), file=sys.stderr)
        print("Run controls require a compatible execution environment. Publishing a crate "
              "does not automatically install it in the public Playground.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
