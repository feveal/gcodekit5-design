//! # GRBL streaming helpers
//!
//! Pure, GTK-free logic used by the Machine Control panel to talk to a GRBL controller:
//!
//! * [`CharCountWindow`] implements GRBL's recommended *character-counting* streaming
//!   protocol: a line is only sent when it fits, together with every line sent but not
//!   yet acknowledged, into the controller's 128-byte serial RX buffer.
//! * [`ResponseLineSplitter`] turns raw serial reads into complete response lines,
//!   carrying partial lines (and partial UTF-8 sequences) over to the next read.
//! * [`classify_response`] sorts a response line into acks, errors, alarms, status
//!   reports and other messages.
//! * Small helpers for probe reports, axis detection, jog step presets and jog feed
//!   validation.
//!
//! Depends only on `gcodekit5_core::units` and the GRBL status parser types.

use std::collections::VecDeque;

use gcodekit5_communication::ConnectionDriver;
use gcodekit5_communication::firmware::grbl::status_parser::MachinePosition;
use gcodekit5_core::gcode::{
    arc,
    words::{Word, code_number, parse_line, word_value},
};
use gcodekit5_core::units::{
    FeedRateUnits, MeasurementSystem, format_feed_rate, get_unit_label, parse_feed_rate,
};
use std::f64::consts::PI;

/// Usable size of GRBL's serial RX buffer in bytes (the buffer is 128 bytes; one slot is
/// kept free by the ring buffer, so at most 127 bytes may be outstanding).
pub const GRBL_RX_BUFFER_BYTES: usize = 127;

/// Millimetres per inch.
const MM_PER_INCH: f32 = 25.4;
const MAX_ARC_LINEARIZATION_SEGMENTS: usize = 10_000;

/// Returns the number of RX-buffer bytes a G-code line occupies once sent
/// (its bytes plus the terminating newline).
///
/// # Arguments
/// * `line` - The line without its newline.
///
/// # Returns
/// `line.len() + 1`.
pub fn line_cost(line: &str) -> usize {
    line.len() + 1
}

/// Tracks the bytes of lines that were sent to GRBL but not yet acknowledged.
///
/// Every `ok` or `error:N` reply acknowledges exactly one line, in order, so the
/// outstanding line sizes are kept in a FIFO.
#[derive(Debug, Clone)]
pub struct CharCountWindow {
    capacity: usize,
    in_flight: VecDeque<usize>,
    in_flight_bytes: usize,
}

impl Default for CharCountWindow {
    fn default() -> Self {
        Self::new(GRBL_RX_BUFFER_BYTES)
    }
}

impl CharCountWindow {
    /// Creates an empty window.
    ///
    /// # Arguments
    /// * `capacity` - Maximum outstanding bytes (normally [`GRBL_RX_BUFFER_BYTES`]).
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            in_flight: VecDeque::new(),
            in_flight_bytes: 0,
        }
    }

    /// Returns whether a line can be sent now without overflowing the RX buffer.
    ///
    /// # Arguments
    /// * `line` - The line without its newline.
    ///
    /// # Returns
    /// True if `in_flight_bytes + line.len() + 1 <= capacity`.
    pub fn can_send(&self, line: &str) -> bool {
        self.in_flight_bytes + line_cost(line) <= self.capacity
    }

    /// Returns whether a line could ever be sent (fits into an empty buffer).
    ///
    /// # Arguments
    /// * `line` - The line without its newline.
    pub fn fits_at_all(&self, line: &str) -> bool {
        line_cost(line) <= self.capacity
    }

    /// Records that a line was written to the controller.
    ///
    /// # Arguments
    /// * `line` - The line without its newline.
    pub fn on_sent(&mut self, line: &str) {
        let cost = line_cost(line);
        self.in_flight.push_back(cost);
        self.in_flight_bytes += cost;
    }

    /// Records one `ok` / `error:N` reply, which frees the oldest outstanding line.
    ///
    /// # Returns
    /// True if an outstanding line was acknowledged, false for an unexpected reply.
    pub fn on_ack(&mut self) -> bool {
        match self.in_flight.pop_front() {
            Some(cost) => {
                self.in_flight_bytes -= cost;
                true
            }
            None => false,
        }
    }

    /// Bytes currently outstanding in the controller's RX buffer.
    pub fn in_flight_bytes(&self) -> usize {
        self.in_flight_bytes
    }

    /// Number of lines sent but not yet acknowledged.
    pub fn in_flight_lines(&self) -> usize {
        self.in_flight.len()
    }

    /// True when every sent line has been acknowledged.
    pub fn is_empty(&self) -> bool {
        self.in_flight.is_empty()
    }
}

