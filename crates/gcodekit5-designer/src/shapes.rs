//! Geometric shapes for the designer tool.
//! Deprecated: Use crate::model instead.

/// Type of CAM operation to perform on the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperationType {
    #[default]
    Profile,
    Pocket,
}
