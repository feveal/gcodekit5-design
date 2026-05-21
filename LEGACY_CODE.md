# Legacy & Unused Code Inventory

This document catalogues legacy, deprecated, and unused code in the gcodekit5
codebase. Items listed here are candidates for removal or refactoring.

---

## 1. Legacy Directory (`legacy/`)

The `legacy/` directory contains ~6,400 lines of code from an earlier
architecture. None of this code is compiled into the current application.

| File | Lines | Purpose |
|------|------:|---------|
| `callbacks/cam.rs` | 2,126 | Old CAM tool callback handlers |
| `callbacks/designer.rs` | 1,581 | Old designer callback handlers |
| `callbacks/machine.rs` | 964 | Old machine control callbacks |
| `callbacks/editor.rs` | 740 | Old editor callbacks |
| `callbacks/settings.rs` | 241 | Old settings callbacks |
| `callbacks/mod.rs` | 4 | Module declaration |
| `app/designer.rs` | 353 | Old designer application logic |
| `app/helpers.rs` | 352 | Old helper functions |
| `app/types.rs` | 42 | Old type definitions |

**Recommendation**: Safe to delete entirely if no longer referenced.

---

## 2. Slint Legacy Test Infrastructure

The project migrated from Slint to GTK4. Remnants of the Slint UI layer remain
behind a `slint_legacy_tests` feature gate (never enabled in normal builds).

| File | Lines | Notes |
|------|------:|-------|
| `crates/gcodekit5-ui/src/editor/slint_bridge.rs` | 23 | Stub module for legacy test builds |
| `crates/gcodekit5-ui/src/editor/mod.rs` | 331 | Contains `cfg(feature = "slint_legacy_tests")` gates |
| `crates/gcodekit5-ui/tests/ui/state_test.rs` | 40 | Gated behind `slint_legacy_tests` |
| `crates/gcodekit5-ui/tests/ui/file_management_test.rs` | 41 | Gated behind `slint_legacy_tests` |
| `crates/gcodekit5-ui/tests/ui/console_output_debug.rs` | 45 | Gated behind `slint_legacy_tests` |
| `crates/gcodekit5-ui/tests/ui/console_listener.rs` | 90 | Gated behind `slint_legacy_tests` |
| `crates/gcodekit5-ui/tests/ui/console_panel_test.rs` | 127 | Gated behind `slint_legacy_tests` |

**Total**: ~697 lines of effectively dead code.

**Recommendation**: Remove the feature gate, `slint_bridge.rs`, and the five
gated test files. Simplify `editor/mod.rs` to remove the conditional compilation.

---

## 3. Dead Code Suppressions (`#[allow(dead_code)]`)

43 instances across 21 files (excluding `legacy/` and build artifacts).

### 3a. Main Window Callback Fields (19 instances)

**File**: `crates/gcodekit5-ui/src/ui/main_window.rs` (lines 350–387)

19 callback fields (`on_refresh_ports_cb`, `on_connect_cb`,
`on_disconnect_cb`, `on_menu_view_machine_cb`, `on_machine_zero_all_cb`,
`on_machine_emergency_stop_cb`, `on_machine_jog_home_cb`,
`on_machine_jog_x_positive_cb`, `on_machine_jog_x_negative_cb`,
`on_machine_jog_y_positive_cb`, `on_machine_jog_y_negative_cb`,
`on_machine_jog_z_positive_cb`, `on_machine_jog_z_negative_cb`,
`on_generate_tabbed_box_cb`, `on_generate_jigsaw_puzzle_cb`,
`on_generate_spoilboard_surfacing_cb`, `on_generate_spoilboard_grid_cb`,
`on_generate_laser_engraving_cb`, `on_generate_vector_engraving_cb`) are
all marked `#[allow(dead_code)]`.

**Recommendation**: Wire up or remove. These are stored but never read back,
suggesting the callback registration pattern is incomplete.

### 3b. Spatial Manager (5 instances)

**File**: `crates/gcodekit5-designer/src/spatial_manager.rs`

Five methods marked dead_code: utility methods for spatial queries that are
implemented but not yet called from application code.