/// Splits raw serial data into complete, trimmed response lines.
///
/// Lines may end with `\n`, `\r` or `\r\n`. Incomplete data is carried over to the next
/// call as bytes, so a UTF-8 sequence split across two reads is decoded correctly.
#[derive(Debug, Default, Clone)]
pub struct ResponseLineSplitter {
    carry: Vec<u8>,
}

impl ResponseLineSplitter {
    /// Creates an empty splitter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds received bytes and returns every line completed by them.
    ///
    /// # Arguments
    /// * `data` - Bytes as returned by one serial read.
    ///
    /// # Returns
    /// Complete lines, trimmed, with empty lines dropped.
    pub fn push(&mut self, data: &[u8]) -> Vec<String> {
        self.carry.extend_from_slice(data);
        let mut lines = Vec::new();
        while let Some(pos) = self.carry.iter().position(|&b| b == b'\n' || b == b'\r') {
            let raw: Vec<u8> = self.carry.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&raw[..raw.len() - 1]);
            let line = text.trim();
            if !line.is_empty() {
                lines.push(line.to_string());
            }
        }
        lines
    }

    /// Returns the bytes of the incomplete line still waiting for its terminator.
    pub fn pending(&self) -> &[u8] {
        &self.carry
    }

    /// Discards any incomplete data.
    pub fn clear(&mut self) {
        self.carry.clear();
    }
}

/// Classification of one GRBL response line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrblResponseLine {
    /// `ok`: one line was accepted.
    Ok,
    /// `error:N`: one line was rejected; holds the code text after `error:`.
    Error(String),
    /// `ALARM:N`: the controller entered an alarm; holds the code text.
    Alarm(String),
    /// `<...>` real-time status report (the whole line).
    Status(String),
    /// Start-up banner (`Grbl 1.1h ['$' for help]`): the controller was reset.
    Reset(String),
    /// Anything else (`[MSG:...]`, `[PRB:...]`, `$n=v`, ...).
    Other(String),
}

/// Classifies a single, trimmed GRBL response line.
///
/// Only an exact `ok` counts as an acknowledgement, so text that merely contains
/// "ok" (for example inside a `[MSG:...]`) is never mistaken for one.
///
/// # Arguments
/// * `line` - One complete response line.
///
/// # Returns
/// The line's [`GrblResponseLine`] kind.
pub fn classify_response(line: &str) -> GrblResponseLine {
    let line = line.trim();
    if line.eq_ignore_ascii_case("ok") {
        return GrblResponseLine::Ok;
    }
    let lower = line.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("error:") {
        return GrblResponseLine::Error(rest.trim().to_string());
    }
    if let Some(rest) = lower.strip_prefix("alarm:") {
        return GrblResponseLine::Alarm(rest.trim().to_string());
    }
    if line.starts_with('<') && line.ends_with('>') {
        return GrblResponseLine::Status(line.to_string());
    }
    if lower.starts_with("grbl ") || lower.starts_with("grblhal ") {
        return GrblResponseLine::Reset(line.to_string());
    }
    GrblResponseLine::Other(line.to_string())
}

