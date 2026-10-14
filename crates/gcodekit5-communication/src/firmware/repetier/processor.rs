//! gcodekit5-communication/src/communication/repetier.rs
//! Stream processor specific to the Repetier protocol
//! Operational controls in Machine Control
//! Commands from the console

/// Structured event extracted from a firmware response
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepetierEvent {
    /// Firmware has confirmed: ok, wait, exec
    Ack,
    /// Firmware requests forwarding from line N
    Resend(u32),
}

/// Result of processing incoming firmware bytes
pub struct IncomingResult {
    pub display: Vec<u8>,
    pub events: Vec<RepetierEvent>,
}

pub struct RepetierProcessor {
    pub line_counter: u32,
    last_known_x: f32,
    last_known_y: f32,
    last_known_z: f32,
    last_state: String,
    pub is_waiting_for_hardware: bool,
}

impl RepetierProcessor {
    pub fn new() -> Self {
        Self {
            line_counter: 0,
            last_known_x: 0.0,
            last_known_y: 0.0,
            last_known_z: 0.0,
            last_state: "Idle".to_string(),
            is_waiting_for_hardware: false,
        }
    }

    pub fn reset(&mut self) {
        self.line_counter = 0;
        self.is_waiting_for_hardware = false;
    }

    pub fn mark_buffer_full(&mut self) {
        self.is_waiting_for_hardware = true;
    }

    pub fn mark_ack(&mut self) {
        self.is_waiting_for_hardware = false;
    }

    /// State reset when firmware signals M30 / ok 14
    /// Resets the N line counter to 0 and clears the buffer
    pub fn on_hardware_reset(&mut self) {
        self.line_counter = 0;
        self.is_waiting_for_hardware = false;
    }

    /// Activates Repetier mode after signature detection.
    /// Leaves the processor in a clean state to begin translation.
    pub fn activate(&mut self) {
        self.reset();
    }

    pub fn calculate_checksum(&self, command: &str) -> u8 {
        let mut checksum: u8 = 0;
        for byte in command.as_bytes() {
            checksum ^= byte;
        }
        checksum
    }

    pub fn generate_mock_grbl_status(&self) -> Vec<u8> {
        let mock_str = format!(
            "<{}|MPos:{:.3},{:.3},{:.3}|Bf:15,64|FS:0,0>\r\n",
            self.last_state, self.last_known_x, self.last_known_y, self.last_known_z
        );
        mock_str.into_bytes()
    }

