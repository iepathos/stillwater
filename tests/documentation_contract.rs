use std::fs;
use std::path::{Path, PathBuf};

const CORE_CHAPTERS: &[&str] = &[
    "guide/03-effects.md",
    "guide/04-error-context.md",
    "guide/06-helper-combinators.md",
    "guide/09-reader-pattern.md",
    "guide/11-traverse-patterns.md",
    "guide/15-testing.md",
];

fn book_chapters(root: &Path) -> Vec<PathBuf> {
    fs::read_to_string(root.join("docs/SUMMARY.md"))
        .unwrap()
        .lines()
        .filter_map(|line| line.split_once("](")?.1.split_once(')').map(|pair| pair.0))
        .filter(|link| !link.contains("://"))
        .map(|link| root.join("docs").join(link))
        .collect()
}

fn rust_sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn documentation_line(line: &str) -> &str {
    let line = line.trim_start();
    line.strip_prefix("///")
        .or_else(|| line.strip_prefix("//!"))
        .unwrap_or(line)
        .trim_start()
}

fn fence(line: &str) -> Option<(char, usize, &str)> {
    let marker = line.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    (length >= 3).then(|| (marker, length, line[length..].trim()))
}

fn is_rust(info: &str) -> bool {
    let language = info.split([',', ' ', '\t']).next().unwrap_or("");
    matches!(
        language,
        "" | "rust" | "no_run" | "compile_fail" | "should_panic" | "ignore"
    ) || language.starts_with("edition20")
}

fn test_only(line: &str) -> bool {
    let compact: String = line
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    compact.contains("#[test]")
        || (compact.contains("#[cfg")
            && compact
                .split(|character: char| !character.is_alphanumeric() && character != '_')
                .any(|word| word == "test"))
        || compact.contains("#[tokio::test")
        || compact.contains("#[async_std::test")
}

#[derive(Default, Debug)]
struct Audit {
    rust_blocks: usize,
    text_blocks: usize,
    violations: Vec<usize>,
}

fn audit(source: &str) -> Audit {
    let mut result = Audit::default();
    let mut active: Option<(char, usize, bool)> = None;
    for (index, line) in source.lines().map(documentation_line).enumerate() {
        if let Some((marker, length, rust)) = active {
            if fence(line).is_some_and(|(close, count, info)| {
                close == marker && count >= length && info.is_empty()
            }) {
                active = None;
            } else if rust && test_only(line) {
                result.violations.push(index + 1);
            }
        } else if let Some((marker, length, info)) = fence(line) {
            let rust = is_rust(info);
            result.rust_blocks += usize::from(rust);
            result.text_blocks += usize::from(info == "text");
            if rust && info.contains("ignore") {
                result.violations.push(index + 1);
            }
            active = Some((marker, length, rust));
        }
    }
    result
}

#[test]
fn executable_documentation_is_checked_and_never_ignored() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("README.md")];
    files.extend(book_chapters(root));
    rust_sources(&root.join("src"), &mut files);
    let violations: Vec<_> = files
        .into_iter()
        .filter_map(|path| {
            let audit = audit(&fs::read_to_string(&path).unwrap());
            (!audit.violations.is_empty()).then_some((path, audit.violations))
        })
        .collect();
    assert!(violations.is_empty(),
        "Doctests must execute their bodies: remove ignore and test-only attributes: {violations:#?}");
}

#[test]
fn core_teaching_examples_cannot_be_downgraded_to_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    for chapter in CORE_CHAPTERS {
        let audit = audit(&fs::read_to_string(root.join(chapter)).unwrap());
        assert!(audit.rust_blocks > 0, "{chapter} needs executable examples");
        assert_eq!(
            audit.text_blocks, 0,
            "{chapter}: fix examples instead of hiding them as text"
        );
    }
}

#[test]
fn every_book_chapter_is_in_the_cargo_doctest_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("src/book_doctests.rs")).unwrap();
    let mut includes = vec!["#[doc = include_str!(\"../README.md\")]".to_string()];
    includes.extend(book_chapters(root).iter().map(|path| {
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        format!("#[doc = include_str!(\"../{relative}\")]")
    }));
    for include in includes {
        assert!(
            manifest.lines().any(|line| line.trim() == include),
            "Missing active doctest include: {include}"
        );
    }
}

#[test]
fn audit_rejects_test_only_bodies_and_ignored_fences() {
    for attribute in [
        "#[test]",
        "#[cfg(test)]",
        "#[tokio::test]",
        "# [ cfg ( test ) ]",
        "#[cfg(all(test, feature = \"async\"))]",
    ] {
        let source = format!("```rust\n{attribute}\nfn hidden() {{ missing(); }}\n```");
        assert_eq!(audit(&source).violations, vec![2], "{attribute}");
    }
    for fence in ["```rust,ignore", "~~~ignore", "```no_run,ignore"] {
        assert_eq!(audit(fence).violations, vec![1]);
    }
}

#[test]
fn audit_checks_rustdoc_and_distinguishes_prose_from_code() {
    let source = "//! ```rust\n//! #[test]\n//! ```";
    assert_eq!(audit(source).violations, vec![2]);
    let source = "```text\n#[test]\n```\n```rust\nassert_eq!(1, 1);\n```";
    let result = audit(source);
    assert!(result.violations.is_empty());
    assert_eq!((result.rust_blocks, result.text_blocks), (1, 1));
}

#[test]
fn book_uses_local_execution_and_links_the_instructions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = fs::read_to_string(root.join("book/book.toml")).unwrap();
    let playground = config.split_once("[output.html.playground]").unwrap().1;
    assert!(playground
        .lines()
        .take_while(|line| !line.trim_start().starts_with('['))
        .any(|line| line.trim() == "runnable = false"));
    let summary = fs::read_to_string(root.join("docs/SUMMARY.md")).unwrap();
    assert!(summary.contains("](running-examples.md)"));
}