**Recommendation**: Keep — these are intentionally pre-built for future use.
Consider removing the `#[allow(dead_code)]` once callers exist.

### 3c. Visualizer (5 instances)

**Files**:
- `crates/gcodekit5-visualizer/src/visualizer/visualizer.rs` (4 instances)
- `crates/gcodekit5-visualizer/src/visualizer/mesh_renderer.rs` (1 instance)
- `crates/gcodekit5-visualizer/src/visualizer/stock_removal_3d.rs` (2 instances)
- `crates/gcodekit5-visualizer/src/helpers.rs` (1 instance)

**Recommendation**: Audit each — some may be leftover from prototyping.

### 3d. Communication Crate (4 instances)

**Files**:
- `crates/gcodekit5-communication/src/firmware/grbl/controller.rs` (1)
- `crates/gcodekit5-communication/src/firmware/grbl/response_parser.rs` (1)
- `crates/gcodekit5-communication/src/firmware/tinyg/controller.rs` (2)
- `crates/gcodekit5-communication/src/firmware/tinyg/response_parser.rs` (1)

**Recommendation**: Review — may be protocol constants reserved for future use.

### 3e. Other (5 instances)

| File | Notes |
|------|-------|
| `crates/gcodekit5-camtools/src/vector_engraver.rs` | 1 field |
| `crates/gcodekit5-designer/src/canvas/types.rs` | 1 enum/struct |
| `crates/gcodekit5-settings/src/manager.rs` | 1 field |
| `crates/gcodekit5-ui/src/ui/gtk/cam_tools/mod.rs` | 1 field |
| `crates/gcodekit5-ui/src/ui/gtk/visualizer/rendering.rs` | 1 field |
| `crates/gcodekit5-ui/src/ui/gtk/visualizer/mod.rs` | 1 field |
| `crates/gcodekit5-ui/src/ui/gtk/config_settings/mod.rs` | 1 field |
| `crates/gcodekit5-ui/src/ui/gtk/settings.rs` | 1 field |

---

## 4. Legacy Type Aliases

| Alias | Location | Notes |
|-------|----------|-------|
| `pub type MachineStatus = MachineStatusSnapshot` | `crates/gcodekit5-core/src/data/mod.rs:841` | Backward-compat alias; used in `lib.rs` re-exports |
| `pub type BoxedResult<T> = Result<T, BoxedError>` | `crates/gcodekit5-core/src/types/aliases.rs:204` | Defined for "legacy compatibility" but never used outside its own test |

**Recommendation**: Search for consumers. If none exist beyond re-exports,
remove the aliases and update the re-export lists.

---

## 5. Deprecated Fields Kept for Compatibility

| Field | Location | Notes |
|-------|----------|-------|
| `halftone_threshold: u8` | `crates/gcodekit5-camtools/src/laser_engraver.rs:63` | Marked "deprecated - kept for compatibility"; still referenced in bitmap_engraving UI |

**Recommendation**: If the UI still uses it, it may not be truly deprecated.
Clarify whether to remove or un-deprecate.

---

## 6. Legacy Comments and Path Handling

| Location | Notes |
|----------|-------|
| `crates/gcodekit5-camtools/src/tabbed_box/mod.rs:471` | Comment: "If path_groups is empty (legacy or bug)" — defensive code for old file formats |
| `crates/gcodekit5-visualizer/src/gcode/parser.rs:343` | Method doc: "for backward compatibility" |
| `crates/gcodekit5-ui/src/helpers.rs:198–199` | "Enable legacy RT commands" — Grbl setting |

**Recommendation**: The Grbl "legacy RT" setting is a real firmware option, not
dead code. The tabbed-box and parser items are defensive compatibility code
that should be kept.

---

## Summary

| Category | Lines | Priority |
|----------|------:|----------|
| `legacy/` directory | ~6,400 | High — entirely unused |
| Slint legacy infrastructure | ~697 | High — feature never enabled |
| Dead-code callback fields | 19 fields | Medium — incomplete wiring |
| Other dead_code suppressions | 24 instances | Low — audit individually |
| Unused type aliases | 2 aliases | Low — trivial cleanup |
| Deprecated fields | 1 field | Low — needs clarification |
