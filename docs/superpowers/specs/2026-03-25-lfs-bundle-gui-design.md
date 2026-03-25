# LFS Bundle GUI Design

**Date:** 2026-03-25  
**Status:** Draft for user review  
**Project:** `lfs_bundle`

## 1. Overview

This project will evolve from a command-line utility into a dual-mode application with:

- a **GUI desktop application** for packaging and importing bundle/LFS archives
- a **retained CLI interface** for scripting and automation
- a shared **Rust core layer** that owns all packaging, import, validation, rollback, and cleanup behavior

The GUI must be modern, easy to operate, visually clear, and safe by default. The most important non-functional requirement is **not polluting the user's repository environment**. Temporary files must be isolated, cleaned automatically, and never left behind after successful or failed operations.

## 2. Goals

- Provide a modern desktop GUI for packaging and importing Git bundle + Git LFS data
- Keep the existing console usage model and expand it instead of replacing it
- Let users package changes by selecting a **commit range**
- Let users import a package into an **existing target branch**
- Make import **strictly atomic**
- Default to a **safe mode** that avoids contaminating working repositories
- Offer a **direct mode** for advanced users
- Provide clear status, logs, validation, and rollback visibility

## 3. Non-Goals

- No support in v1 for tag-based or release-based packaging flows
- No support in v1 for importing into a brand-new branch
- No hidden background daemons or long-lived helper services
- No dependency on a web frontend stack for the desktop product

## 4. Product Decisions Already Agreed

### 4.1 Execution modes

Two modes will be supported:

- **Safe mode (default):** use temporary directories and optional temporary worktrees
- **Direct mode:** operate against the selected repository more directly, but still use transactional rollback and temporary staging for intermediate files

### 4.2 Packaging selection model

Packaging is based on a **commit range**, selected by the user from repository history:

- start commit
- end commit

### 4.3 Import behavior

Import must:

1. apply the bundle to a selected **existing branch**
2. only if that succeeds, import/extract LFS objects
3. if any later step fails, **roll back the branch to the original HEAD**
4. clean temporary files regardless of success or failure

This means import is **strictly atomic** from the user perspective.

### 4.4 UI direction

The GUI will use a **dual-workbench layout**:

- **Packaging workbench**
- **Import workbench**

This was chosen over a large linear wizard because the application has two distinct high-value workflows and needs persistent visibility for transaction state, validation, and logs.

## 5. Architecture

The application will be split into three layers.

### 5.1 Core layer

The core layer owns all business behavior and must not depend on GUI-specific logic.

Responsibilities:

- environment detection (`git`, `git lfs`)
- repository inspection
- commit history loading and filtering
- commit-range validation
- bundle generation
- LFS object discovery and packaging
- archive assembly and extraction
- manifest generation and validation
- import transaction execution
- rollback
- temporary directory and temporary worktree lifecycle
- progress/log event emission

This layer is the source of truth for both GUI and CLI behavior.

### 5.2 CLI layer

The CLI remains supported and becomes a thin interface over the core layer.

Responsibilities:

- argument parsing
- mapping CLI input into core requests
- text log output
- exit codes

Illustrative commands:

- `pack --repo <path> --from <commit> --to <commit> --output <archive>`
- `import --repo <path> --branch <name> --archive <file>`
- `inspect --archive <file>`

### 5.3 GUI layer

The GUI is a thin presentation layer over the core.

Responsibilities:

- repository picker
- commit selection UI
- branch selection UI
- package inspection UI
- validation presentation
- progress steps
- logs and rollback feedback

The GUI must not directly implement Git behavior; it only calls core workflows.

## 6. Proposed Tech Stack

- **Language:** Rust
- **GUI:** `egui` / `eframe`
- **CLI parsing:** `clap`
- **Serialization:** `serde`, `serde_json`
- **Temp resources:** `tempfile`
- **Compression/archive:** existing Rust crates plus any additions needed for zip packaging
- **Git/LFS execution:** call `git` / `git lfs` subprocesses in v1

### Why `egui`

`egui` is preferred because it keeps the stack pure Rust, reduces environment complexity, aligns with the current codebase, and is well-suited for utility-style desktop software with forms, tables, progress areas, and logs.

## 7. Package Format

The final export artifact will be a structured archive containing:

- `bundle.bundle`
- `lfs.tar.gz`
- `manifest.json`

### 7.1 Manifest contents

`manifest.json` should include:

- tool version
- packaging timestamp
- source repository identifier
- start commit
- end commit
- target/end commit summary
- bundle file name, size, checksum
- LFS archive file name, size, checksum
- number of LFS objects
- packaging mode / compatibility metadata

### 7.2 Why a manifest is required

The manifest enables:

- pre-import validation
- clear GUI display of archive contents
- checksum verification
- compatibility checks
- future format evolution

## 8. Packaging Workflow

### 8.1 User flow

1. choose source repository
2. load commit history
3. select start commit and end commit
4. validate commit range
5. choose output archive path
6. run packaging

### 8.2 Internal steps

1. validate environment and repository
2. create temporary session directory
3. generate `bundle.bundle`
4. discover LFS objects referenced in the selected range
5. generate `lfs.tar.gz`
6. generate `manifest.json`
7. assemble final archive
8. remove temporary intermediates

### 8.3 Packaging UX requirements

The GUI should show:

- selected repository summary
- start/end commit cards
- commit count estimate
- LFS object count estimate
- final output path
- live step progress
- expandable detailed logs

## 9. Import Workflow

Import is the highest-risk operation and must be transaction-driven.

### 9.1 User flow

