# GUI Chinese + Picker + Commit Tree Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Chinese-first UI (with i18n extension points), native path picker dialogs, and a current-branch visual commit tree for selecting Start/End commits in the packaging flow.

**Architecture:** Keep business logic in `src/core/*` and UI behavior in `src/gui/*`. Add a small i18n module and picker module in GUI, and add branch commit-tree loading in core. Packaging form state will no longer depend on manual commit hash typing; it will be driven by highlighted commit + explicit Set Start/Set End actions.

**Tech Stack:** Rust 2024, `egui`/`eframe`, `clap`, `serde`, `anyhow`, existing git subprocess helpers, `rfd` (native file/folder dialogs)

---

## Proposed File Structure

### Modify

- `Cargo.toml` - add native picker dependency (`rfd`)
- `src/gui/mod.rs` - export i18n/picker modules
- `src/gui/app.rs` - switch visible labels to i18n keys
- `src/gui/views/pack.rs` - add commit-tree UI, Start/End assignment controls, picker integration
- `src/gui/views/import.rs` - add picker integration and i18n text
- `src/gui/views/history.rs` - i18n labels
- `src/gui/views/settings.rs` - i18n labels
- `src/core/models.rs` - add commit-tree node and packaging selection state types
- `src/core/repo.rs` - add branch-scoped commit-tree loading and range validation helper
- `tests/gui_state.rs` - update/add form behavior tests

### Create

- `src/gui/i18n.rs` - Chinese default dictionary and `tr()` lookup
- `src/gui/picker.rs` - file/folder picker wrappers
- `tests/i18n.rs` - i18n behavior tests
- `tests/repo_commit_tree.rs` - commit-tree loading/range-validation tests

---

## Chunk 1: i18n Foundation (Chinese default + extension points)

### Task 1: Add i18n module and tests

**Files:**
- Create: `src/gui/i18n.rs`
- Create: `tests/i18n.rs`
- Modify: `src/gui/mod.rs`

- [ ] **Step 1: Write failing i18n tests**

Create `tests/i18n.rs` with:

```rust
use lfs_bundle::gui::i18n::{set_locale, tr, Locale};

#[test]
fn zh_cn_default_key_returns_chinese_text() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("nav.packaging"), "打包");
}

#[test]
fn missing_key_falls_back_to_key_literal() {
    set_locale(Locale::ZhCn);
    assert_eq!(tr("missing.key"), "missing.key");
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test i18n`
Expected: FAIL because i18n module does not exist.

- [ ] **Step 3: Implement minimal i18n module**

Add `src/gui/i18n.rs` with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale { ZhCn }

pub fn set_locale(locale: Locale);
pub fn tr(key: &str) -> &'static str;
```

Requirements:
- runtime locale defaults to `ZhCn`
- dictionary includes at least nav keys and common form labels
- unknown key returns key literal fallback

Update `src/gui/mod.rs`:

```rust
pub mod i18n;
pub mod picker;
```

- [ ] **Step 4: Re-run i18n tests**

Run: `cargo test --test i18n`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/gui/mod.rs src/gui/i18n.rs tests/i18n.rs
git commit -m "feat: add chinese-first i18n foundation"
```

## Chunk 2: Path Picker Integration

### Task 2: Add picker wrapper and packaging/import form path actions

**Files:**
- Create: `src/gui/picker.rs`
- Modify: `src/gui/views/pack.rs`
- Modify: `src/gui/views/import.rs`
- Modify: `tests/gui_state.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write failing GUI state tests for picker-updated paths**

Extend `tests/gui_state.rs`:

```rust
#[test]
fn pack_state_accepts_repo_path_from_picker_result() {
    let mut state = lfs_bundle::gui::views::pack::PackViewState::default();
    state.apply_repo_path_from_picker(Some("D:/repo".into()));
    assert_eq!(state.repo_path, "D:/repo");
}

