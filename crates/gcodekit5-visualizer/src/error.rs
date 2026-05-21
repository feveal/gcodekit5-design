//! Error types for the visualizer crate.
//!
//! This module provides structured error types for G-code parsing,
//! visualization, and file processing operations.

use std::io;
use thiserror::Error;

/// Errors that can occur during visualization operations.
#[derive(Error, Debug)]
pub enum VisualizationError {
    /// Failed to initialize the visualizer.
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Invalid viewport configuration.
    #[error("Invalid viewport: {0}")]
    InvalidViewport(String),

    /// Rendering operation failed.
    #[error("Rendering failed: {0}")]
    RenderingFailed(String),

    /// Camera configuration is invalid.
    #[error("Invalid camera: {0}")]
    InvalidCamera(String),

    /// The toolpath is empty or invalid.
    #[error("Empty or invalid toolpath")]
    InvalidToolpath,

    /// File I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Parsing error.
    #[error("Parsing error: {0}")]
    Parsing(#[from] ParsingError),

    /// Memory allocation failed.
    #[error("Memory allocation failed: required {required_bytes} bytes")]
    AllocationFailed { required_bytes: usize },

    /// Unsupported file format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Errors related to G-code parsing.
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Invalid G-code syntax.
    #[error("Syntax error at line {line}: {message}")]
    SyntaxError { line: usize, message: String },

    /// Unknown G-code command.
    #[error("Unknown command at line {line}: {command}")]
    UnknownCommand { line: usize, command: String },

    /// Invalid parameter value.
    #[error("Invalid parameter at line {line}: {param}={value}")]
    InvalidParameter {
        line: usize,
        param: String,
        value: String,
    },

    /// Missing required parameter.
    #[error("Missing parameter at line {line}: {param} required for {command}")]
    MissingParameter {
        line: usize,
        command: String,
        param: String,
    },

    /// File could not be read.
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    /// Invalid arc specification.
    #[error("Invalid arc at line {line}: {reason}")]
    InvalidArc { line: usize, reason: String },

    /// Coordinate out of bounds.
    #[error("Coordinate out of bounds at line {line}: {axis}={value}")]
    CoordinateOutOfBounds { line: usize, axis: char, value: f64 },

    /// Unexpected end of file.
    #[error("Unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },

    /// I/O error during parsing.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

/// Errors related to file processing.
#[derive(Error, Debug)]
pub enum FileError {
    /// File not found.
    #[error("File not found: {path}")]
    NotFound { path: String },

    /// Permission denied.
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    /// File is too large.
    #[error("File too large: {size} bytes exceeds limit of {max_size} bytes")]
    TooLarge { size: u64, max_size: u64 },

    /// Invalid file format.
    #[error("Invalid file format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    /// Encoding error.
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

/// Result type alias for visualization operations.
pub type VisualizationResult<T> = Result<T, VisualizationError>;

/// Result type alias for parsing operations.
pub type ParsingResult<T> = Result<T, ParsingError>;

/// Result type alias for file operations.
pub type FileResult<T> = Result<T, FileError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_error_display() {
        let err = VisualizationError::InitializationFailed("GL context failed".to_string());
        assert_eq!(err.to_string(), "Initialization failed: GL context failed");

        let err = VisualizationError::InvalidToolpath;
        assert_eq!(err.to_string(), "Empty or invalid toolpath");
    }

    #[test]
    fn test_parsing_error_display() {
        let err = ParsingError::SyntaxError {
            line: 42,
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "Syntax error at line 42: unexpected token");

        let err = ParsingError::MissingParameter {
            line: 10,
            command: "G2".to_string(),
            param: "I or J".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Missing parameter at line 10: I or J required for G2"
        );
    }

    #[test]
    fn test_file_error_display() {
        let err = FileError::TooLarge {
            size: 100_000_000,
            max_size: 50_000_000,
        };
        assert_eq!(
            err.to_string(),
            "File too large: 100000000 bytes exceeds limit of 50000000 bytes"
        );
    }

    #[test]
    fn test_error_conversion() {
        let parse_err = ParsingError::UnexpectedEof { line: 100 };
        let viz_err: VisualizationError = parse_err.into();
        assert!(matches!(viz_err, VisualizationError::Parsing(_)));
    }
}