/// Parses the success flag of a GRBL probe report such as `[PRB:0.000,0.000,-1.234:1]`.
///
/// # Arguments
/// * `line` - One response line.
///
/// # Returns
/// `Some(true)` for a successful probe, `Some(false)` for a failed one, `None` if the
/// line is not a probe report or its success flag is not an explicit `0` or `1`.
pub fn parse_probe_report(line: &str) -> Option<bool> {
    // The shared parser treats a missing flag as success; a quick probe must
    // only complete on an explicit `:1`.
    let inner = line.trim().strip_prefix("[PRB:")?.strip_suffix(']')?;
    let (_, flag) = inner.rsplit_once(':')?;
    if !matches!(flag.trim(), "0" | "1") {
        return None;
    }
    gcodekit5_communication::firmware::grbl::status_parser::parse_probe_report(line)
        .map(|report| report.success)
}

/// Returns how many axes a status report's position carries (3 to 6).
///
/// # Arguments
/// * `pos` - A parsed `MPos`/`WPos` position.
pub fn reported_axis_count(pos: &MachinePosition) -> u8 {
    if pos.c.is_some() {
        6
    } else if pos.b.is_some() {
        5
    } else if pos.a.is_some() {
        4
    } else {
        3
    }
}

/// One jog step preset.
#[derive(Debug, Clone, PartialEq)]
pub struct JogStepPreset {
    /// Step size in millimetres (internal unit).
    pub mm: f32,
    /// Label in the display unit, e.g. `"0.001 in"`.
    pub label: String,
}

/// Returns the jog step presets for a measurement system.
///
/// Metric offers 0.001–100 mm; imperial offers 0.0001–1 in. Values are always stored
/// in millimetres.
///
/// # Arguments
/// * `units` - The display measurement system.
pub fn jog_step_presets(units: MeasurementSystem) -> Vec<JogStepPreset> {
    let label = get_unit_label(units);
    let (values, factor): (&[f32], f32) = match units {
        MeasurementSystem::Metric => (&[0.001, 0.01, 0.1, 1.0, 10.0, 100.0], 1.0),
        MeasurementSystem::Imperial => (&[0.0001, 0.001, 0.01, 0.1, 1.0], MM_PER_INCH),
    };
    values
        .iter()
        .map(|&v| JogStepPreset {
            mm: v * factor,
            label: format!("{v} {label}"),
        })
        .collect()
}

/// Returns the index of the preset closest (by ratio) to a step size.
///
/// # Arguments
/// * `presets` - Presets from [`jog_step_presets`].
/// * `step_mm` - The current step in millimetres.
///
/// # Returns
/// The closest preset's index, or `None` if `presets` is empty.
/// The jog step choices for `units` with `step_mm` selectable exactly: the
/// presets, plus a custom entry when `step_mm` is not one of them.
///
/// # Returns
/// The choices and the index of the one equal to `step_mm`.
pub fn jog_step_choices(units: MeasurementSystem, step_mm: f32) -> (Vec<JogStepPreset>, usize) {
    let mut presets = jog_step_presets(units);
    if let Some(i) = presets
        .iter()
        .position(|p| (p.mm - step_mm).abs() <= p.mm * 1e-3)
    {
        return (presets, i);
    }
    let display = match units {
        MeasurementSystem::Metric => step_mm,
        MeasurementSystem::Imperial => step_mm / MM_PER_INCH,
    };
    let custom = JogStepPreset {
        mm: step_mm,
        label: format!("{} {}", display, get_unit_label(units)),
    };
    let at = presets.partition_point(|p| p.mm < step_mm);
    presets.insert(at, custom);
    (presets, at)
}

pub fn nearest_jog_step_preset(presets: &[JogStepPreset], step_mm: f32) -> Option<usize> {
    let target = if step_mm.is_finite() && step_mm > 0.0 {
        step_mm.ln()
    } else {
        0.0
    };
    presets
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a.mm.ln() - target).abs();
            let db = (b.mm.ln() - target).abs();
            da.total_cmp(&db)
        })
        .map(|(i, _)| i)
}

/// Parses a jog feed entry into mm/min.
///
/// # Arguments
/// * `text` - The entry text in the display feed unit.
/// * `units` - The display feed unit.
///
/// # Returns
/// The feed in mm/min, or an error message if the text is not a positive number.
pub fn parse_jog_feed(text: &str, units: FeedRateUnits) -> Result<f32, String> {
    let value = parse_feed_rate(text, units)?;
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(format!(
            "jog feed must be a positive number, got '{}'",
            text.trim()
        ))
    }
}

