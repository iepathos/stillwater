//! Integration tests for bracket resource management with tokio file I/O.
//!
//! These tests verify that the bracket pattern correctly handles real-world
//! async I/O operations, ensuring resources are always cleaned up.

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use stillwater::effect::bracket::{acquiring, bracket, bracket2, bracket_full, BracketError};
use stillwater::{fail, from_fn, pure, Effect};

// ============================================================================
// File I/O Integration Tests
// ============================================================================

/// Helper to create a unique temp file path
fn temp_file_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("stillwater_bracket_test_{}.txt", name))
}

#[tokio::test]
async fn bracket_cleans_up_temp_file_on_success() {
    let path = temp_file_path("success");
    let path_clone = path.clone();
    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    // Create and use a temp file, then clean it up
    let result = bracket(
        // Acquire: create temp file
        from_fn(move |_: &()| {
            std::fs::write(&path_clone, "test content")?;
            Ok::<_, io::Error>(path_clone.clone())
        }),
        // Release: delete temp file
        move |p: PathBuf| {
            cleanup_ran_clone.store(true, Ordering::SeqCst);
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
        // Use: read the file
        |p: &PathBuf| {
            let content = std::fs::read_to_string(p).unwrap();
            pure::<_, io::Error, ()>(content)
        },
    )
    .run(&())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test content");
    assert!(
        cleanup_ran.load(Ordering::SeqCst),
        "cleanup should have run"
    );
    assert!(!path.exists(), "temp file should be deleted");
}

#[tokio::test]
async fn bracket_cleans_up_temp_file_on_use_failure() {
    let path = temp_file_path("use_failure");
    let path_clone = path.clone();
    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    // Create temp file, fail during use, verify cleanup still runs
    let result = bracket(
        // Acquire: create temp file
        from_fn(move |_: &()| {
            std::fs::write(&path_clone, "test content")?;
            Ok::<_, io::Error>(path_clone.clone())
        }),
        // Release: delete temp file (should run even on use failure)
        move |p: PathBuf| {
            cleanup_ran_clone.store(true, Ordering::SeqCst);
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
        // Use: fail
        |_: &PathBuf| fail::<String, io::Error, ()>(io::Error::other("use failed")),
    )
    .run(&())
    .await;

    assert!(result.is_err());
    assert!(
        cleanup_ran.load(Ordering::SeqCst),
        "cleanup must run on use failure"
    );
    assert!(
        !path.exists(),
        "temp file should be deleted despite failure"
    );
}

#[tokio::test]
async fn bracket_does_not_cleanup_on_acquire_failure() {
    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    let result = bracket(
        // Acquire: fail immediately
        from_fn(|_: &()| Err::<PathBuf, io::Error>(io::Error::other("acquire failed"))),
        // Release: should NOT run
        move |_: PathBuf| {
            cleanup_ran_clone.store(true, Ordering::SeqCst);
            async move { Ok(()) }
        },
        // Use: should NOT run
        |_: &PathBuf| pure::<_, io::Error, ()>("unused".to_string()),
    )
    .run(&())
    .await;

    assert!(result.is_err());
    assert!(
        !cleanup_ran.load(Ordering::SeqCst),
        "cleanup must NOT run when acquire fails"
    );
}

// ============================================================================
// Multiple Resources with LIFO Cleanup
// ============================================================================

#[tokio::test]
async fn bracket2_cleans_up_both_files_in_lifo_order() {
    let path1 = temp_file_path("lifo1");
    let path2 = temp_file_path("lifo2");
    let path1_clone = path1.clone();
    let path2_clone = path2.clone();

    let cleanup_order = Arc::new(std::sync::Mutex::new(Vec::<&str>::new()));
    let order1 = cleanup_order.clone();
    let order2 = cleanup_order.clone();

    let result = bracket2(
        // Acquire first file
        from_fn(move |_: &()| {
            std::fs::write(&path1_clone, "file 1")?;
            Ok::<_, io::Error>(path1_clone.clone())
        }),
        // Acquire second file
        from_fn(move |_: &()| {
            std::fs::write(&path2_clone, "file 2")?;
            Ok::<_, io::Error>(path2_clone.clone())
        }),
        // Release first (runs second in LIFO)
        move |p: PathBuf| {
            order1.lock().unwrap().push("file1");
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
        // Release second (runs first in LIFO)
        move |p: PathBuf| {
            order2.lock().unwrap().push("file2");
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
        // Use both files
        |p1: &PathBuf, p2: &PathBuf| {
            let c1 = std::fs::read_to_string(p1).unwrap();
            let c2 = std::fs::read_to_string(p2).unwrap();
            pure::<_, io::Error, ()>(format!("{} + {}", c1, c2))
        },
    )
    .run(&())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "file 1 + file 2");

    // Verify LIFO order: file2 should be released before file1
    let order = cleanup_order.lock().unwrap();
    assert_eq!(*order, vec!["file2", "file1"], "cleanup should be LIFO");

    // Both files should be deleted
    assert!(!path1.exists(), "file1 should be deleted");
    assert!(!path2.exists(), "file2 should be deleted");
}

#[tokio::test]
async fn bracket2_cleans_up_first_resource_when_second_acquire_fails() {
    let path1 = temp_file_path("partial_acquire");
    let path1_clone = path1.clone();

    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    let result = bracket2(
        // Acquire first: succeeds
        from_fn(move |_: &()| {
            std::fs::write(&path1_clone, "file 1")?;
            Ok::<_, io::Error>(path1_clone.clone())
        }),
        // Acquire second: fails
        from_fn(|_: &()| Err::<PathBuf, io::Error>(io::Error::other("second acquire failed"))),
        // Release first: should run to clean up
        move |p: PathBuf| {
            cleanup_ran_clone.store(true, Ordering::SeqCst);
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
        // Release second: should NOT run
        |_: PathBuf| async { Ok(()) },
        // Use: should NOT run
        |_: &PathBuf, _: &PathBuf| pure::<_, io::Error, ()>("unused".to_string()),
    )
    .run(&())
    .await;

    assert!(result.is_err());
    assert!(
        cleanup_ran.load(Ordering::SeqCst),
        "first resource must be released when second acquire fails"
    );
    assert!(!path1.exists(), "first file should be deleted");
}

// ============================================================================
// Acquiring Builder with File I/O
// ============================================================================

#[tokio::test]
async fn acquiring_builder_cleans_up_all_files() {
    let path1 = temp_file_path("builder1");
    let path2 = temp_file_path("builder2");
    let path3 = temp_file_path("builder3");
    let path1_clone = path1.clone();
    let path2_clone = path2.clone();
    let path3_clone = path3.clone();

    let cleanup_order = Arc::new(std::sync::Mutex::new(Vec::<&str>::new()));
    let order1 = cleanup_order.clone();
    let order2 = cleanup_order.clone();
    let order3 = cleanup_order.clone();

    let result = acquiring(
        from_fn(move |_: &()| {
            std::fs::write(&path1_clone, "content 1")?;
            Ok::<_, io::Error>(path1_clone.clone())
        }),
        move |p: PathBuf| {
            order1.lock().unwrap().push("file1");
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
    )
    .and(
        from_fn(move |_: &()| {
            std::fs::write(&path2_clone, "content 2")?;
            Ok::<_, io::Error>(path2_clone.clone())
        }),
        move |p: PathBuf| {
            order2.lock().unwrap().push("file2");
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
    )
    .and(
        from_fn(move |_: &()| {
            std::fs::write(&path3_clone, "content 3")?;
            Ok::<_, io::Error>(path3_clone.clone())
        }),
        move |p: PathBuf| {
            order3.lock().unwrap().push("file3");
            async move {
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
                Ok(())
            }
        },
    )
    .with_flat3(|p1: &PathBuf, p2: &PathBuf, p3: &PathBuf| {
        let c1 = std::fs::read_to_string(p1).unwrap();
        let c2 = std::fs::read_to_string(p2).unwrap();
        let c3 = std::fs::read_to_string(p3).unwrap();
        pure::<_, io::Error, ()>(format!("{}, {}, {}", c1, c2, c3))
    })
    .run(&())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "content 1, content 2, content 3");

    // Verify LIFO order
    let order = cleanup_order.lock().unwrap();
    assert_eq!(*order, vec!["file3", "file2", "file1"]);

    // All files should be deleted
    assert!(!path1.exists());
    assert!(!path2.exists());
    assert!(!path3.exists());
}

// ============================================================================
// BracketFull Error Handling Tests
// ============================================================================

#[tokio::test]
async fn bracket_full_returns_cleanup_error_when_use_succeeds() {
    let path = temp_file_path("cleanup_error");
    let path_clone = path.clone();

    let result = bracket_full(
        from_fn(move |_: &()| {
            std::fs::write(&path_clone, "content")?;
            Ok::<_, io::Error>(path_clone.clone())
        }),
        // Release: fails intentionally
        |_: PathBuf| async { Err::<(), io::Error>(io::Error::other("cleanup failed")) },
        // Use: succeeds
        |p: &PathBuf| {
            let content = std::fs::read_to_string(p).unwrap();
            pure::<_, io::Error, ()>(content)
        },
    )
    .run(&())
    .await;

    match result {
        Err(BracketError::CleanupError(e)) => {
            assert!(e.to_string().contains("cleanup failed"));
        }
        other => panic!("expected CleanupError, got {:?}", other),
    }

    // Clean up manually since the bracket cleanup "failed"
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn bracket_full_returns_both_errors() {
    let path = temp_file_path("both_errors");
    let path_clone = path.clone();

    let result = bracket_full(
        from_fn(move |_: &()| {
            std::fs::write(&path_clone, "content")?;
            Ok::<_, io::Error>(path_clone.clone())
        }),
        // Release: fails
        |_: PathBuf| async { Err::<(), io::Error>(io::Error::other("cleanup failed")) },
        // Use: also fails
        |_: &PathBuf| fail::<String, io::Error, ()>(io::Error::other("use failed")),
    )
    .run(&())
    .await;

    match result {
        Err(BracketError::Both {
            use_error,
            cleanup_error,
        }) => {
            assert!(use_error.to_string().contains("use failed"));
            assert!(cleanup_error.to_string().contains("cleanup failed"));
        }
        other => panic!("expected Both, got {:?}", other),
    }

    // Clean up manually
    let _ = std::fs::remove_file(&path);
}

// ============================================================================
// Async-specific Tests with tokio
// ============================================================================

#[tokio::test]
async fn bracket_works_with_tokio_async_file_operations() {
    let path = temp_file_path("tokio_async");
    let path_clone = path.clone();
    let path_for_release = path.clone();

    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_ran_clone = cleanup_ran.clone();

    let result = bracket(
        // Acquire: create file using tokio async
        from_fn(move |_: &()| {
            // We use sync write in acquire for simplicity, but release is async
            std::fs::write(&path_clone, "async test content")?;
            Ok::<_, io::Error>(path_clone.clone())
        }),
        // Release: async cleanup
        move |p: PathBuf| {
            cleanup_ran_clone.store(true, Ordering::SeqCst);
            async move {
                tokio::fs::remove_file(&p).await?;
                Ok(())
            }
        },
        // Use: async read
        |p: &PathBuf| {
            let p = p.clone();
            from_fn(move |_: &()| {
                let content = std::fs::read_to_string(&p)?;
                Ok::<_, io::Error>(content)
            })
        },
    )
    .run(&())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "async test content");
    assert!(cleanup_ran.load(Ordering::SeqCst));
    assert!(
        !path_for_release.exists(),
        "file should be deleted by async cleanup"
    );
}

#[tokio::test]
async fn bracket_handles_concurrent_resource_access() {
    // Simulate a counter that tracks active resources
    let active_count = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    let tasks: Vec<_> = (0..5)
        .map(|i| {
            let active = active_count.clone();
            let active_for_release = active_count.clone();
            let max = max_concurrent.clone();

            tokio::spawn(async move {
                bracket(
                    from_fn(move |_: &()| {
                        let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max.fetch_max(count, Ordering::SeqCst);
                        Ok::<_, String>(i)
                    }),
                    move |_: i32| {
                        active_for_release.fetch_sub(1, Ordering::SeqCst);
                        async { Ok(()) }
                    },
                    |id: &i32| {
                        // Simulate some async work
                        let id = *id;
                        from_fn(move |_: &()| Ok::<_, String>(id * 2))
                    },
                )
                .run(&())
                .await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All tasks should succeed
    assert!(results.iter().all(|r| r.is_ok()));

    // After all tasks complete, active count should be 0
    assert_eq!(active_count.load(Ordering::SeqCst), 0);
}
