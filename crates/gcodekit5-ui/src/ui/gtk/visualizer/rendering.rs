//! # Visualizer Rendering
//!
//! Implements the OpenGL rendering pipeline for the visualizer,
//! including toolpath drawing, grid rendering, axis display,
//! and stock material visualization.

use super::*;

use crate::ui::gtk::common::colors;
use gcodekit5_core::constants as core_constants;
use gcodekit5_designer::stock_removal::{SimulationResult, StockMaterial};
use gcodekit5_devicedb::DeviceManager;
use gcodekit5_visualizer::visualizer::GCodeCommand;
use gcodekit5_visualizer::Visualizer;
use std::sync::Arc;
use libadwaita::StyleManager;

impl GcodeVisualizer {
    // Rendering pipeline requires all GL state and geometry parameters.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        cr: &gtk4::cairo::Context,
        vis: &Visualizer,
        _cache: &mut RenderCache,
        width: f64,
        height: f64,
        show_rapid: bool,
        show_cut: bool,
        show_grid: bool,
        show_bounds: bool,
        _show_intensity: bool,
        show_laser: bool,
        show_stock_removal: bool,
        _simulation_result: &Option<SimulationResult>,
        simulation_visualization: &Option<StockRemovalVisualization>,
        _stock_material: &Option<StockMaterial>,
        current_pos: (f32, f32, f32),
        device_manager: &Option<Arc<DeviceManager>>,
        grid_spacing_mm: f64,
        grid_major_line_width: f64,
        grid_minor_line_width: f64,
        style_context: &gtk4::StyleContext,
    ) {
        // Colores del tema
        let fg_color = style_context.color();
        let accent_color = style_context
            .lookup_color("accent_color")
            .unwrap_or(gtk4::gdk::RGBA::new(0.0, 0.5, 1.0, 1.0));
        let success_color = style_context
            .lookup_color("success_color")
            .unwrap_or(gtk4::gdk::RGBA::new(0.0, 0.8, 0.0, 1.0));
        let warning_color = style_context
            .lookup_color("warning_color")
            .unwrap_or(gtk4::gdk::RGBA::new(0.0, 0.8, 1.0, 1.0));

        // Fondo según tema
        let style_manager = StyleManager::default();
        let is_dark_theme = style_manager.is_dark();
        let bg = if is_dark_theme {
            colors::to_rgb_f64(&colors::BACKGROUND_DARK)
        } else {
            colors::to_rgb_f64(&colors::BACKGROUND_LIGHT)
        };
        cr.set_source_rgb(bg.0, bg.1, bg.2);
        let _ = cr.paint();

        // Max S Value
        let max_s_value = if let Some(manager) = device_manager {
            manager
                .get_active_profile()
                .map(|p| p.max_s_value)
                .unwrap_or(1000.0)
        } else {
            1000.0
        };

        // Transformaciones
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let _ = cr.save();
        cr.translate(center_x, center_y);
        cr.scale(vis.zoom_scale as f64, -vis.zoom_scale as f64);
        cr.translate(vis.x_offset as f64, vis.y_offset as f64);

        // Grid
        if show_grid {
            Self::draw_grid(
                cr,
                vis,
                grid_spacing_mm.max(0.1),
                &fg_color,
                grid_major_line_width,
                grid_minor_line_width,
            );
        }

        // Machine Bounds
        if show_bounds {
            if let Some(manager) = device_manager {
                if let Some(profile) = manager.get_active_profile() {
                    let min_x = profile.x_axis.min;
                    let max_x = profile.x_axis.max;
                    let min_y = profile.y_axis.min;
                    let max_y = profile.y_axis.max;
                    let width = max_x - min_x;
                    let height = max_y - min_y;

                    cr.set_source_rgba(
                        accent_color.red() as f64,
                        accent_color.green() as f64,
                        accent_color.blue() as f64,
                        1.0,
                    );
                    cr.set_line_width(3.0 / vis.zoom_scale as f64);
                    cr.rectangle(min_x, min_y, width, height);
                    let _ = cr.stroke();
                }
            }
        }

        // Origin Axes
        let extent = core_constants::WORLD_EXTENT_MM;
        let line_width = 1.0 / vis.zoom_scale as f64;
        crate::ui::gtk::common::rendering::draw_origin_axes(
            cr,
            extent,
            vis.zoom_scale as f64,
            line_width,
        );

        // Stock Removal
        if show_stock_removal {
            if let Some(cached_viz) = simulation_visualization {
                Self::draw_stock_removal_cached(cr, vis, cached_viz);
            }
        }

        // ==========================================
        // DIBUJAR TOOLPATH CON BUCLE UNIFICADO
        // ==========================================

        // Viewport culling
        let half_width_world = (width as f32 / 2.0) / vis.zoom_scale;
        let half_height_world = (height as f32 / 2.0) / vis.zoom_scale;
        let margin = 0.1;
        let margin_x = half_width_world * margin;
        let margin_y = half_height_world * margin;

        let view_min_x = -vis.x_offset - half_width_world - margin_x;
        let view_max_x = -vis.x_offset + half_width_world + margin_x;
        let view_min_y = -vis.y_offset - half_height_world - margin_y;
        let view_max_y = -vis.y_offset + half_height_world + margin_y;

        // Calcular max_s para intensidad
        let max_s = if vis.max_intensity > vis.min_intensity {
            vis.max_intensity
        } else {
            max_s_value as f32
        };

        // BUCLE UNIFICADO
        for cmd in vis.commands() {
            // Viewport culling
            if !Self::is_command_visible(cmd, view_min_x, view_max_x, view_min_y, view_max_y) {
                continue;
            }

            match cmd {
                GCodeCommand::Move { rapid, intensity, .. } => {
                    if *rapid {
                        if show_rapid {
                            Self::draw_with_fixed_color(cr, cmd, &warning_color, vis.zoom_scale);
                        }
                    } else if show_cut {
                        // Si tiene intensidad y show_intensity está activo, usar colores de intensidad
                        if intensity.is_some() {
                            Self::draw_with_intensity(cr, cmd, vis, max_s, vis.zoom_scale);
                        } else {
                            Self::draw_with_fixed_color(cr, cmd, &success_color, vis.zoom_scale);
                        }
                    }
                }
                GCodeCommand::Arc { intensity, .. } => {
                    if show_cut {
                        if intensity.is_some() {
                            Self::draw_with_intensity(cr, cmd, vis, max_s, vis.zoom_scale);
                        } else {
                            Self::draw_with_fixed_color(cr, cmd, &success_color, vis.zoom_scale);
                        }
                    }
                }
                _ => {}
            }
        }

        // Laser/Spindle Position
        if show_laser {
            cr.set_source_rgb(1.0, 0.0, 0.0);
            let radius = 4.0 / vis.zoom_scale as f64;
            cr.arc(
                current_pos.0 as f64,
                current_pos.1 as f64,
                radius,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            let _ = cr.fill();
        }

        let _ = cr.restore();
    }

    // ==========================================
    // FUNCIONES AUXILIARES
    // ==========================================

    /// Dibuja un comando específico (línea o arco) sin establecer color
    fn draw_command(cr: &gtk4::cairo::Context, cmd: &GCodeCommand, zoom_scale: f32) {
        cr.set_line_width(1.5 / zoom_scale as f64);

        match cmd {
            GCodeCommand::Move { from, to, .. } => {
                cr.move_to(from.x as f64, from.y as f64);
                cr.line_to(to.x as f64, to.y as f64);
                let _ = cr.stroke();
            }
            GCodeCommand::Arc {
                from,
                to,
                center,
                clockwise,
                ..
            } => {
                let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt() as f64;
                let start_angle = (from.y - center.y).atan2(from.x - center.x) as f64;
                let end_angle = (to.y - center.y).atan2(to.x - center.x) as f64;

                cr.new_path();
                if *clockwise {
                    cr.arc_negative(center.x as f64, center.y as f64, radius, start_angle, end_angle);
                } else {
                    cr.arc(center.x as f64, center.y as f64, radius, start_angle, end_angle);
                }
                let _ = cr.stroke();
            }
            _ => {}
        }
    }

    /// Dibuja un comando con color fijo
    fn draw_with_fixed_color(
        cr: &gtk4::cairo::Context,
        cmd: &GCodeCommand,
        color: &gtk4::gdk::RGBA,
        zoom_scale: f32,
    ) {
        cr.set_source_rgba(
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            color.alpha() as f64,
        );
        Self::draw_command(cr, cmd, zoom_scale);
    }

    /// Dibuja un comando con color según intensidad
    fn draw_with_intensity(
        cr: &gtk4::cairo::Context,
        cmd: &GCodeCommand,
        vis: &Visualizer,
        max_s: f32,
        zoom_scale: f32,
    ) {
        let intensity = match cmd {
            GCodeCommand::Move { intensity, .. } => intensity.unwrap_or(0.0),
            GCodeCommand::Arc { intensity, .. } => intensity.unwrap_or(0.0),
            _ => 0.0,
        };

        // Si max_s es 0, usar un verde fijo
        if max_s <= 0.0 {
            cr.set_source_rgb(0.0, 0.8, 0.0);
            Self::draw_command(cr, cmd, zoom_scale);
            return;
        }

        let (r, g, b) = if vis.use_intensity_colors {
            vis.get_color_for_intensity(intensity)
        } else {
            let gray = 1.0 - (intensity / max_s).clamp(0.0, 1.0);
            (gray, gray, gray)
        };

        // Asegurar que no sea negro
        let r = r.max(0.05);
        let g = g.max(0.05);
        let b = b.max(0.05);

        cr.set_source_rgb(r as f64, g as f64, b as f64);
        Self::draw_command(cr, cmd, zoom_scale);
    }

    /// Comprueba si un comando es visible en el viewport
    fn is_command_visible(
        cmd: &GCodeCommand,
        view_min_x: f32,
        view_max_x: f32,
        view_min_y: f32,
        view_max_y: f32,
    ) -> bool {
        match cmd {
            GCodeCommand::Move { from, to, .. } => {
                let min_x = from.x.min(to.x);
                let max_x = from.x.max(to.x);
                let min_y = from.y.min(to.y);
                let max_y = from.y.max(to.y);
                !(max_x < view_min_x || min_x > view_max_x || max_y < view_min_y || min_y > view_max_y)
            }
            GCodeCommand::Arc { from, to: _, center, .. } => {
                let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt();
                let arc_min_x = center.x - radius;
                let arc_max_x = center.x + radius;
                let arc_min_y = center.y - radius;
                let arc_max_y = center.y + radius;
                !(arc_max_x < view_min_x || arc_min_x > view_max_x || arc_max_y < view_min_y || arc_min_y > view_max_y)
            }
            _ => true,
        }
    }

    // ==========================================
    // FUNCIONES DE DIBUJO EXISTENTES
    // ==========================================

    pub(crate) fn draw_grid(
        cr: &gtk4::cairo::Context,
        vis: &Visualizer,
        grid_size: f64,
        fg_color: &gtk4::gdk::RGBA,
        major_line_width: f64,
        minor_line_width: f64,
    ) {
        let range = core_constants::WORLD_EXTENT_MM;
        let minor_spacing = grid_size / 5.0;

        // Minor grid lines
        cr.set_source_rgba(
            fg_color.red() as f64,
            fg_color.green() as f64,
            fg_color.blue() as f64,
            0.2,
        );
        cr.set_line_width(minor_line_width / vis.zoom_scale as f64);

        let mut x = -range;
        while x <= range {
            if ((x / grid_size).round() - x / grid_size).abs() > 0.01 {
                cr.move_to(x, -range);
                cr.line_to(x, range);
            }
            x += minor_spacing;
        }

        let mut y = -range;
        while y <= range {
            if ((y / grid_size).round() - y / grid_size).abs() > 0.01 {
                cr.move_to(-range, y);
                cr.line_to(range, y);
            }
            y += minor_spacing;
        }
        let _ = cr.stroke();

        // Major grid lines
        cr.set_source_rgba(
            fg_color.red() as f64,
            fg_color.green() as f64,
            fg_color.blue() as f64,
            0.4,
        );
        cr.set_line_width(major_line_width / vis.zoom_scale as f64);

        let mut x = -range;
        while x <= range {
            cr.move_to(x, -range);
            cr.line_to(x, range);
            x += grid_size;
        }

        let mut y = -range;
        while y <= range {
            cr.move_to(-range, y);
            cr.line_to(range, y);
            y += grid_size;
        }
        let _ = cr.stroke();
    }

    pub(crate) fn draw_stock_removal_cached(
        cr: &gtk4::cairo::Context,
        vis: &Visualizer,
        cached_viz: &StockRemovalVisualization,
    ) {
        cr.set_line_width(1.5 / vis.zoom_scale as f64);

        for layer in &cached_viz.contour_layers {
            cr.set_source_rgba(
                layer.color.0 as f64,
                layer.color.1 as f64,
                layer.color.2 as f64,
                0.7,
            );

            for contour in &layer.contours {
                if contour.len() < 2 {
                    continue;
                }

                cr.move_to(contour[0].0 as f64, contour[0].1 as f64);
                for point in &contour[1..] {
                    cr.line_to(point.0 as f64, point.1 as f64);
                }
                let _ = cr.stroke();
            }
        }
    }
}