/// Formats a probe move for display in the user's units.
///
/// The command itself is always sent in millimetres (G21); this is only for the
/// console log and tooltips.
///
/// # Arguments
/// * `axis_move` - Signed distance in mm and axis letter, e.g. `('Z', -20.0)`.
/// * `feed_mm_per_min` - Probe feed in mm/min.
/// * `units` - Display length unit.
/// * `feed_units` - Display feed unit.
///
/// # Returns
/// Text such as `"Z-0.79 in at 3.94 in/min"` (2 decimal places).
pub fn format_probe_move(
    axis_move: (char, f32),
    feed_mm_per_min: f32,
    units: MeasurementSystem,
    feed_units: FeedRateUnits,
) -> String {
    let (axis, distance_mm) = axis_move;
    let distance = match units {
        MeasurementSystem::Metric => distance_mm,
        MeasurementSystem::Imperial => distance_mm / MM_PER_INCH,
    };
    let feed: f32 = format_feed_rate(feed_mm_per_min, feed_units)
        .parse()
        .unwrap_or(feed_mm_per_min);
    format!(
        "{axis}{distance:.2} {} at {feed:.2} {feed_units}",
        get_unit_label(units)
    )
}

/// Maps a device profile's connection type to a network driver.
///
/// # Arguments
/// * `connection_type` - The profile's connection type ("Serial", "TCP/IP", "WebSocket").
///
/// # Returns
/// `Some(Tcp)` or `Some(WebSocket)` for network profiles, `None` for serial ones.
pub fn network_driver_for(connection_type: &str) -> Option<ConnectionDriver> {
    let kind = connection_type.trim().to_ascii_lowercase();
    if kind.contains("websocket") {
        Some(ConnectionDriver::WebSocket)
    } else if kind.contains("tcp") || kind == "network" || kind == "telnet" {
        Some(ConnectionDriver::Tcp)
    } else {
        None
    }
}

/// Prepares a program for streaming according to Preferences > File
/// Processing: removes comments (`(...)` and `;...`) unless
/// `preserve_comments` is set, drops lines left empty, and rejects a line
/// longer than `max_line_length` characters, which the controller would
/// refuse (GRBL's line buffer is 80 characters, grblHAL's 256).
///
/// # Returns
/// The program to send, or an error naming the first line that is too long.
pub fn prepare_program(
    content: &str,
    preserve_comments: bool,
    max_line_length: usize,
) -> Result<String, String> {
    prepare_program_with_arc_linearization(
        content,
        preserve_comments,
        max_line_length,
        false,
        0.5,
        0.5,
    )
}

/// Prepares a program for streaming and optionally linearizes short G2/G3 arcs.
///
/// Arc centers and endpoints are interpreted in the active plane, distance mode,
/// and units. Arcs with a radius or endpoint chord below
/// `minimum_arc_length` are emitted as G1 moves no longer than
/// `arc_segment_length` along the arc.
///
/// # Errors
/// Returns an error if a prepared line exceeds `max_line_length` or an arc
/// would require more than the supported segment limit.
pub fn prepare_program_with_arc_linearization(
    content: &str,
    preserve_comments: bool,
    max_line_length: usize,
    linearize_arcs: bool,
    minimum_arc_length: f64,
    arc_segment_length: f64,
) -> Result<String, String> {
    let mut out = String::with_capacity(content.len());
    let mut modal = GcodeModalState::default();
    for (index, raw) in content.lines().enumerate() {
        let line = if preserve_comments {
            raw.trim().to_string()
        } else {
            strip_gcode_comments(raw)
        };
        if line.is_empty() {
            continue;
        }

        let prepared_lines = if linearize_arcs {
            modal.prepare_line(
                raw,
                &line,
                index + 1,
                preserve_comments,
                false,
                ArcLinearizationConfig {
                    minimum_arc_length,
                    arc_segment_length,
                },
            )?
        } else {
            vec![line]
        };
        for prepared_line in prepared_lines {
            let length = prepared_line.chars().count();
            if max_line_length > 0 && length > max_line_length {
                return Err(format!(
                    "line {} has {length} characters; the maximum is {max_line_length}",
                    index + 1
                ));
            }
            out.push_str(&prepared_line);
            out.push('\n');
        }
    }
    Ok(out)
}

