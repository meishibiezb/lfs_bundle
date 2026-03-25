# LFS Bundle GUI Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current single-file Rust prototype into a shared core + retained CLI + `egui` desktop app that can package commit-range bundle/LFS archives and import them atomically with rollback and cleanup guarantees.

**Architecture:** Extract reusable package/import logic into `src/lib.rs` and focused `src/core/*` modules, keep `src/main.rs` as a thin mode selector, and route all repository mutations through a transaction executor that records original branch state, applies bundle first, imports LFS second, and rolls back on failure. The GUI becomes a thin `egui` shell over the same core requests/events used by the CLI.

**Tech Stack:** Rust 2024, `anyhow`, `clap`, `eframe`/`egui`, `serde`, `serde_json`, `tempfile`, `tar`, `flate2`, `zip`, `sha2`, `regex`

---

## Proposed File Structure

### Modify

- `Cargo.toml` — add GUI/CLI/serialization/temp/archive dependencies
- `src/main.rs` — replace hard-coded prototype flow with CLI/GUI entry dispatch

### Create

- `src/lib.rs` — shared library surface
- `src/core/mod.rs` — module exports
- `src/core/models.rs` — request/response structs and shared domain types
- `src/core/events.rs` — progress/log event definitions used by CLI and GUI
- `src/core/process.rs` — subprocess helpers for `git` / `git lfs`
- `src/core/repo.rs` — repository inspection, commit history, branch listing
- `src/core/lfs.rs` — LFS parsing, path mapping, archive collection helpers
- `src/core/manifest.rs` — manifest schema and checksum helpers
- `src/core/archive.rs` — final archive assembly/extraction helpers
- `src/core/temp.rs` — temporary session/worktree lifecycle helpers
- `src/core/pack.rs` — package workflow implementation
- `src/core/import.rs` — import workflow entrypoints
- `src/core/transaction.rs` — rollback-aware transaction executor
- `src/cli/mod.rs` — clap setup and CLI dispatch
- `src/gui/mod.rs` — GUI bootstrap entrypoint
- `src/gui/app.rs` — `egui` app state
- `src/gui/theme.rs` — colors, spacing, badges, reusable UI tokens
- `src/gui/views/mod.rs` — view exports
- `src/gui/views/pack.rs` — packaging workbench
- `src/gui/views/import.rs` — import workbench
- `src/gui/views/history.rs` — recent operations view
- `src/gui/views/settings.rs` — minimal settings view
- `tests/parse_lfs.rs` — LFS parsing tests
- `tests/manifest_roundtrip.rs` — manifest/archive tests
- `tests/pack_flow.rs` — package integration tests with temp repos
- `tests/import_transaction.rs` — import success/failure/rollback integration tests
- `tests/cli_smoke.rs` — CLI command smoke tests
- `tests/gui_state.rs` — GUI state/request mapping tests

---

## Chunk 1: Foundation and Shared Core Skeleton

### Task 1: Add dependencies and expose a library crate

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/core/mod.rs`

- [ ] **Step 1: Add the failing compile target for the future library split**

Add this to `src/lib.rs`:

```rust
pub mod core;
pub mod cli;
pub mod gui;
```

Expected current result: compile fails because modules do not exist yet.

- [ ] **Step 2: Run compile to verify the split is currently incomplete**

Run: `cargo test --no-run`
Expected: FAIL with missing module errors for `core`, `cli`, or `gui`

- [ ] **Step 3: Add the required dependencies to `Cargo.toml`**

Add:

```toml
clap = { version = "4", features = ["derive"] }
eframe = "0.33"
egui = "0.33"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
tempfile = "3"
zip = { version = "2", default-features = false, features = ["deflate"] }
```

Keep existing `anyhow`, `flate2`, `regex`, and `tar`.

- [ ] **Step 4: Create minimal module stubs**

Create:

```rust
// src/core/mod.rs
pub mod archive;
pub mod events;
pub mod import;
pub mod lfs;
pub mod manifest;
pub mod models;
pub mod pack;
pub mod process;
pub mod repo;
pub mod temp;
pub mod transaction;
```

And minimal placeholder `mod.rs` / file stubs for `src/cli/mod.rs`, `src/gui/mod.rs`, and each declared core module that compile with `todo!()` or empty structs/functions.

- [ ] **Step 5: Run compile to verify the new skeleton builds**

Run: `cargo test --no-run`
Expected: PASS for compilation

- [ ] **Step 6: Commit**

Run:

```bash
git add Cargo.toml src
git commit -m "refactor: add shared library and module skeleton"
```

### Task 2: Move LFS parsing into the core with tests

**Files:**
- Create: `tests/parse_lfs.rs`
- Create: `src/core/lfs.rs`

- [ ] **Step 1: Write the failing LFS parsing test**

Create `tests/parse_lfs.rs` with:

```rust
use lfs_bundle::core::lfs::parse_lfs_output;
use std::path::PathBuf;

