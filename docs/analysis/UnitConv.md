# Implementing Selectable Units (Metric/Imperial)

This guide describes the steps required to implement selectable measurement units (Metric vs. Imperial) for a dialog or panel in GCodeKit5. The system allows users to view and enter values in their preferred unit system (mm or inches), while the application internally maintains all values in millimeters.

## 1. UI Implementation (.slint)

### Add Unit Label Property
Add a property to your component to hold the current unit label. This will be set by the backend ("mm" or "in").

### Remove Spinboxes ###
Spinboxes cant handle imperial values, so convert all spinboxes to TextEdit controls. 

### Update Input Properties
Ensure all dimension-related properties are of type `string`. This is necessary to support fractional Imperial inputs (e.g., "1 1/2") and formatted decimal strings.

### Update UI Labels
Update text labels to dynamically display the unit.

### Update Input Fields
Change `input-type` to `text` to allow for flexible input formats.

## 2. Backend Implementation (Rust)

### Imports
Import the unit conversion utilities from the core crate.

```rust
use gcodekit5_core::units::{MeasurementSystem, to_display_string, parse_from_string, get_unit_label};
use std::str::FromStr;
```

### Retrieve Measurement System
Get the user's preferred measurement system from the settings configuration.

```rust
let system = {
    let persistence = settings_persistence.borrow();
    let sys_str = &persistence.config().ui.measurement_system;
    MeasurementSystem::from_str(sys_str).unwrap_or(MeasurementSystem::Metric)
};
```

### Initialize Dialog
Set the unit label and convert default/initial values to the display format.


### Processing User Input
When generating G-code or processing data, parse the 
### Handling Dynamic Updates
If your dialog updates values programmatically (e.g., selecting a preset or device), ensure you convert those values to the current display system.



## Summary Checklist

- [ ] **UI**: Add `unit-label` property.
- [ ] **UI**: Convert dimension properties to `string`.
- [ ] **UI**: Update labels to use `unit-label`.
- [ ] **UI**: Set `input-type: text` on LineEdits.
- [ ] **Rust**: Import `gcodekit5_core::units` modules.
- [ ] **Rust**: Get `MeasurementSystem` from settings.
- [ ] **Rust**: Set `unit_label` on dialog.
- [ ] **Rust**: Use `to_display_string` for initialization and updates.
- [ ] **Rust**: Use `parse_from_string` for reading inputs.