/// Linearizes eligible arcs while leaving every other program line unchanged.
///
/// This form is used for generated-code previews and exports. It preserves
/// comments, whitespace, blank lines, and the original line endings unless a
/// line needs conversion or a modal motion command must be restored.
///
/// # Errors
/// Returns an error if an arc would require more than the supported segment limit.
pub fn linearize_program_for_display(
    content: &str,
    minimum_arc_length: f64,
    arc_segment_length: f64,
) -> Result<String, String> {
    let mut out = String::with_capacity(content.len());
    let mut modal = GcodeModalState::default();

    for (index, line_with_ending) in content.split_inclusive('\n').enumerate() {
        let (line, line_ending) = if let Some(line) = line_with_ending.strip_suffix("\r\n") {
            (line, "\r\n")
        } else if let Some(line) = line_with_ending.strip_suffix('\n') {
            (line, "\n")
        } else {
            (line_with_ending, "")
        };
        let prepared_lines = modal.prepare_line(
            line,
            line,
            index + 1,
            true,
            true,
            ArcLinearizationConfig {
                minimum_arc_length,
                arc_segment_length,
            },
        )?;

        if prepared_lines.len() == 1 && prepared_lines[0] == line {
            out.push_str(line_with_ending);
            continue;
        }

        for (line_index, prepared_line) in prepared_lines.iter().enumerate() {
            out.push_str(prepared_line);
            if line_index + 1 < prepared_lines.len() || !line_ending.is_empty() {
                out.push_str(if line_ending.is_empty() {
                    "\n"
                } else {
                    line_ending
                });
            }
        }
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy)]
struct GcodeModalState {
    motion_mode: Option<u8>,
    output_motion_mode: Option<u8>,
    plane: u8,
    absolute_distance: bool,
    absolute_arc_center: bool,
    units_to_mm: f64,
    inverse_time_feed: bool,
    position: [f64; 9],
    absolute_position_known: [bool; 9],
}

#[derive(Clone, Copy)]
struct ArcLinearizationConfig {
    minimum_arc_length: f64,
    arc_segment_length: f64,
}

impl Default for GcodeModalState {
    fn default() -> Self {
        Self {
            motion_mode: Some(0),
            output_motion_mode: Some(0),
            plane: 17,
            absolute_distance: true,
            absolute_arc_center: false,
            units_to_mm: 1.0,
            inverse_time_feed: false,
            position: [0.0; 9],
            absolute_position_known: [false; 9],
        }
    }
}