#[test]
fn parse_lfs_output_converts_hashes_to_object_paths() {
    let output = "52b6c1c0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa * file.bin";
    let paths = parse_lfs_output(output);
    assert_eq!(
        paths,
        vec![PathBuf::from(".git/lfs/objects/52/b6/52b6c1c0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    );
}
```

- [ ] **Step 2: Run the test to confirm it fails**

Run: `cargo test --test parse_lfs`
Expected: FAIL because `parse_lfs_output` is not exported yet

- [ ] **Step 3: Implement the parser in `src/core/lfs.rs`**

Move and adapt the current regex-based parser into:

```rust
pub fn parse_lfs_output(output: &str) -> Vec<PathBuf> { /* ... */ }
```

Requirements:
- keep the current hash parsing behavior
- return repository-relative `.git/lfs/objects/...` paths
- ignore lines that do not match the expected hash prefix

- [ ] **Step 4: Re-run the test**

Run: `cargo test --test parse_lfs`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/lfs.rs tests/parse_lfs.rs
git commit -m "test: cover lfs output parsing"
```

### Task 3: Add shared models and event types

**Files:**
- Create: `src/core/models.rs`
- Create: `src/core/events.rs`

- [ ] **Step 1: Write the failing request model test**

Add a small serialization test in `tests/manifest_roundtrip.rs`:

```rust
use lfs_bundle::core::models::PackageRequest;

#[test]
fn package_request_keeps_selected_commit_range() {
    let req = PackageRequest {
        repo_path: "D:/repo".into(),
        start_commit: "abc123".into(),
        end_commit: "def456".into(),
        output_archive: "D:/out/pkg.zip".into(),
        safe_mode: true,
    };
    assert_eq!(req.start_commit, "abc123");
    assert_eq!(req.end_commit, "def456");
}
```

- [ ] **Step 2: Run the targeted test**

Run: `cargo test package_request_keeps_selected_commit_range -- --exact`
Expected: FAIL because `PackageRequest` does not exist yet

- [ ] **Step 3: Implement the shared domain types**

In `src/core/models.rs`, add concrete structs/enums for:

```rust
pub struct PackageRequest { /* repo_path, start_commit, end_commit, output_archive, safe_mode */ }
pub struct ImportRequest { /* repo_path, branch, archive_path, safe_mode */ }
pub struct CommitInfo { /* id, short_id, summary, author, timestamp */ }
pub struct BranchInfo { /* name, head */ }
pub struct PackageSummary { /* commit_count, lfs_object_count, bundle_name, lfs_name */ }
pub enum RunMode { Cli, Gui }
```

In `src/core/events.rs`, add:

```rust
pub enum ProgressState { Pending, Running, Succeeded, Failed }
pub struct ProgressEvent { pub step: String, pub state: ProgressState, pub detail: Option<String> }
pub struct LogEvent { pub message: String, pub is_error: bool }
```

Derive `Clone`, `Debug`, and `Serialize`/`Deserialize` where useful.

- [ ] **Step 4: Run the test again**

Run: `cargo test package_request_keeps_selected_commit_range -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/models.rs src/core/events.rs tests/manifest_roundtrip.rs
git commit -m "feat: add shared workflow models and events"
```

## Chunk 2: Repository, Manifest, and Packaging Workflows

### Task 4: Add repository inspection helpers

**Files:**
- Create: `src/core/process.rs`
- Create: `src/core/repo.rs`

- [ ] **Step 1: Write the failing repository helper tests**

Extend `tests/pack_flow.rs` with a temp-dir smoke test skeleton:

```rust
use lfs_bundle::core::repo::{is_git_repo_path, split_short_id};

#[test]
fn split_short_id_truncates_long_commit_ids() {
    assert_eq!(split_short_id("1234567890abcdef"), "12345678");
}
```

- [ ] **Step 2: Run the targeted test**

Run: `cargo test split_short_id_truncates_long_commit_ids -- --exact`
Expected: FAIL because helpers do not exist yet

- [ ] **Step 3: Implement subprocess and repo helpers**

In `src/core/process.rs`, add a helper similar to:

```rust
pub fn run_command(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<Output> { /* ... */ }
```

Requirements:
- capture stdout/stderr
- include cwd and full command in errors
- do not print directly

In `src/core/repo.rs`, implement:

```rust
pub fn is_git_repo_path(path: &Path) -> bool;
pub fn split_short_id(commit: &str) -> String;
pub fn load_commits(repo: &Path, limit: usize) -> Result<Vec<CommitInfo>>;
pub fn load_branches(repo: &Path) -> Result<Vec<BranchInfo>>;
pub fn current_head(repo: &Path, branch: &str) -> Result<String>;
```

Use `git log` and `git for-each-ref`.

- [ ] **Step 4: Re-run the targeted test**

Run: `cargo test split_short_id_truncates_long_commit_ids -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/process.rs src/core/repo.rs tests/pack_flow.rs
git commit -m "feat: add repository inspection helpers"
```

### Task 5: Implement manifest schema and archive helpers

**Files:**
- Create: `src/core/manifest.rs`
- Create: `src/core/archive.rs`
- Modify: `tests/manifest_roundtrip.rs`

- [ ] **Step 1: Write the failing manifest roundtrip test**

In `tests/manifest_roundtrip.rs`, add:

```rust
use lfs_bundle::core::manifest::{BundleManifest, FileEntry};

#[test]
fn manifest_roundtrips_through_json() {
    let manifest = BundleManifest {
        tool_version: "0.1.0".into(),
        source_repo: "demo".into(),
        start_commit: "abc".into(),
        end_commit: "def".into(),
        target_commit: "def".into(),
        bundle: FileEntry { name: "bundle.bundle".into(), size_bytes: 10, sha256: "1".into() },
        lfs: FileEntry { name: "lfs.tar.gz".into(), size_bytes: 20, sha256: "2".into() },
        lfs_object_count: 1,
        created_at: "2026-03-25T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let reparsed: BundleManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(reparsed.end_commit, "def");
}
```

- [ ] **Step 2: Run the test and confirm failure**

Run: `cargo test manifest_roundtrips_through_json -- --exact`
Expected: FAIL because manifest types do not exist yet

- [ ] **Step 3: Implement manifest types and archive helpers**

In `src/core/manifest.rs`, add:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry { pub name: String, pub size_bytes: u64, pub sha256: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    pub tool_version: String,
    pub source_repo: String,
    pub start_commit: String,
    pub end_commit: String,
    pub target_commit: String,
    pub bundle: FileEntry,
    pub lfs: FileEntry,
    pub lfs_object_count: usize,
    pub created_at: String,
}
```

Also implement:

```rust
pub fn sha256_file(path: &Path) -> Result<String>;
pub fn write_manifest(path: &Path, manifest: &BundleManifest) -> Result<()>;
pub fn read_manifest(path: &Path) -> Result<BundleManifest>;
```

In `src/core/archive.rs`, implement:

```rust
pub fn build_final_archive(output_zip: &Path, bundle: &Path, lfs: &Path, manifest: &Path) -> Result<()>;
pub fn extract_archive(input_zip: &Path, dest: &Path) -> Result<()>;
```

Use the `zip` crate and preserve exact filenames `bundle.bundle`, `lfs.tar.gz`, and `manifest.json`.

- [ ] **Step 4: Re-run the manifest test**

Run: `cargo test manifest_roundtrips_through_json -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/manifest.rs src/core/archive.rs tests/manifest_roundtrip.rs
git commit -m "feat: add manifest and final archive helpers"
```

### Task 6: Implement the packaging workflow

**Files:**
- Create: `src/core/temp.rs`
- Create: `src/core/pack.rs`
- Modify: `tests/pack_flow.rs`

- [ ] **Step 1: Write the failing package integration test**

In `tests/pack_flow.rs`, create a temp-repo integration test that:

```rust
#[test]
fn package_workflow_creates_bundle_lfs_and_manifest_archive() {
    // init temp git repo
    // create at least two commits
    // call package_repository(...)
    // assert final zip exists
    // extract zip
    // assert bundle.bundle, lfs.tar.gz, manifest.json exist
}
```

Use `std::process::Command` inside the test to initialize the repo. Skip LFS assertions if `git lfs version` is unavailable by returning early.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test pack_flow`
Expected: FAIL because `package_repository` does not exist yet

- [ ] **Step 3: Implement temporary session and packaging workflow**

In `src/core/temp.rs`, add:

```rust
pub struct SessionDir { pub root: TempDir }
pub fn new_session_dir(prefix: &str) -> Result<SessionDir>;
```

In `src/core/pack.rs`, implement:

```rust
pub fn package_repository(req: &PackageRequest) -> Result<PackageSummary>;
```

The function must:
- validate repo path
- create a session temp dir
- run `git bundle create`
- run `git lfs ls-files --long <start> <end>`
- collect matching LFS object files
- write `bundle.bundle`
- write `lfs.tar.gz`
- write `manifest.json`
- assemble final zip at `req.output_archive`
- rely on `TempDir` cleanup instead of manual deletion

- [ ] **Step 4: Re-run the package integration test**

Run: `cargo test --test pack_flow`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/temp.rs src/core/pack.rs tests/pack_flow.rs
git commit -m "feat: implement packaging workflow"
```

## Chunk 3: Atomic Import Transaction

### Task 7: Build the rollback-aware transaction executor

**Files:**
- Create: `src/core/transaction.rs`
- Create: `src/core/import.rs`
- Modify: `tests/import_transaction.rs`

- [ ] **Step 1: Write the failing rollback test**

Add to `tests/import_transaction.rs`:

```rust
#[test]
fn lfs_import_failure_restores_original_head() {
    // arrange source repo + package archive
    // arrange target repo with existing branch
    // record original head
    // call import_archive with a test hook that forces LFS stage failure
    // assert branch head == original head after error
}
```

Expose a test-only failure injection like:

```rust
pub enum FailureInjection {
    None,
    FailAfterBundleApplied,
}
```

- [ ] **Step 2: Run the test to confirm failure**

Run: `cargo test lfs_import_failure_restores_original_head -- --exact`
Expected: FAIL because transaction/import APIs do not exist yet

- [ ] **Step 3: Implement transaction state and rollback**

In `src/core/transaction.rs`, implement:

```rust
pub struct ImportTransaction {
    pub original_head: String,
    pub branch: String,
    pub repo_path: PathBuf,
}

impl ImportTransaction {
    pub fn begin(repo_path: PathBuf, branch: String) -> Result<Self>;
    pub fn rollback(&self) -> Result<()>;
}
```

In `src/core/import.rs`, implement:

```rust
pub fn import_archive(req: &ImportRequest) -> Result<()>;
```

Required order:
- validate archive/repo/branch
- extract archive to temp dir
- read `manifest.json`
- begin transaction and record original HEAD
- apply bundle to selected branch
- only then import LFS objects
- on any failure after bundle application, call rollback

Use helper functions like:

```rust
fn apply_bundle_to_branch(...);
fn import_lfs_objects(...);
```

- [ ] **Step 4: Re-run the rollback test**

Run: `cargo test lfs_import_failure_restores_original_head -- --exact`
Expected: PASS

- [ ] **Step 5: Add the happy-path import integration test**

Add:

```rust
#[test]
fn import_archive_updates_existing_branch_when_bundle_and_lfs_succeed() {
    // arrange package + target repo
    // call import_archive
    // assert target branch head matches package target commit
}
```

- [ ] **Step 6: Run the full import test file**

Run: `cargo test --test import_transaction`
Expected: PASS

- [ ] **Step 7: Commit**

Run:

```bash
git add src/core/transaction.rs src/core/import.rs tests/import_transaction.rs
git commit -m "feat: implement atomic import transaction"
```

### Task 8: Add manifest/package inspection helpers

**Files:**
- Modify: `src/core/archive.rs`
- Modify: `src/core/manifest.rs`
- Modify: `src/core/models.rs`
- Modify: `tests/manifest_roundtrip.rs`

- [ ] **Step 1: Write the failing archive inspection test**

Add:

```rust
#[test]
fn extracted_archive_manifest_matches_expected_commits() {
    // create a minimal synthetic archive with manifest.json
    // call inspect_archive(...)
    // assert summary start/end commit values are preserved
}
```

- [ ] **Step 2: Run the targeted inspection test**

Run: `cargo test extracted_archive_manifest_matches_expected_commits -- --exact`
Expected: FAIL because inspection API does not exist yet

- [ ] **Step 3: Implement archive inspection**

Add to `src/core/archive.rs`:

```rust
pub fn inspect_archive(input_zip: &Path) -> Result<BundleManifest>;
```

Add a lightweight `ArchiveInspection` or reuse `BundleManifest` in `src/core/models.rs` for CLI/GUI consumption.

- [ ] **Step 4: Re-run the targeted inspection test**

Run: `cargo test extracted_archive_manifest_matches_expected_commits -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/core/archive.rs src/core/manifest.rs src/core/models.rs tests/manifest_roundtrip.rs
git commit -m "feat: add archive inspection helpers"
```

## Chunk 4: Retained CLI on Top of the Core

### Task 9: Add the CLI entrypoint and pack/inspect subcommands

**Files:**
- Create: `src/cli/mod.rs`
- Modify: `src/main.rs`
- Modify: `tests/cli_smoke.rs`

- [ ] **Step 1: Write the failing CLI smoke tests**

In `tests/cli_smoke.rs`, add:

```rust
#[test]
fn help_command_mentions_pack_import_and_gui() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lfs_bundle"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pack"));
    assert!(stdout.contains("import"));
    assert!(stdout.contains("gui"));
}
```

- [ ] **Step 2: Run the smoke test**

Run: `cargo test --test cli_smoke`
Expected: FAIL because clap CLI does not exist yet

- [ ] **Step 3: Implement clap commands**

In `src/cli/mod.rs`, add:

```rust
#[derive(clap::Parser)]
pub struct Cli { pub command: Option<Commands> }

