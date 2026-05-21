//! Viewport management (zoom, pan, grid) for designer state.

use super::DesignerState;

impl DesignerState {
    /// Toggle grid visibility.
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Toggle toolpath visibility.
    pub fn toggle_toolpaths(&mut self) {
        self.show_toolpaths = !self.show_toolpaths;
    }

    /// Zooms in on the canvas.
    pub fn zoom_in(&mut self) {
        let current = self.canvas.zoom();
        let new_zoom = (current * 1.2).min(50.0);
        self.canvas.set_zoom(new_zoom);
    }

    /// Zooms out on the canvas.
    pub fn zoom_out(&mut self) {
        let current = self.canvas.zoom();
        let new_zoom = (current / 1.2).max(0.1);
        self.canvas.set_zoom(new_zoom);
    }

    /// Zoom to fit all shapes.
    pub fn zoom_fit(&mut self) {
        self.canvas.fit_all_shapes();
    }

    /// Reset view to default (origin at bottom-left with padding).
    pub fn reset_view(&mut self) {
        // Reset zoom to 100%
        self.canvas.set_zoom(1.0);

        // Reset pan to place origin at bottom-left with 5px padding
        // In screen coordinates, (0, height) is bottom-left.
        // We want world (0,0) to be at screen (5, height-5).
        // screen_x = world_x * zoom + pan_x => pan_x = 5
        // screen_y = height - (world_y * zoom + pan_y) => pan_y = 5
        self.canvas.set_pan(5.0, 5.0);
    }
}
