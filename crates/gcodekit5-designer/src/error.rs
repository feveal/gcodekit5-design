//! Error types for the designer crate.
//!
//! This module provides structured error types for design operations,
//! geometry calculations, and toolpath generation.

use std::io;
use thiserror::Error;

/// Errors that can occur during design operations.
#[derive(Error, Debug)]
pub enum DesignError {
    /// Shape with the given ID was not found.
    #[error("Shape not found: {0}")]
    ShapeNotFound(i32),

    /// Shape with the given name was not found.
    #[error("Shape not found by name: {0}")]
    ShapeNotFoundByName(String),

    /// Invalid shape type for the requested operation.
    #[error("Invalid shape type: expected {expected}, got {actual}")]
    InvalidShapeType { expected: String, actual: String },

    /// Operation requires a selection but nothing is selected.
    #[error("No shape selected")]
    NoSelection,

    /// The design file could not be loaded.
    #[error("Failed to load design: {0}")]
    LoadError(String),

    /// The design file could not be saved.
    #[error("Failed to save design: {0}")]
    SaveError(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// A geometry error occurred during the operation.
    #[error("Geometry error: {0}")]
    Geometry(#[from] GeometryError),

    /// A toolpath error occurred during generation.
    #[error("Toolpath error: {0}")]
    Toolpath(#[from] ToolpathError),
}

/// Errors related to geometric calculations.
#[derive(Error, Debug)]
pub enum GeometryError {
    /// The geometry is invalid or degenerate.
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// The shape has zero or negative dimensions.
    #[error("Invalid dimensions: width={width}, height={height}")]
    InvalidDimensions { width: f64, height: f64 },

    /// A point is outside the valid bounds.
    #[error("Point out of bounds: ({x}, {y})")]
    PointOutOfBounds { x: f64, y: f64 },

    /// The polygon has too few vertices.
    #[error("Polygon requires at least {required} vertices, got {actual}")]
    InsufficientVertices { required: usize, actual: usize },

    /// The path is empty or has no segments.
    #[error("Empty path")]
    EmptyPath,

    /// Boolean operation failed.
    #[error("Boolean operation failed: {0}")]
    BooleanOperationFailed(String),

    /// Offset operation failed.
    #[error("Offset operation failed: {0}")]
    OffsetFailed(String),

    /// Transformation is invalid (e.g., singular matrix).
    #[error("Invalid transformation: {0}")]
    InvalidTransform(String),
}

/// Errors related to toolpath generation.
#[derive(Error, Debug)]
pub enum ToolpathError {
    /// The shape cannot be converted to a toolpath.
    #[error("Cannot generate toolpath for shape: {0}")]
    UnsupportedShape(String),

    /// Invalid toolpath parameters.
    #[error("Invalid toolpath parameters: {0}")]
    InvalidParameters(String),

    /// Tool diameter is too large for the shape.
    #[error(
        "Tool too large: diameter {tool_diameter}mm exceeds shape minimum dimension {shape_min}mm"
    )]
    ToolTooLarge { tool_diameter: f64, shape_min: f64 },

    /// Step-over is invalid.
    #[error("Invalid step-over: {0}")]
    InvalidStepOver(f64),

    /// Depth parameters are invalid.
    #[error("Invalid depth: start={start_depth}, total={total_depth}, step={step_down}")]
    InvalidDepth {
        start_depth: f64,
        total_depth: f64,
        step_down: f64,
    },

    /// Pocket generation failed.
    #[error("Pocket generation failed: {0}")]
    PocketGenerationFailed(String),

    /// No toolpath segments were generated.
    #[error("Empty toolpath generated")]
    EmptyToolpath,

    /// Geometry error during toolpath generation.
    #[error("Geometry error: {0}")]
    Geometry(#[from] GeometryError),
}

/// Result type alias for design operations.
pub type DesignResult<T> = Result<T, DesignError>;

/// Result type alias for geometry operations.
pub type GeometryResult<T> = Result<T, GeometryError>;

/// Result type alias for toolpath operations.
pub type ToolpathResult<T> = Result<T, ToolpathError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_error_display() {
        let err = DesignError::ShapeNotFound(42);
        assert_eq!(err.to_string(), "Shape not found: 42");

        let err = DesignError::NoSelection;
        assert_eq!(err.to_string(), "No shape selected");
    }

    #[test]
    fn test_geometry_error_display() {
        let err = GeometryError::InvalidDimensions {
            width: -10.0,
            height: 5.0,
        };
        assert_eq!(err.to_string(), "Invalid dimensions: width=-10, height=5");

        let err = GeometryError::InsufficientVertices {
            required: 3,
            actual: 2,
        };
        assert_eq!(
            err.to_string(),
            "Polygon requires at least 3 vertices, got 2"
        );
    }

    #[test]
    fn test_toolpath_error_display() {
        let err = ToolpathError::ToolTooLarge {
            tool_diameter: 10.0,
            shape_min: 5.0,
        };
        assert_eq!(
            err.to_string(),
            "Tool too large: diameter 10mm exceeds shape minimum dimension 5mm"
        );
    }

    #[test]
    fn test_error_conversion() {
        let geo_err = GeometryError::EmptyPath;
        let design_err: DesignError = geo_err.into();
        assert!(matches!(design_err, DesignError::Geometry(_)));

        let tp_err = ToolpathError::EmptyToolpath;
        let design_err: DesignError = tp_err.into();
        assert!(matches!(design_err, DesignError::Toolpath(_)));
    }
}
