# ADR-009: Communication Protocol Architecture

## Status
Accepted

## Date
2026-02-18

## Context
GCodeKit5 must communicate with CNC controllers over multiple transport layers (USB serial, TCP/IP, WebSocket) and support multiple firmware protocols (GRBL, potentially others in the future). The communication must be non-blocking to keep the UI responsive, handle connection loss gracefully, and support command buffering for streaming G-code files.

## Decision
Implement a **trait-based abstraction** with async I/O in the `gcodekit5-communication` crate:

### Transport Layer
```rust
pub trait Communicator: Send {
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn send(&mut self, data: &str) -> Result<()>;
    fn is_connected(&self) -> bool;
}
```

**Backends:**
- `SerialCommunicator` — USB serial via `serialport` crate
- `TcpCommunicator` — Network TCP/IP connections
- `NoOpCommunicator` — Testing and simulation

### Buffering
`BufferedCommunicatorWrapper` wraps any `Communicator` to provide:
- Command queuing with configurable buffer depth
- Status tracking (sent, acknowledged, errored)
- Flow control based on controller feedback

### Firmware
- `FirmwareDetector` auto-detects controller firmware on connection
- `CapabilityManager` exposes firmware-specific features
- GRBL protocol handler parses `ok`, `error:N`, `ALARM:N`, and `<status>` responses

### Async Architecture
- Built on Tokio for non-blocking I/O
- Event channels for UI notification (connection state, responses, errors)
- Separate read/write tasks for full-duplex communication

## Consequences

**Positive:**
- New transports (Bluetooth, WebSocket) only need to implement `Communicator`
- New firmware protocols only need a parser module
- `NoOpCommunicator` enables full integration testing without hardware
- Non-blocking design keeps UI responsive during long streaming operations

**Negative:**
- Trait objects add a layer of indirection
- Async architecture increases complexity vs synchronous I/O
- Serial port handling varies across OS (Linux udev, macOS IOKit, Windows COM)
- Connection state machine must handle many edge cases (disconnect during stream, alarm during jog)

## Alternatives Considered
- **Direct serial I/O** (no trait abstraction): Simpler but locks out TCP/WebSocket support
- **Synchronous blocking I/O**: Simpler code but freezes UI during communication
- **Plugin-based firmware** (dynamic loading): Maximum flexibility but overkill for current needs
