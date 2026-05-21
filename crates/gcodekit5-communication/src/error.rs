//! Error types for the communication crate.
//!
//! This module provides structured error types for serial/network communication,
//! protocol handling, and firmware interactions.

use std::io;
use thiserror::Error;

/// Errors that can occur during communication operations.
#[derive(Error, Debug)]
pub enum CommunicationError {
    /// Failed to connect to the device.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// The connection was lost unexpectedly.
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    /// Connection timeout.
    #[error("Connection timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// The port is not available or in use.
    #[error("Port unavailable: {port}")]
    PortUnavailable { port: String },

    /// Failed to configure the port.
    #[error("Port configuration failed: {0}")]
    ConfigurationError(String),

    /// Write operation failed.
    #[error("Write failed: {0}")]
    WriteFailed(String),

    /// Read operation failed.
    #[error("Read failed: {0}")]
    ReadFailed(String),

    /// I/O error during communication.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Serial port error.
    #[error("Serial port error: {0}")]
    SerialError(String),

    /// Protocol-level error.
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    /// The device is not connected.
    #[error("Device not connected")]
    NotConnected,

    /// The device is busy processing another command.
    #[error("Device busy")]
    DeviceBusy,

    /// Buffer overflow - too much data.
    #[error("Buffer overflow: received {received} bytes, max {max}")]
    BufferOverflow { received: usize, max: usize },
}

/// Errors related to communication protocols.
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Invalid command format.
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// Invalid response from device.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Unsupported protocol version.
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),

    /// Checksum mismatch.
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// Command rejected by device.
    #[error("Command rejected: {reason}")]
    CommandRejected { reason: String },

    /// Unknown error code from device.
    #[error("Unknown error code: {code}")]
    UnknownErrorCode { code: i32 },

    /// Parsing error for response data.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// The response was incomplete.
    #[error("Incomplete response: expected {expected} bytes, got {actual}")]
    IncompleteResponse { expected: usize, actual: usize },

    /// Firmware reported an error.
    #[error("Firmware error: {message} (code: {code})")]
    FirmwareError { code: i32, message: String },

    /// Command not supported by firmware.
    #[error("Command not supported: {command}")]
    UnsupportedCommand { command: String },

    /// Alarm state - machine requires attention.
    #[error("Alarm: {message}")]
    Alarm { message: String },
}

/// Errors related to firmware operations.
#[derive(Error, Debug)]
pub enum FirmwareError {
    /// Unknown firmware type.
    #[error("Unknown firmware: {0}")]
    UnknownFirmware(String),

    /// Firmware version not supported.
    #[error("Firmware version {version} not supported, minimum: {minimum}")]
    VersionNotSupported { version: String, minimum: String },

    /// Failed to detect firmware type.
    #[error("Firmware detection failed: {0}")]
    DetectionFailed(String),

    /// Setting not found.
    #[error("Setting not found: {0}")]
    SettingNotFound(String),

    /// Invalid setting value.
    #[error("Invalid setting value for {setting}: {value}")]
    InvalidSettingValue { setting: String, value: String },

    /// Protocol error from firmware interaction.
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
}

/// Result type alias for communication operations.
pub type CommunicationResult<T> = Result<T, CommunicationError>;

/// Result type alias for protocol operations.
pub type ProtocolResult<T> = Result<T, ProtocolError>;

/// Result type alias for firmware operations.
pub type FirmwareResult<T> = Result<T, FirmwareError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communication_error_display() {
        let err = CommunicationError::ConnectionFailed("port busy".to_string());
        assert_eq!(err.to_string(), "Connection failed: port busy");

        let err = CommunicationError::Timeout { timeout_ms: 5000 };
        assert_eq!(err.to_string(), "Connection timeout after 5000ms");

        let err = CommunicationError::NotConnected;
        assert_eq!(err.to_string(), "Device not connected");
    }

    #[test]
    fn test_protocol_error_display() {
        let err = ProtocolError::ChecksumMismatch {
            expected: "0x1234".to_string(),
            actual: "0x5678".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Checksum mismatch: expected 0x1234, got 0x5678"
        );

        let err = ProtocolError::Alarm {
            message: "Hard limit triggered".to_string(),
        };
        assert_eq!(err.to_string(), "Alarm: Hard limit triggered");
    }

    #[test]
    fn test_firmware_error_display() {
        let err = FirmwareError::VersionNotSupported {
            version: "1.0".to_string(),
            minimum: "1.1".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Firmware version 1.0 not supported, minimum: 1.1"
        );
    }

    #[test]
    fn test_error_conversion() {
        let proto_err = ProtocolError::InvalidResponse("bad format".to_string());
        let comm_err: CommunicationError = proto_err.into();
        assert!(matches!(comm_err, CommunicationError::Protocol(_)));

        let proto_err = ProtocolError::Alarm {
            message: "test".to_string(),
        };
        let fw_err: FirmwareError = proto_err.into();
        assert!(matches!(fw_err, FirmwareError::Protocol(_)));
    }
}