impl GcodeModalState {
    fn prepare_line(
        &mut self,
        raw_line: &str,
        line: &str,
        line_number: usize,
        preserve_comments: bool,
        preserve_passthrough: bool,
        arc_config: ArcLinearizationConfig,
    ) -> Result<Vec<String>, String> {
        let words = parse_line(line);
        let mut explicit_motion = false;
        let mut reset_coordinates = false;
        let mut invalidate_position = false;
        let mut machine_coordinates = false;
        let mut coordinate_data_command = false;
        let mut non_modal_motion_command = false;

        for word in &words {
            if word.letter != 'G' {
                continue;
            }
            match code_number(word.value) {
                (motion @ 0..=3, 0) => {
                    self.motion_mode = Some(motion as u8);
                    explicit_motion = true;
                }
                (80, 0) => {
                    self.motion_mode = None;
                    explicit_motion = true;
                }
                (17..=19, 0) => self.plane = word.value as u8,
                (20, 0) => self.units_to_mm = 25.4,
                (21, 0) => self.units_to_mm = 1.0,
                (90, 0) => self.absolute_distance = true,
                (91, 0) => self.absolute_distance = false,
                (90, 1) => self.absolute_arc_center = true,
                (91, 1) => self.absolute_arc_center = false,
                (93, 0) => self.inverse_time_feed = true,
                (94 | 95, 0) => self.inverse_time_feed = false,
                (92, 0) => reset_coordinates = true,
                (92, 1 | 2) => invalidate_position = true,
                (54..=59, 0) => invalidate_position = true,
                (53, 0) => machine_coordinates = true,
                (10, 0) => {
                    coordinate_data_command = true;
                    invalidate_position = true;
                }
                (28 | 30, 0) => {
                    invalidate_position = true;
                    non_modal_motion_command = true;
                }
                (38, 2..=5) => {
                    invalidate_position = true;
                    non_modal_motion_command = true;
                }
                _ => {}
            }
        }

        if invalidate_position {
            self.absolute_position_known = [false; 9];
        }

        if reset_coordinates {
            self.set_absolute_coordinates(&words);
            return Ok(vec![if preserve_passthrough {
                line.to_string()
            } else {
                line.trim().to_string()
            }]);
        }

        let has_axis_words = words.iter().any(|word| axis_index(word.letter).is_some());
        let has_arc_words = words.iter().any(|word| {
            axis_index(word.letter).is_some() || matches!(word.letter, 'I' | 'J' | 'K' | 'R')
        });
        let has_repeat_count = words.iter().any(|word| word.letter == 'P');
        let has_checksum = line.contains('*');
        let is_block_delete = line.trim_start().starts_with('/');
        let has_expression = line.contains(['#', '[', ']', '=']);

        if !invalidate_position
            && !machine_coordinates
            && !self.inverse_time_feed
            && arc_config.minimum_arc_length.is_finite()
            && arc_config.minimum_arc_length > 0.0
            && arc_config.arc_segment_length.is_finite()
            && arc_config.arc_segment_length > 0.0
            && !has_repeat_count
            && !has_checksum
            && !is_block_delete
            && !has_expression
            && !non_modal_motion_command
            && !coordinate_data_command
            && has_arc_words
            && matches!(self.motion_mode, Some(2 | 3))
            && let Some(expanded) =
                self.expand_arc(&words, raw_line, line_number, preserve_comments, arc_config)?
        {
            self.output_motion_mode = Some(1);
            return Ok(expanded);
        }

        if has_axis_words {
            if machine_coordinates {
                self.set_machine_coordinates(&words);
            } else if !coordinate_data_command && !invalidate_position {
                self.apply_axis_words(&words);
            }
        }

        if explicit_motion {
            self.output_motion_mode = self.motion_mode;
        }

        let mut output = if preserve_passthrough {
            line.to_string()
        } else {
            line.trim().to_string()
        };
        if has_axis_words
            && !explicit_motion
            && !coordinate_data_command
            && !non_modal_motion_command
            && !machine_coordinates
            && self.output_motion_mode != self.motion_mode
        {
            output = match self.motion_mode {
                Some(mode) => format!("G{mode} {output}"),
                None => format!("G80 {output}"),
            };
            self.output_motion_mode = self.motion_mode;
        }
        Ok(vec![output])
    }