#[test]
fn import_state_ignores_cancelled_picker_result() {
    let mut state = lfs_bundle::gui::views::import::ImportViewState::default();
    state.archive_path = "existing.zip".into();
    state.apply_archive_path_from_picker(None);
    assert_eq!(state.archive_path, "existing.zip");
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test gui_state`
Expected: FAIL because helper methods do not exist.

- [ ] **Step 3: Add picker module and wire form helpers**

Add `src/gui/picker.rs` wrappers:

```rust
pub fn pick_repo_dir() -> Option<std::path::PathBuf>;
pub fn pick_archive_file() -> Option<std::path::PathBuf>;
pub fn pick_output_archive_file() -> Option<std::path::PathBuf>;
```

Use `rfd::FileDialog` in wrappers.

In `pack.rs` and `import.rs`:
- add methods to apply picker result (`Option<PathBuf>`) safely
- add Browse buttons calling picker wrappers
- keep text input + picker dual entry model

In `Cargo.toml` add:

```toml
rfd = "0.15"
```

- [ ] **Step 4: Re-run GUI state tests**

Run: `cargo test --test gui_state`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/gui/picker.rs src/gui/views/pack.rs src/gui/views/import.rs tests/gui_state.rs
git commit -m "feat: add native path picker integration"
```

## Chunk 3: Current-Branch Commit Tree Data + Validation

### Task 3: Add branch commit-tree loader in core

**Files:**
- Modify: `src/core/models.rs`
- Modify: `src/core/repo.rs`
- Create: `tests/repo_commit_tree.rs`

- [ ] **Step 1: Write failing commit-tree loader tests**

Create `tests/repo_commit_tree.rs`:

```rust
use anyhow::Result;
use lfs_bundle::core::repo::{load_branch_commit_tree, is_valid_commit_range};

#[test]
fn load_branch_commit_tree_returns_current_branch_commits() -> Result<()> {
    // setup temp repo with two commits on master
    // assert returned list length >= 2 and ids are populated
    Ok(())
}

#[test]
fn commit_range_validation_detects_invalid_order() -> Result<()> {
    // setup temp repo with two commits
    // assert start..end valid and reverse invalid
    Ok(())
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test repo_commit_tree`
Expected: FAIL because APIs do not exist.

- [ ] **Step 3: Implement minimal tree model + core APIs**

In `models.rs` add:

```rust
pub struct CommitTreeNode {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub author: String,
    pub timestamp: String,
    pub parents: Vec<String>,
    pub graph_prefix: String,
}
```

In `repo.rs` add:

```rust
pub fn load_branch_commit_tree(repo: &Path, branch: &str, limit: usize) -> Result<Vec<CommitTreeNode>>;
pub fn is_valid_commit_range(repo: &Path, start: &str, end: &str) -> Result<bool>;
```

Implementation notes:
- use branch-scoped `git log` (not all branches)
- parse parent hashes from `%P`
- initial graph prefix can be simple (`*`, `|`, etc.) and improved later
- range validity check via `git merge-base --is-ancestor start end`

- [ ] **Step 4: Re-run repo commit tree tests**

Run: `cargo test --test repo_commit_tree`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/models.rs src/core/repo.rs tests/repo_commit_tree.rs
git commit -m "feat: add current-branch commit tree loading"
```

## Chunk 4: Packaging View Commit Tree Selection UX

### Task 4: Replace free-text start/end flow with highlighted commit + explicit assignment

**Files:**
- Modify: `src/gui/views/pack.rs`
- Modify: `src/gui/app.rs`
- Modify: `tests/gui_state.rs`

- [ ] **Step 1: Write failing GUI selection tests**

Extend `tests/gui_state.rs`:

```rust
#[test]
fn clicking_commit_then_set_start_updates_start_commit() {
    // create state with commit nodes
    // set highlighted index
    // call set_highlighted_as_start()
    // assert start_commit updated
}

#[test]
fn invalid_commit_range_blocks_request_generation() {
    // set start/end in invalid order
    // assert to_request() returns None when range_valid == false
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test gui_state`
Expected: FAIL because new selection API does not exist.

- [ ] **Step 3: Implement commit tree state + controls**

In `PackViewState`, add:
- loaded `Vec<CommitTreeNode>`
- current branch name
- highlighted commit index
- selected start/end ids
- `range_valid` state

Add methods:

```rust
pub fn set_highlighted_as_start(&mut self);
pub fn set_highlighted_as_end(&mut self);
pub fn refresh_range_validity(&mut self, repo: &Path) -> Result<()>;
```

UI requirements:
- commit tree list (current branch only)
- click row = highlight only
- explicit buttons:
  - `设为起始提交`
  - `设为结束提交`
- fixed summary area:
  - start
  - end
  - validity status text
- disable package action on invalid range

- [ ] **Step 4: Re-run GUI state tests**

Run: `cargo test --test gui_state`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/gui/views/pack.rs src/gui/app.rs tests/gui_state.rs
git commit -m "feat: add commit-tree start-end selection workflow"
```

## Chunk 5: Chinese Text Rollout Across GUI

### Task 5: Switch visible GUI copy to i18n keys and Chinese text

**Files:**
- Modify: `src/gui/app.rs`
- Modify: `src/gui/views/pack.rs`
- Modify: `src/gui/views/import.rs`
- Modify: `src/gui/views/history.rs`
- Modify: `src/gui/views/settings.rs`
- Modify: `src/gui/i18n.rs`

- [ ] **Step 1: Write failing smoke assertions for key labels**

Add/extend tests in `tests/gui_state.rs` or `tests/i18n.rs`:

```rust
#[test]
fn chinese_nav_labels_exist_in_dictionary() {
    assert_eq!(lfs_bundle::gui::i18n::tr("nav.packaging"), "打包");
    assert_eq!(lfs_bundle::gui::i18n::tr("nav.import"), "导入");
}
```

- [ ] **Step 2: Run targeted tests**

Run: `cargo test --test i18n`
Expected: FAIL until all keys are defined.

- [ ] **Step 3: Replace hard-coded English UI strings**

Replace labels in GUI view files with i18n lookups:
- nav titles
- section headings
- form labels
- validation/warning text
- action button text

Keep technical content (SHA, branch names, command detail) unchanged.

- [ ] **Step 4: Re-run i18n and GUI state tests**

Run:

```bash
cargo test --test i18n
cargo test --test gui_state
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/gui/app.rs src/gui/views/pack.rs src/gui/views/import.rs src/gui/views/history.rs src/gui/views/settings.rs src/gui/i18n.rs tests/i18n.rs tests/gui_state.rs
git commit -m "feat: localize gui labels to chinese via i18n keys"
```

## Chunk 6: End-to-End Verification

### Task 6: Full verification and launch checks

**Files:**
- Modify: `tests/cli_smoke.rs` (only if needed for wording changes)

- [ ] **Step 1: Run formatter check**

Run: `cargo fmt --check`
Expected: PASS.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 3: Manual CLI sanity check**

Run:

```bash
cargo run -- --help
```

Expected:
- subcommands still include `gui`, `pack`, `import`, `inspect`
- no regression in CLI help output

- [ ] **Step 4: Manual GUI launch check**

Run:

```bash
cargo run -- gui
```

Expected:
- GUI opens
- Chinese labels visible
- Packaging tab shows commit tree area and Start/End controls
- Browse buttons visible for path fields

- [ ] **Step 5: Commit final polish changes (if any)**

```bash
git add src tests Cargo.toml
# only commit if working tree changed after verification fixes
git commit -m "chore: finalize chinese picker commit-tree polish"
```

## Notes for Implementers

- Keep this increment focused on current-branch tree only.
- Do not add runtime language switching UI in this scope.
- Do not regress atomic import behavior already covered by integration tests.
- Keep picker wrappers thin and test form-state behavior rather than OS dialog internals.
- If `rfd` backend has platform caveats, keep typed path entry fully usable as fallback.