#[derive(clap::Subcommand)]
pub enum Commands {
    Gui,
    Pack { repo: PathBuf, from: String, to: String, output: PathBuf, #[arg(long)] direct: bool },
    Import { repo: PathBuf, branch: String, archive: PathBuf, #[arg(long)] direct: bool },
    Inspect { archive: PathBuf },
}
```

In `src/main.rs`, dispatch:
- `Some(Commands::Gui)` => launch GUI
- `Some(Commands::Pack { .. })` => call `package_repository`
- `Some(Commands::Import { .. })` => call `import_archive`
- `Some(Commands::Inspect { .. })` => print manifest summary
- `None` => launch GUI by default

- [ ] **Step 4: Re-run the CLI smoke tests**

Run: `cargo test --test cli_smoke`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/main.rs src/cli/mod.rs tests/cli_smoke.rs
git commit -m "feat: add shared cli entrypoint"
```

### Task 10: Add the CLI import subcommand output and exit behavior

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `tests/cli_smoke.rs`

- [ ] **Step 1: Write the failing inspect/import output tests**

Add smoke tests that assert:
- `inspect <archive>` prints `start_commit` and `end_commit`
- failed import exits non-zero

Example:

```rust
#[test]
fn inspect_prints_manifest_commit_range() {
    // create synthetic archive fixture
    // run `lfs_bundle inspect <archive>`
    // assert stdout contains both commit ids
}
```

- [ ] **Step 2: Run the CLI smoke test file**

Run: `cargo test --test cli_smoke`
Expected: FAIL because output handling is incomplete

- [ ] **Step 3: Implement readable CLI output**

Requirements:
- print concise success summaries to stdout
- print failures to stderr
- return `ExitCode::FAILURE` on workflow errors
- include rollback status in import failure output when available

- [ ] **Step 4: Re-run the CLI smoke tests**

Run: `cargo test --test cli_smoke`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/cli/mod.rs tests/cli_smoke.rs
git commit -m "feat: improve cli workflow output"
```

## Chunk 5: GUI Shell and Packaging Workbench

### Task 11: Create the GUI app shell and navigation

**Files:**
- Create: `src/gui/mod.rs`
- Create: `src/gui/app.rs`
- Create: `src/gui/theme.rs`
- Create: `src/gui/views/mod.rs`
- Create: `src/gui/views/history.rs`
- Create: `src/gui/views/settings.rs`
- Modify: `tests/gui_state.rs`

- [ ] **Step 1: Write the failing GUI state tests**

Add to `tests/gui_state.rs`:

```rust
use lfs_bundle::gui::app::{AppTab, BundleStudioApp};

#[test]
fn default_gui_tab_is_packaging() {
    let app = BundleStudioApp::default();
    assert_eq!(app.active_tab(), AppTab::Packaging);
}
```

- [ ] **Step 2: Run the targeted GUI state test**

Run: `cargo test default_gui_tab_is_packaging -- --exact`
Expected: FAIL because GUI app state does not exist yet

- [ ] **Step 3: Implement GUI shell**

In `src/gui/app.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab { Packaging, Import, History, Settings }

pub struct BundleStudioApp { /* active_tab, logs, recent_ops, theme, pack_view, import_view */ }

impl Default for BundleStudioApp { /* active tab = Packaging */ }
```

In `src/gui/mod.rs`, implement:

```rust
pub fn launch() -> Result<()>;
```

Use `eframe::run_native`.

- [ ] **Step 4: Re-run the GUI state test**

Run: `cargo test default_gui_tab_is_packaging -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/gui tests/gui_state.rs
git commit -m "feat: add gui shell and navigation"
```

### Task 12: Implement the packaging workbench UI

**Files:**
- Create: `src/gui/views/pack.rs`
- Modify: `src/gui/app.rs`
- Modify: `tests/gui_state.rs`

- [ ] **Step 1: Write the failing pack request mapping test**

Add:

```rust
#[test]
fn pack_form_builds_safe_mode_request_from_selected_commits() {
    // fill form state
    // convert to PackageRequest
    // assert repo/start/end/output/safe_mode values
}
```

- [ ] **Step 2: Run the targeted test**

Run: `cargo test pack_form_builds_safe_mode_request_from_selected_commits -- --exact`
Expected: FAIL because pack form state is not implemented

- [ ] **Step 3: Implement packaging workbench**

Requirements:
- repo path input and picker hook
- commit search box
- start/end commit selectors
- output archive path input
- safe/direct mode badge
- preview cards for commit count, LFS count, output path
- action button that spawns `package_repository`
- progress/log display fed from core events

Keep file dialog integration optional; if no picker is wired yet, support typed paths first.

- [ ] **Step 4: Re-run the targeted GUI state test**

Run: `cargo test pack_form_builds_safe_mode_request_from_selected_commits -- --exact`
Expected: PASS

- [ ] **Step 5: Launch the GUI manually for visual smoke test**

Run: `cargo run -- gui`
Expected: window opens with left navigation and Packaging tab selected

- [ ] **Step 6: Commit**

Run:

```bash
git add src/gui tests/gui_state.rs
git commit -m "feat: add packaging workbench ui"
```

## Chunk 6: Import Workbench, Polish, and Final Verification

### Task 13: Implement the import workbench UI

**Files:**
- Create: `src/gui/views/import.rs`
- Modify: `src/gui/app.rs`
- Modify: `tests/gui_state.rs`

- [ ] **Step 1: Write the failing import request mapping test**

Add:

```rust
#[test]
fn import_form_builds_request_for_existing_branch() {
    // fill import form state
    // convert to ImportRequest
    // assert archive path, repo path, branch, safe_mode
}
```

- [ ] **Step 2: Run the targeted test**

Run: `cargo test import_form_builds_request_for_existing_branch -- --exact`
Expected: FAIL because import form state does not exist yet

- [ ] **Step 3: Implement import workbench**

Requirements:
- archive picker / path input
- manifest summary card
- target repo path input
- existing branch dropdown
- validation checklist with pass/warn/fail badges
- transaction progress rail:
  - record HEAD
  - apply bundle
  - verify result
  - import LFS
  - rollback if needed
  - cleanup
- clear success/failure panel including rollback and cleanup status

- [ ] **Step 4: Re-run the targeted GUI state test**

Run: `cargo test import_form_builds_request_for_existing_branch -- --exact`
Expected: PASS

- [ ] **Step 5: Launch the GUI manually for visual smoke test**

Run: `cargo run -- gui`
Expected: Import tab renders and can switch back to Packaging

- [ ] **Step 6: Commit**

Run:

```bash
git add src/gui tests/gui_state.rs
git commit -m "feat: add import workbench ui"
```

### Task 14: Add history/settings polish and end-to-end verification

**Files:**
- Modify: `src/gui/views/history.rs`
- Modify: `src/gui/views/settings.rs`
- Modify: `src/gui/app.rs`
- Modify: `tests/gui_state.rs`

- [ ] **Step 1: Write the failing recent-operations test**

Add:

```rust
#[test]
fn completed_operation_is_recorded_in_history_view_model() {
    // push a finished operation into app state
    // assert history list length increments
}
```

- [ ] **Step 2: Run the targeted test**

Run: `cargo test completed_operation_is_recorded_in_history_view_model -- --exact`
Expected: FAIL because history state is not wired

- [ ] **Step 3: Implement minimal polish**

Requirements:
- record completed package/import summaries in in-memory recent history
- add a minimal settings panel for default safe mode and optional custom git path placeholder
- ensure navigation badges and status colors are consistent

- [ ] **Step 4: Re-run the targeted history test**

Run: `cargo test completed_operation_is_recorded_in_history_view_model -- --exact`
Expected: PASS

- [ ] **Step 5: Run formatting and the full automated test suite**

Run:

```bash
cargo fmt --check
cargo test
```

Expected:
- `cargo fmt --check`: PASS
- `cargo test`: PASS

- [ ] **Step 6: Run final manual verification**

Run:

```bash
cargo run -- --help
cargo run -- gui
```

Expected:
- help output lists `gui`, `pack`, `import`, `inspect`
- GUI opens with dual workbench navigation

- [ ] **Step 7: Commit**

Run:

```bash
git add src tests
git commit -m "feat: finish gui workflows and verification"
```

## Notes for the implementing agent

- Keep all Git/LFS side effects behind `src/core/import.rs` + `src/core/transaction.rs`.
- Do not let the GUI call `Command` directly.
- Prefer typed paths first; add native file pickers only if time permits after the core is stable.
- Keep safe mode as the default in both GUI and CLI.
- If a temp-repo integration test depends on `git lfs`, guard it with an early availability check instead of making the suite flaky.
- Before claiming success, run `cargo test` and at least one manual GUI launch.