    fn expand_arc(
        &mut self,
        words: &[Word],
        raw_line: &str,
        line_number: usize,
        preserve_comments: bool,
        arc_config: ArcLinearizationConfig,
    ) -> Result<Option<Vec<String>>, String> {
        let clockwise = self.motion_mode == Some(2);
        let (axis_a, axis_b, linear_axis, center_a, center_b) = match self.plane {
            17 => (0, 1, 2, 'I', 'J'),
            18 => (2, 0, 1, 'K', 'I'),
            19 => (1, 2, 0, 'J', 'K'),
            _ => return Ok(None),
        };
        if self.absolute_distance
            && (!self.absolute_position_known[axis_a] || !self.absolute_position_known[axis_b])
        {
            return Ok(None);
        }

        let start = self.position;
        let mut end = start;
        let mut end_known = self.absolute_position_known;
        for word in words {
            if let Some(axis) = axis_index(word.letter) {
                let value = word.value * self.units_to_mm;
                if self.absolute_distance {
                    end[axis] = value;
                    end_known[axis] = true;
                } else {
                    end[axis] += value;
                }
            }
        }

        let start_plane = (start[axis_a], start[axis_b]);
        let end_plane = (end[axis_a], end[axis_b]);
        let delta = (end_plane.0 - start_plane.0, end_plane.1 - start_plane.1);
        let offsets =
            if word_value(words, center_a).is_some() || word_value(words, center_b).is_some() {
                let offset_a = if self.absolute_arc_center {
                    word_value(words, center_a)
                        .map(|value| value * self.units_to_mm - start_plane.0)
                        .unwrap_or(0.0)
                } else {
                    word_value(words, center_a).unwrap_or(0.0) * self.units_to_mm
                };
                let offset_b = if self.absolute_arc_center {
                    word_value(words, center_b)
                        .map(|value| value * self.units_to_mm - start_plane.1)
                        .unwrap_or(0.0)
                } else {
                    word_value(words, center_b).unwrap_or(0.0) * self.units_to_mm
                };
                (offset_a, offset_b)
            } else {
                let Some(radius) = word_value(words, 'R') else {
                    return Ok(None);
                };
                let Some(offsets) = arc::center_offset_from_radius(
                    delta.0,
                    delta.1,
                    radius * self.units_to_mm,
                    clockwise,
                ) else {
                    return Ok(None);
                };
                offsets
            };

        let center = (start_plane.0 + offsets.0, start_plane.1 + offsets.1);
        let Some(sweep) = arc::arc_sweep(center, start_plane, end_plane, clockwise) else {
            return Ok(None);
        };
        let chord_length = delta.0.hypot(delta.1);
        let is_full_circle = chord_length <= 1e-6;
        if sweep.radius >= arc_config.minimum_arc_length
            && (is_full_circle || chord_length >= arc_config.minimum_arc_length)
        {
            return Ok(None);
        }

        let arc_length = sweep.radius * sweep.sweep.abs();
        let segment_count = ((arc_length / arc_config.arc_segment_length).ceil() as usize)
            .max((sweep.sweep.abs() / (PI / 18.0)).ceil() as usize)
            .max(1);
        if segment_count > MAX_ARC_LINEARIZATION_SEGMENTS {
            return Err(format!(
                "line {line_number} needs {segment_count} segments to linearize this arc; maximum is {MAX_ARC_LINEARIZATION_SEGMENTS}"
            ));
        }

        let mut axes = vec![axis_a, axis_b];
        if words
            .iter()
            .any(|word| axis_index(word.letter) == Some(linear_axis))
        {
            axes.push(linear_axis);
        }
        for word in words {
            if let Some(axis) = axis_index(word.letter)
                && !axes.contains(&axis)
            {
                axes.push(axis);
            }
        }

        let extras = format_non_geometry_words(words);
        let comments = if preserve_comments {
            extract_gcode_comments(raw_line)
        } else {
            String::new()
        };
        let mut output = Vec::with_capacity(segment_count);
        let mut previous = start;

        for segment in 1..=segment_count {
            let fraction = segment as f64 / segment_count as f64;
            let mut point = start;
            if segment == segment_count {
                point = end;
            } else {
                let angle = sweep.start_angle + sweep.sweep * fraction;
                point[axis_a] = center.0 + sweep.radius * angle.cos();
                point[axis_b] = center.1 + sweep.radius * angle.sin();
                point[linear_axis] =
                    start[linear_axis] + (end[linear_axis] - start[linear_axis]) * fraction;
                for axis in 3..point.len() {
                    point[axis] = start[axis] + (end[axis] - start[axis]) * fraction;
                }
            }

            let coordinates = axes
                .iter()
                .map(|&axis| {
                    let value = if self.absolute_distance {
                        point[axis] / self.units_to_mm
                    } else {
                        (point[axis] - previous[axis]) / self.units_to_mm
                    };
                    format!("{}{}", axis_letter(axis), format_gcode_number(value))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let mut line = format!("G1 {coordinates}");
            if segment == 1 && !extras.is_empty() {
                line.push(' ');
                line.push_str(&extras);
            }
            if segment == segment_count && !comments.is_empty() {
                line.push(' ');
                line.push_str(&comments);
            }
            output.push(line);
            previous = point;
        }

        self.position = end;
        self.absolute_position_known = end_known;
        Ok(Some(output))
    }

    fn apply_axis_words(&mut self, words: &[Word]) {
        for word in words {
            if let Some(axis) = axis_index(word.letter) {
                let value = word.value * self.units_to_mm;
                if self.absolute_distance {
                    self.position[axis] = value;
                    self.absolute_position_known[axis] = true;
                } else {
                    self.position[axis] += value;
                }
            }
        }
    }

    fn set_absolute_coordinates(&mut self, words: &[Word]) {
        for word in words {
            if let Some(axis) = axis_index(word.letter) {
                self.position[axis] = word.value * self.units_to_mm;
                self.absolute_position_known[axis] = true;
            }
        }
    }

    fn set_machine_coordinates(&mut self, words: &[Word]) {
        for word in words {
            if let Some(axis) = axis_index(word.letter) {
                self.position[axis] = word.value * self.units_to_mm;
                self.absolute_position_known[axis] = false;
            }
        }
    }
}

fn axis_index(letter: char) -> Option<usize> {
    match letter {
        'X' => Some(0),
        'Y' => Some(1),
        'Z' => Some(2),
        'A' => Some(3),
        'B' => Some(4),
        'C' => Some(5),
        'U' => Some(6),
        'V' => Some(7),
        'W' => Some(8),
        _ => None,
    }
}

fn axis_letter(axis: usize) -> char {
    match axis {
        0 => 'X',
        1 => 'Y',
        2 => 'Z',
        3 => 'A',
        4 => 'B',
        5 => 'C',
        6 => 'U',
        7 => 'V',
        8 => 'W',
        _ => unreachable!("axis index is bounded by the coordinate array"),
    }
}

fn format_non_geometry_words(words: &[Word]) -> String {
    words
        .iter()
        .filter(|word| {
            if axis_index(word.letter).is_some() || matches!(word.letter, 'I' | 'J' | 'K' | 'R') {
                return false;
            }
            if word.letter == 'G' {
                return !matches!(code_number(word.value), (0..=3, 0));
            }
            true
        })
        .map(|word| format!("{}{}", word.letter, format_gcode_number(word.value)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_gcode_number(value: f64) -> String {
    let value = if value.abs() < 0.0000005 { 0.0 } else { value };
    let mut formatted = format!("{value:.6}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    formatted
}

fn extract_gcode_comments(line: &str) -> String {
    let mut comments = String::new();
    let mut depth = 0usize;
    for (index, character) in line.char_indices() {
        match character {
            '(' => {
                if depth == 0 {
                    comments.push(' ');
                }
                depth += 1;
                comments.push(character);
            }
            ')' if depth > 0 => {
                depth -= 1;
                comments.push(character);
            }
            ';' if depth == 0 => {
                comments.push(' ');
                comments.push_str(&line[index..]);
                break;
            }
            _ if depth > 0 => comments.push(character),
            _ => {}
        }
    }
    comments
}

/// Removes `( ... )` and `; ...` comments from one G-code line and trims it.
fn strip_gcode_comments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut depth = 0usize;
    for c in line.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            // A `;` inside parentheses is part of that comment, not the start
            // of an end-of-line comment.
            ';' if depth == 0 => break,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// The `$J=` jog for a rotary axis: `delta` degrees at `feed_mm_per_min`.
/// `G21` is stated because the feed is stored in mm/min; a program that left
/// the controller in `G20` would otherwise read it as inches per minute.
pub fn rotary_jog_command(axis: char, delta: f32, feed_mm_per_min: f32) -> String {
    format!("$J=G21 G91 {axis}{delta} F{feed_mm_per_min}\n")
}

/// The steps-per-degree value to send for Preferences text `value`, or
/// `None` when it is not a finite number. Values below 1 become 1, as saving
/// Preferences stores them, so the controller and the config agree.
pub fn steps_per_degree_to_send(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|v| v.max(1.0))
}
