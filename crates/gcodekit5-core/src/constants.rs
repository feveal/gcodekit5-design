//! Shared constants for view/visualizer/designer behavior
//
// This module defines shared defaults used across UI modules such as the
// default working area size, world extent, and padding used when fitting view
// bounds.

/// Default working area width (mm) used when there's no active device profile.
pub const DEFAULT_WORK_WIDTH_MM: f64 = 250.0;

/// Default working area height (mm) used when there's no active device profile.
pub const DEFAULT_WORK_HEIGHT_MM: f64 = 250.0;

/// Default fractional padding used by `fit_to_bounds` - 5% per edge
pub const VIEW_PADDING: f64 = 0.05;

/// Default world extent (in mm) for canvas/world bounds (+/- this value)
pub const WORLD_EXTENT_MM: f64 = 2000.0;

/// Canvas padding (in pixels) used by the Visualizer for content inset.
/// Keep this distinct so UI pixel sizing isn't tied to world-space defaults.
pub const CANVAS_PADDING_PX: f64 = 20.0;
