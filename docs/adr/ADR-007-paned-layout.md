# ADR-007: Paned Layout for Resizable Panels

## Status
Accepted

## Date
2026-02-18

## Context
GCodeKit5 has many UI panels (Connection, DRO, Jog, Editor, Console, Visualizer, Overrides, Macros, Settings) that need to be visible simultaneously on varying screen sizes. Users operate on screens from 1080p at 125% scaling to 4K monitors, and need the ability to resize panels to prioritize different workflows (e.g., more space for the visualizer during preview, more space for the editor during G-code editing).

Early approaches used fixed layouts, which wasted space on large screens and cramped content on smaller ones.

## Decision
Implement a centralized `LayoutManager` with a `PanelState` model that tracks each panel's visibility, location, dock state, and size. Panels are assigned to regions (`Left`, `Center`, `Right`, `Bottom`) and can be toggled visible/hidden independently.

Key design choices:
- **GTK4-native Paned widgets** for resizable dividers between regions
- **Serializable layout state** (`PanelState`) for persistence across sessions
- **Predefined layouts** (Default, Designer, Operator) as starting points
- **Panel locations are configurable** — panels can move between Left/Center/Right/Bottom

```rust
pub enum PanelLocation {
    Left,
    Center,
    Right,
    Bottom,
}

pub struct PanelState {
    pub id: PanelId,
    pub visible: bool,
    pub location: PanelLocation,
    pub docked: bool,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
```

## Consequences

**Positive:**
- Users can resize panels to match their workflow and screen size
- Layout persistence means users don't lose their configuration
- Predefined layouts reduce initial setup friction
- GTK4-native approach avoids custom layout engine complexity

**Negative:**
- Panel minimum sizes must be carefully managed to prevent unusable states
- More complex state management than fixed layouts
- Layout serialization adds a potential point of failure on startup

## Alternatives Considered
- **Fixed CSS Grid layout**: Simpler but inflexible for different screen sizes
- **Floating/detachable windows**: More flexible but poor UX with tiling window managers
- **Tab-based panels**: Hides panels, requiring clicks to switch — bad for monitoring workflows
