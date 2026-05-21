# Aspect Ratio Lock Feature

**Date**: January 10, 2026  
**Status**: Implemented  
**Component**: Designer / Properties Panel

## Overview

Added a "Lock Aspect" checkbox to the shape inspector in the Properties Panel that allows users to maintain the aspect ratio of shapes when resizing them by editing width or height values.

## Implementation Details

### UI Changes

1. **New Checkbox Widget**: Added `lock_aspect_ratio: CheckButton` to the `PropertiesPanel` struct
2. **Size Section Updated**: Added a new row in the size grid with:
   - Label: "Lock Aspect:"
   - CheckButton widget for toggling the lock

### State Management

1. **Aspect Ratio Storage**: Added `aspect_ratio: Rc<RefCell<f64>>` field to store the current width/height ratio
2. **Lock Toggle Handler**: When the checkbox is activated, the current aspect ratio is calculated and stored from the width and height entry values

### Resize Logic

The aspect ratio maintenance logic was added to four handlers:

1. **width_entry.connect_activate**: When width is changed and lock is active, height is automatically adjusted
2. **height_entry.connect_activate**: When height is changed and lock is active, width is automatically adjusted
3. **width_focus_controller.connect_leave**: Same logic for focus-out events on width entry
4. **height_focus_controller.connect_leave**: Same logic for focus-out events on height entry

### Algorithm

When aspect ratio is locked:
- **Changing Width**: `new_height = new_width / aspect_ratio`
- **Changing Height**: `new_width = new_height * aspect_ratio`

The opposite dimension entry is automatically updated with the calculated value before the shape resize is applied.

## User Experience

1. User selects a shape in the designer
2. In the Properties Panel, the Size section shows Width, Height, and Lock Aspect fields
3. User checks the "Lock Aspect" checkbox
   - Current aspect ratio (width/height) is captured
4. When user edits width:
   - Height automatically updates to maintain aspect ratio
   - Shape is resized with both new dimensions
5. When user edits height:
   - Width automatically updates to maintain aspect ratio
   - Shape is resized with both new dimensions
6. User can uncheck "Lock Aspect" to resize dimensions independently

## Code Location

- File: `crates/gcodekit5-ui/src/ui/gtk/designer_properties.rs`
- Lines affected:
  - Struct field additions: ~63, ~124
  - UI creation: ~221-228
  - Lock toggle handler: ~911-924
  - Width handlers: ~800-831, ~1865-1905
  - Height handlers: ~860-891, ~1925-1965

## Benefits

- Common design tool feature that users expect
- Prevents accidental distortion of shapes when resizing
- Useful for maintaining proportions in logos, text, and other design elements
- Works seamlessly with the existing undo system

## Testing

The implementation:
- ✅ Compiles without errors
- ✅ Maintains existing width/height editing behavior when lock is disabled
- ✅ Properly updates both dimensions when lock is enabled
- ✅ Uses the existing undo-aware resize methods
- ✅ Handles both single and multi-selection cases
