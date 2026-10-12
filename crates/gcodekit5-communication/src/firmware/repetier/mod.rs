//! # Repetier Protocol Firmware Module
//!
//! Encapsulates both the stream processor (aduana) and the asynchronous
//! sender (control de flujo) for 3D Printers.

pub mod processor;
pub mod repetier_sender;

// Re-exportamos para que desde fuera se pueda importar de forma más cómoda
pub use processor::RepetierProcessor;
pub use repetier_sender::RepetierSender;
