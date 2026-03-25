# GUI Chinese + Picker + Commit Tree Design

**Date:** 2026-03-25  
**Status:** Draft for user review  
**Project:** `lfs_bundle`

## 1. Scope

This is an incremental GUI design on top of the merged dual-workbench baseline.

Requested features:

1. Add Chinese UI.
2. Add native path picker windows.
3. Add a visual commit-selection system that shows the selected repository's commit tree and allows click selection of `start` and `end`.

## 2. Decisions Confirmed With User

### 2.1 UI language

- Default language is Chinese (`zh-CN`).
- Language switching is not implemented in this iteration.
- i18n extension points must be preserved for future multi-language support.

### 2.2 Commit visualization scope

- Show only the currently selected branch (not all branches).
- Prioritize simplicity and clear selection feedback.

### 2.3 Selection interaction

- Single click on a commit row only highlights it.
- Explicit buttons are used to assign highlighted commit as `Start` or `End`.
- Range validity (`Start..End`) is auto-validated and shown in UI.

## 3. Chosen Approach

Selected approach is "lightweight structured upgrade":

- Introduce a minimal i18n layer now.
- Introduce a shared path-picker helper now.
- Introduce a branch-scoped commit tree model now.
- Keep implementation complexity controlled and avoid over-engineering.

This gives immediate user value while keeping clear extension paths.

## 4. Architecture Changes

## 4.1 New GUI i18n module

Create:

- `src/gui/i18n.rs`

Responsibilities:

- Provide translation lookup by key (`tr("...")`).
- Centralize all user-facing UI strings.
- Default locale is `zh-CN`.
- Keep internal model/field names in English.

Initial design:

- enum for `Locale` (currently only `ZhCn` in runtime use)
- static dictionary for `zh-CN`
- fallback behavior for missing keys (return key itself)

## 4.2 New path picker module

Create:

- `src/gui/picker.rs`

Responsibilities:

- Encapsulate native folder/file picker calls.
- Provide focused helpers:
  - pick repository directory
  - pick output archive path
  - pick input archive file
- Keep picker behavior consistent across Packaging/Import workbenches.

## 4.3 Commit tree model and loading

Extend core model/loading:

- Add a commit graph node type for GUI rendering (can live in `src/core/models.rs`).
- Add branch-scoped commit graph loader in `src/core/repo.rs`.

Recommended data shape:

- commit id
- short id
- summary
- author
- timestamp
- parent ids
- graph prefix text (or lane metadata) for visual rendering

The initial implementation favors reliable rendering over complex graph algorithms.

## 5. Packaging Workbench UX Changes

Current free-text `Start`/`End` input flow becomes:

1. Repository path input + "Browse..." button.
2. Branch selector (default current branch).
3. Commit tree panel (current branch only).
4. Selection controls:
   - "Set As Start"
   - "Set As End"
5. Fixed selection summary area:
   - selected `Start`
   - selected `End`
   - range validity status

Validation behavior:

- Before both points selected: warning state.
- Invalid range: error state + disable package action.
- Valid range: success state + allow package action.

## 6. Import Workbench UX Changes

Add native pickers to existing import form:

- Archive path: file picker button.
- Repository path: directory picker button.

All labels, hints, and warnings appear in Chinese via i18n keys.

## 7. Text and Terminology Plan

UI text policy:

- Navigation and labels use Chinese terms.
- Technical identifiers (branch name, commit SHA, command output) remain raw.
- Errors shown to user include Chinese summary + optional technical detail.

Examples:

- Packaging -> "打包"
- Import -> "导入"
- History -> "历史记录"
- Settings -> "设置"
- Start commit -> "起始提交"
- End commit -> "结束提交"
- Range valid -> "提交范围有效"
- Range invalid -> "提交范围无效"

## 8. Testing Strategy

Add/extend tests in three areas:

1. i18n:
   - key lookup returns Chinese text
   - missing key fallback behavior

2. picker integration:
   - state update path after picker returns a path
   - no-op behavior on canceled selection

3. commit selection:
   - selecting highlighted commit as `Start`/`End`
   - range validation states update correctly
   - package request uses selected commit ids instead of manual free text

Existing workflow tests remain unchanged and continue to verify packaging/import behavior.

## 9. Non-Goals

Not included in this iteration:

- Runtime language switcher UI
- Multi-locale translation management tooling
- Full all-branch graph rendering
- High-complexity interactive graph canvas

## 10. Acceptance Criteria

This increment is complete when:

1. GUI labels/hints/status text are Chinese by default.
2. Translation keys are centralized and reusable (i18n extension point exists).
3. Packaging and Import views include native path picker buttons.
4. Packaging view can display current-branch commit tree.
5. User can click a commit, then assign `Start` and `End` via explicit buttons.
6. Range validity is visualized and blocks invalid package requests.
7. Existing `cargo test` suite passes with added tests for new behaviors.