1. choose import archive
2. inspect manifest
3. choose target repository
4. choose existing target branch
5. run validations
6. execute atomic import
7. review success or rollback result

### 9.2 Transaction model

Before mutation:

- verify environment
- verify target repository
- verify target branch exists
- verify archive structure
- verify manifest and checksums
- verify bundle applicability
- record original branch HEAD
- create temporary session resources

Execution order:

1. apply bundle to the selected branch
2. verify bundle application result
3. import/extract LFS objects
4. finalize transaction
5. clean temporary files

Rollback behavior:

- if bundle application fails: stop immediately, do not import LFS
- if LFS import fails after bundle success: reset target branch back to original HEAD and remove any newly introduced temporary/import state that can be safely cleaned
- always report rollback status clearly

### 9.3 Atomicity guarantee

The user-visible guarantee is:

- either bundle + LFS import both succeed
- or the target branch is restored to its original commit and temporary artifacts are cleaned

## 10. Safe Mode vs Direct Mode

### 10.1 Safe mode (default)

Safe mode prioritizes repository cleanliness.

Characteristics:

- temporary session directory under system temp
- optional temporary worktree for isolated operations
- no intermediate bundle/archive files placed inside the selected repository
- aggressive cleanup on success and failure

### 10.2 Direct mode

Direct mode exists for advanced users who need fewer indirections.

Characteristics:

- still uses a transaction executor
- still records original HEAD
- still avoids storing intermediate packaging artifacts in the repository root by default
- shows stronger visual warnings in the GUI

## 11. GUI Design

### 11.1 Overall layout

The GUI uses a dual-workbench layout with:

- left navigation rail
- top contextual summary/status area
- main workbench panel
- bottom status/log area

Navigation items:

- Packaging
- Import
- History
- Settings

### 11.2 Packaging workbench

Main sections:

1. **Repository section**
   - repository path picker
   - repository status summary

2. **Commit range section**
   - searchable commit list
   - start commit selector
   - end commit selector
   - display of SHA, title, author, time

3. **Output and preview section**
   - output archive path
   - computed package summary
   - manifest preview

4. **Execution section**
   - progress steps
   - status badges
   - logs

### 11.3 Import workbench

Main sections:

1. **Archive section**
   - archive picker
   - manifest inspection

2. **Target repository section**
   - target repo picker
   - existing branch selector
   - current branch HEAD display

3. **Validation section**
   - checklist with pass/warn/fail states

4. **Transaction section**
   - step-by-step transaction progress:
     - record original HEAD
     - apply bundle
     - verify bundle result
     - import LFS
     - rollback if needed
     - cleanup

5. **Result section**
   - success/failure summary
   - rollback result
   - cleanup result

### 11.4 Visual style

The GUI should feel like a professional engineering tool:

- modern card-based layout
- strong visual hierarchy
- restrained color usage
- clear state colors:
  - green: success/safe
  - blue: running/info
  - yellow: warning
  - red: failure/high risk
- status badges for:
  - Safe Mode
  - Direct Mode
  - Atomic Import
  - Rollback Performed

Avoid excessive animation or decorative effects.

## 12. Logging and Error Handling

Errors must be structured and visible.

### 12.1 Error categories

- environment errors
- input/validation errors
- transaction errors
- rollback errors
- cleanup errors

### 12.2 Presentation requirements

GUI:

- short error summary near the active task
- expandable detailed logs below
- explicit rollback status after failure
- explicit cleanup status after completion

CLI:

- machine-usable exit status
- readable text output
- detailed stderr for failure paths

## 13. Testing Strategy

### 13.1 Unit tests

Test pure logic such as:

- LFS listing parsing
- manifest read/write
- commit-range validation
- temporary path planning
- transaction state transitions

### 13.2 Integration tests

Use temporary repositories to validate:

- package creation
- import success path
- bundle failure prevents LFS import
- LFS failure triggers rollback to original HEAD
- temporary files are cleaned

### 13.3 GUI verification

Focus on:

- form state correctness
- event-to-core request mapping
- progress and error rendering

Heavy GUI automation is not required for v1.

## 14. File/Module Direction

Expected evolution from a single-file prototype to a modular application:

- core domain/workflow modules
- CLI entrypoint and command parsing
- GUI app state and views
- shared progress/log event definitions
- manifest/archive helpers
- transaction/rollback executor

The exact file plan should be created in the implementation planning step.

## 15. Delivery Priorities

Priority order for implementation:

1. extract reusable core from current prototype
2. keep CLI working through the new core
3. implement structured package format with manifest
4. implement safe-mode transaction import
5. implement rollback and cleanup guarantees
6. build dual-workbench GUI
7. polish visual design and diagnostics

## 16. Risks and Mitigations

### Risk: Git/LFS behavior differs across environments
Mitigation: centralize subprocess execution, capture stderr/stdout, validate environment early.

### Risk: rollback logic leaves partial repository state
Mitigation: record original HEAD, isolate temporary artifacts, add integration tests for failure injection.

### Risk: GUI and CLI drift apart
Mitigation: keep all business behavior in the core layer; GUI/CLI remain thin shells.

### Risk: repository pollution through temporary files
Mitigation: use session-scoped temp directories and explicit cleanup in both success and failure paths.

## 17. Acceptance Criteria

The design is considered satisfied when:

- users can package a repository by selecting a commit range
- the produced archive contains bundle, LFS archive, and manifest
- users can import into an existing branch
- import applies bundle before LFS
- if bundle fails, LFS is not imported
- if LFS fails after bundle success, the branch is rolled back to its original HEAD
- temporary artifacts are cleaned automatically
- GUI clearly shows validation, progress, and rollback state
- CLI remains available and uses the same core behavior

