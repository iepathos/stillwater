use std::fs;
use std::path::{Path, PathBuf};

fn markdown_link(line: &str) -> Option<&str> {
    let start = line.find("](")? + 2;
    let end = line[start..].find(')')? + start;
    Some(&line[start..end])
}

fn book_chapters(root: &Path) -> Vec<PathBuf> {
    let summary = fs::read_to_string(root.join("docs/SUMMARY.md")).unwrap();
    summary
        .lines()
        .filter_map(markdown_link)
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

#[test]
fn executable_documentation_is_never_ignored() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("README.md")];
    files.extend(book_chapters(root));
    rust_sources(&root.join("src"), &mut files);

    let ignored = files
        .into_iter()
        .filter(|path| {
            fs::read_to_string(path).unwrap().lines().any(|line| {
                let fence = line.trim_start_matches([' ', '/', '!', '*']);
                fence.starts_with("```ignore")
                    || (fence.starts_with("```rust") && fence.contains("ignore"))
            })
        })
        .collect::<Vec<_>>();

    assert!(
        ignored.is_empty(),
        "replace ignored Rust with compiled examples or explicit text: {ignored:#?}"
    );
}

#[test]
fn every_book_chapter_is_in_the_cargo_doctest_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("src/book_doctests.rs")).unwrap();
    let missing = book_chapters(root)
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root.join("docs")).unwrap();
            let relative = relative.to_string_lossy();
            (!manifest.contains(relative.as_ref())).then_some(relative.into_owned())
        })
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "add new mdBook chapters to src/book_doctests.rs: {missing:#?}"
    );
}