    // CENTRALIZED SENDING METHOD (Called by both mod.rs and repetier_sender.rs)
    pub fn process_outgoing(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let raw_string = String::from_utf8_lossy(data);
        let cleaned_upper = raw_string.trim().to_ascii_uppercase();

        // Filter for pre-formatted indexed commands originating from repetier_sender.rs itself
        if cleaned_upper.starts_with('N') && cleaned_upper.contains('*') {
            let mut raw_line = cleaned_upper.to_string();
            if let Some(ast_idx) = raw_line.find('*') {
                raw_line = raw_line[..ast_idx].trim().to_string();
            }

            if let Some(first_space_idx) = raw_line.find(' ') {
                let prefix_n = &raw_line[..first_space_idx].trim();
                let remainder_gcode = &raw_line[first_space_idx..].trim();

                let pieces: Vec<&str> = remainder_gcode.split_whitespace().collect();
                let mut normalized_pieces = Vec::new();
                for piece in pieces {
                    if piece.starts_with("M4") {
                        normalized_pieces.push("M3");
                    } else if piece == "X0" || piece == "X0.0" || piece == "X-0" {
                        normalized_pieces.push("X0.00");
                    } else if piece == "Y0" || piece == "Y0.0" || piece == "Y-0" {
                        normalized_pieces.push("Y0.00");
                    } else if piece == "Z0" || piece == "Z0.0" || piece == "Z-0" {
                        normalized_pieces.push("Z0.00");
                    } else {
                        normalized_pieces.push(piece);
                    }
                }
                let normalized_gcode = normalized_pieces.join(" ");
                let cmd = format!("{} {} ", prefix_n, normalized_gcode);
                let ck = self.calculate_checksum(&cmd);
                return Some(format!("{}*{}\n", cmd, ck).into_bytes());
            }
            return Some(format!("{}\n", cleaned_upper).into_bytes());
        }

        let mut translated_payload = String::new();

        // Homing Transmutation / UI Origins
        if cleaned_upper.contains("$H") || cleaned_upper.contains("G28") {
            self.line_counter += 1;
            let cmd = format!("N{} G28 ", self.line_counter);
            let ck = self.calculate_checksum(&cmd);
            translated_payload.push_str(&format!("{}*{}\n", cmd, ck));
            tracing::debug!(
                "📡 [PROCESSOR-REPETIER] Transmutado a G28 Repetier: {}",
                translated_payload.trim()
            );
            return Some(translated_payload.into_bytes());
        }

        // Manual Jog Interception ($J=)
        if cleaned_upper.starts_with("$J=") {
            let extracted = cleaned_upper.trim_start_matches("$J=").trim();
            let clean_move = extracted.replace("G91", "").to_string();
            let pieces: Vec<&str> = clean_move.split_whitespace().collect();
            let axis_coords = pieces.first().unwrap_or(&"X0");

            self.line_counter += 1;
            let cmd1 = format!("N{} G91 ", self.line_counter);
            let ck1 = self.calculate_checksum(&cmd1);
            translated_payload.push_str(&format!("{}*{}\n", cmd1, ck1));

            self.line_counter += 1;
            let cmd2 = format!("N{} G1 {} F2000 ", self.line_counter, axis_coords);
            let ck2 = self.calculate_checksum(&cmd2);
            translated_payload.push_str(&format!("{}*{}\n", cmd2, ck2));
            return Some(translated_payload.into_bytes());
        }

        // Manual Jog Interception (Raw manual console commands or clean lines of G-code)
        if cleaned_upper.starts_with('G')
            || cleaned_upper.starts_with('M')
            || cleaned_upper.starts_with('T')
            || cleaned_upper.starts_with('X')
            || cleaned_upper.starts_with('Y')
            || cleaned_upper.starts_with('Z')
        {
            let mut final_gcode = cleaned_upper.to_string();
            if let Some(comment_idx) = final_gcode.find(';') {
                final_gcode = final_gcode[..comment_idx].to_string();
            }
            if let Some(comment_idx) = final_gcode.find('(') {
                final_gcode = final_gcode[..comment_idx].to_string();
            }
            final_gcode = final_gcode.trim().to_string();

            let first_clean_char = final_gcode.chars().next().unwrap_or(' ');
            if first_clean_char == 'X' || first_clean_char == 'Y' || first_clean_char == 'Z' {
                final_gcode = format!("G1 {}", final_gcode);
            }

            if final_gcode.is_empty() {
                return None;
            }

            let pieces: Vec<&str> = final_gcode.split_whitespace().collect();
            let mut normalized_pieces = Vec::new();
            for piece in pieces {
                if piece.starts_with("M4") {
                    normalized_pieces.push("M3");
                } else if piece == "X0" || piece == "X0.0" || piece == "X-0" {
                    normalized_pieces.push("X0.00");
                } else if piece == "Y0" || piece == "Y0.0" || piece == "Y-0" {
                    normalized_pieces.push("Y0.00");
                } else if piece == "Z0" || piece == "Z0.0" || piece == "Z-0" {
                    normalized_pieces.push("Z0.00");
                } else {
                    normalized_pieces.push(piece);
                }
            }
            let normalized_gcode = normalized_pieces.join(" ");

            self.line_counter += 1;
            let cmd = format!("N{} {} ", self.line_counter, normalized_gcode);
            let ck = self.calculate_checksum(&cmd);
            return Some(format!("{}*{}\n", cmd, ck).into_bytes());
        }

        None
    }

    pub fn process_incoming(&mut self, raw_bytes: Vec<u8>) -> IncomingResult {
        let text = String::from_utf8_lossy(&raw_bytes);
        let lower_text = text.to_lowercase();
        let mut display: Vec<u8> = Vec::new();
        let mut events: Vec<RepetierEvent> = Vec::new();

        // Early detection by Resend to synchronize the global counter
        if lower_text.contains("resend:") {
            if let Some(idx) = lower_text.find("resend:") {
                let num_str: String = lower_text[idx + 7..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(target_line) = num_str.parse::<u32>() {
                    tracing::debug!(
                        "⚠️  [PROCESSOR] Resend detectado. Sincronizando línea global a N{}",
                        target_line
                    );
                }
            }
        }

        // Line-by-line walkthrough
        for line in text.lines() {
            let l = line.trim();

            // Line-by-line walkthrough. Firmware noise/internal echoes: not displayed, do not generate events
            if l.is_empty()
                || l.contains("==")
                || l.contains("recv")
                || l.contains("exec")
                || l.contains("G11")
            {
                continue;
            }

            // Line classification.
            //     - Ack: "ok", "wait", "exec"
            //     - Resend(n): "Resend:N"
            //     - Other: displayed but does not trigger an event
            let lower = l.to_lowercase();

            if lower.starts_with("ok") || lower.contains("wait") {
                events.push(RepetierEvent::Ack);
            }

            if let Some(idx) = l.find("Resend:") {
                let num_str: String = l[idx + 7..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(n) = num_str.parse::<u32>() {
                    events.push(RepetierEvent::Resend(n));
                }
            }

            // Add the line to the display, WITHOUT injecting an extra `ok`.
            display.extend_from_slice(line.as_bytes());
            display.extend_from_slice(b"\r\n");
        }

        // Final queue entry with the GRBL status mock so the UI continues to have
        // data to feed its DROs while we are in Repetier mode.
        display.extend_from_slice(&self.generate_mock_grbl_status());

        IncomingResult { display, events }
    }
}
