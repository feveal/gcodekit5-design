# ADR-008: Internal Units System

## Status
Accepted

## Date
2026-02-18

## Context
CNC machines operate with both metric (mm) and imperial (inch) measurement systems. G-code files may use either system (G21 for metric, G20 for imperial). Feed rates can be expressed in mm/min, mm/sec, in/min, or in/sec. Users expect the UI to display values in their preferred system while the application must perform consistent internal calculations.

Handling dual units throughout the codebase would double the testing surface and create conversion bugs at every calculation boundary.

## Decision
All internal calculations and storage use **millimeters (mm)** for dimensions and **mm/min** for feed rates. Conversion to/from the user's preferred display units happens exclusively at the UI boundary layer.

The `gcodekit5_core::units` module provides the conversion API:

```rust
pub enum MeasurementSystem { Metric, Imperial }
pub enum FeedRateUnits { MmPerMin, MmPerSec, InPerMin, InPerSec }

pub fn format_length(value_mm: f32, system: MeasurementSystem) -> String;
pub fn format_feed_rate(rate_mm_min: f32, units: FeedRateUnits) -> String;
pub fn parse_length(input: &str, system: MeasurementSystem) -> Option<f32>;
```

Key rules:
- Dimensions are `f32` in mm, displayed to 2 decimal places (0.01mm precision)
- DateTime values are stored in UTC, converted to locale in the UI layer
- Text strings are internally UTF-8
- The UI listens for measurement system changes and re-renders displayed values
- G-code parser preserves the source unit mode but the visualizer works in mm

## Consequences

**Positive:**
- Single representation eliminates unit mismatch bugs in calculations
- Conversion logic is centralized and testable
- New features don't need to handle unit variants
- Consistent precision across the application (0.01mm)

**Negative:**
- Rounding errors accumulate during metric↔imperial roundtrips
- Must remember to convert at UI boundaries — forgetting causes display bugs
- G-code generation must re-apply the correct unit prefix (G20/G21)

## Alternatives Considered
- **Store in user's preferred units**: Simpler display but every calculation needs unit awareness
- **Use f64 everywhere**: Better precision but unnecessary for CNC (0.01mm is sufficient) and doubles memory
- **Unit-typed wrappers** (e.g., `Millimeters(f32)`): Type-safe but adds verbosity to every arithmetic operation
