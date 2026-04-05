## Stage 1: Inspect Release State
**Goal**: Confirm the current crate version, release history, and changelog structure.
**Success Criteria**: Existing version and next release target are identified from repository state.
**Tests**: Review `Cargo.toml`, `CHANGELOG.md`, tags, and recent commits.
**Status**: Complete

## Stage 2: Update Release Metadata
**Goal**: Prepare the next release metadata in project files.
**Success Criteria**: `Cargo.toml` reflects the new version and `CHANGELOG.md` has a dated release entry with relevant notes.
**Tests**: Diff review of version and changelog changes.
**Status**: Complete

## Stage 3: Regenerate Lockfile
**Goal**: Refresh `Cargo.lock` to match the release metadata and dependency resolution.
**Success Criteria**: `Cargo.lock` is regenerated successfully with no manual inconsistencies.
**Tests**: Run `cargo generate-lockfile`.
**Status**: Complete

## Stage 4: Verify And Commit
**Goal**: Validate the release state and create a clean commit.
**Success Criteria**: Checks pass and the repository contains a single release-preparation commit.
**Tests**: Run `cargo check`, inspect `git diff --stat`, and commit.
**Status**: Complete
