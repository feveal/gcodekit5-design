# REVIEW.CAMTOOLS.md

## Scope
This review covers the **CAM Tools** tab UI implementation and proposes a more space-efficient, consistent, and discoverable design.

**Primary implementation location:**
- `crates/gcodekit5-ui/src/ui/gtk/cam_tools.rs` (`CamToolsView` + tool pages)
- Tab wiring: `crates/gcodekit5-ui/src/gtk_app.rs` (stack child `"cam_tools"`)

---

## Key observations (current)
- The CAM Tools dashboard is a fixed 3-column `Grid` of large cards (space-heavy on FHD @ 125%).
- Most tools re-implement similar patterns (header + back button, Paned layout, sidebar sizing).
- Several tool pages force a fixed sidebar ratio continuously via `paned.add_tick_callback(...)` (fights user resizing).
- Parameter UX is mixed: many `Entry` fields for numeric input (no range/stepper), and inconsistent units/help.
- Progress/cancel behavior is inconsistent: some tools use modal progress windows; others likely do background work differently.

---

## Recommendations (actionable)

### 1) Replace the dashboard “card grid” with a compact tool list + details panel
**Change:** Use a left `ListBox` (or `PreferencesGroup` sections) listing tools with icon + title + one-line description; right panel shows selected tool description + “Open” button.

**Why:** Cards consume lots of vertical + horizontal space and don’t scale well to small widths.

**Done when:** On FHD @ 125%, the dashboard shows all tools without scrolling, or minimal scrolling.

**Execution prompt:** In `crates/gcodekit5-ui/src/ui/gtk/cam_tools.rs` `CamToolsView::create_dashboard()`, replace `Grid` + `create_tool_card()` with `ListBox` rows and a details pane; keep `stack.set_visible_child_name(...)` navigation.

---

### 2) Add a dashboard toolbar row (Search + Category filter)
**Change:** Add a top row with:
- search entry (“Search tools…”) filtering tool list rows
- segmented filter (“Generators”, “Engraving”, “Calculators”, “Maintenance”, “All”)

**Why:** Improves discoverability and makes the dashboard usable as tool count grows.

**Done when:** Typing “spoil” filters down to surfacing/grid; “engrave” filters to bitmap/vector.

**Execution prompt:** In `CamToolsView::create_dashboard()` add a header `gtk::Box` above the list and filter the list model/rows.

---

### 3) Normalize icon usage (use symbolic icon names consistently)
**Change:** Ensure all icons use either `*-symbolic` names or shipped resource icons; avoid non-symbolic fallback strings.

**Why:** Mixed icon styles look inconsistent in Adwaita dark theme.

**Done when:** All dashboard rows and tool headers use consistent symbolic styling.

**Execution prompt:** In `create_dashboard()` and `create_tool_card()` usages, standardize icon names (e.g. `insert-image-symbolic` instead of `insert-image`).

---

### 4) Standardize tool page layout: header + content + footer actions
**Change:** For each tool page:
- header row: Back, Tool Title, optional help (“?”)
- main content: split view for Preview (left) + Parameters (right) where relevant
- footer: persistent action row (Load/Save/Cancel/Generate)

**Why:** Users build muscle memory across tools.

**Done when:** All tools share the same structural layout and action placement.

**Execution prompt:** In `cam_tools.rs`, refactor each tool’s `new()` to use a shared builder function (or helper struct) that constructs the header + paned + action footer.

---

### 5) Stop forcing sidebar width every frame; allow user resizing + persist position
**Change:** Remove `paned.add_tick_callback(...)` sizing loops. Instead:
- set initial position once on first size-allocate
- allow user to resize
- persist per-tool paned position in settings

**Why:** The current tick callback prevents user control and causes layout jank.

**Done when:** Users can drag the split and it stays.

**Execution prompt:** In tool pages (e.g. Jigsaw at `paned.add_tick_callback` around where it sets 40%), replace with a one-time sizing hook + persistence via `gcodekit5-settings`.

---

### 6) Use numeric widgets with units (SpinButton or validated Entry)
**Change:** Replace free-form numeric `Entry` with:
- `SpinButton` (with step/page increments and min/max)
- or an `Entry` with input validation + error styling

Add units consistently as suffix labels (`mm`, `mm/min`, `RPM`, `%`).

**Why:** Reduces invalid input and improves speed.

**Done when:** Invalid values are visibly rejected and do not silently parse to defaults.

**Execution prompt:** In each tool’s parameter construction (e.g. Jigsaw `Entry::builder().text(...)` block), replace with `SpinButton` and bounds.

---

### 7) Add inline “Estimated runtime / path length / output size” summary
**Change:** Add a read-only summary group near Generate:
- estimated time
- bounds/output size
- expected passes

**Why:** Users want feedback before committing to generation.

**Done when:** The user can see expected output dimensions and a time estimate at a glance.

**Execution prompt:** In each tool page, add a `PreferencesGroup` called “Summary” and update it whenever parameters change.

---

### 8) Unify progress + cancellation UI across tools (non-modal)
**Change:** Replace per-tool modal progress windows with a consistent non-blocking progress surface:
- either a small overlay in the preview area
- or a shared status bar mechanism with cancel

**Why:** Modal progress breaks flow and feels like the app is frozen.

**Done when:** All generators show progress and can be cancelled without blocking the UI.

**Execution prompt:** In `cam_tools.rs` background generation paths (e.g. Jigsaw’s `progress_window`), replace with shared progress presenter (same pattern as Visualizer/Designer). Track cancellation via a shared token.

---

### 9) Improve “Load/Save” UX (default folder + recent presets)
**Change:**
- use a consistent default directory preference
- add a “Recent presets” dropdown (or “Load recent” section)

**Why:** These tools are parameter-heavy; re-use is common.

**Done when:** Users can re-open the last-used preset in 1 click.

**Execution prompt:** Extend existing load/save hooks in each tool to remember last-used preset path; add a small “Recent” menu to the footer.

---

### 10) Add per-tool help popover (“?”) listing shortcuts and workflow
**Change:** Add a help button that shows:
- what the tool does
- recommended workflow
- key shortcuts (if any)

**Why:** Reduces documentation hunting.

**Done when:** User can learn “what does Offset X/Y do” without leaving the tool.

**Execution prompt:** In each tool header, add a `Button` with `help-about-symbolic` or `question-round-symbolic` and show a `Popover` with short help text.

---

### 11) Make preview + parameters collapsible on narrow widths
**Change:** When window width is small:
- collapse to a single column with tabs: “Preview” / “Parameters”

**Why:** Split views don’t work well at narrow widths.

**Done when:** On narrow widths, no horizontal squashing occurs.

**Execution prompt:** In tool pages using `Paned`, add width threshold logic to swap layout to a `Stack` with two pages.

---

### 12) Add consistent “Generate → open in Editor” confirmation
**Change:** After successful generation, show a brief non-blocking toast: “Generated G-code loaded into Editor” with an “Open Editor” action.

**Why:** The auto-switch is useful but can be surprising; toasts improve clarity.

**Done when:** Generation completion is obvious and reversible.

**Execution prompt:** In `gtk_app.rs` `CamToolsView::new(move |gcode| { ... })`, keep the auto-switch but also trigger a toast/snackbar on success.

---

## Notes
- The CAM tools already use `libadwaita::{PreferencesGroup, ActionRow}` in parameter panels. Lean into this and standardize all tool pages to the same Adw-ish patterns (consistent group titles, spacing, and footer actions).
