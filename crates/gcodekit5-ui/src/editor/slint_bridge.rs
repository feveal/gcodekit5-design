//! Slint legacy test bridge.
//
// This module exists primarily to satisfy optional legacy builds/tests.
// The GTK UI uses `gcodekit5_gcodeeditor` directly.

pub use gcodekit5_gcodeeditor::EditorBridgeBackend as EditorBridge;

#[derive(Clone, Debug)]
pub struct SlintTextLine {
    pub line_number: i32,
    pub content: String,
    pub is_dirty: bool,
}

impl SlintTextLine {
    pub fn new(line_number: usize, content: String, is_dirty: bool) -> Self {
        Self {
            line_number: line_number as i32,
            content,
            is_dirty,
        }
    }
}
