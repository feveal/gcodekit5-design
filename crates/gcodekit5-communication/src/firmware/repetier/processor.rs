//! gcodekit5-communication/src/communication/repetier.rs
//! Procesador de flujo específico para el protocolo Repetier
//! Controles operativos en Machine Control
//! Comandos desde la consola

use std::time::Instant;

/// Evento estructurado extraído de una respuesta del firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepetierEvent {
    /// Firmware ha confirmado: ok, wait, exec.
    Ack,
    /// Firmware pide reenviar desde la línea N.
    Resend(u32),
}

/// Resultado de procesar bytes entrantes del firmware.
pub struct IncomingResult {
    /// Bytes que la UI debe mostrar / reenviar.
    pub display: Vec<u8>,
    /// Eventos estructurados para el control de flujo.
    pub events: Vec<RepetierEvent>,
}

pub struct RepetierProcessor {
    pub line_counter: u32,
    last_known_x: f32,
    last_known_y: f32,
    last_known_z: f32,
    last_state: String,
    connect_time: Option<Instant>,
    pub initialized_printer: bool,
    pub inject_buffer: Vec<Vec<u8>>,
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
            connect_time: Some(Instant::now()),
            initialized_printer: false,
            inject_buffer: Vec::new(),
            is_waiting_for_hardware: false,
        }
    }

    pub fn reset(&mut self) {
        self.line_counter = 0;
        self.connect_time = Some(Instant::now());
        self.initialized_printer = false;
        self.inject_buffer.clear();
        self.is_waiting_for_hardware = false;
    }

    /// Marca que el firmware ha recibido un comando y no acepta más hasta su ack.
    pub fn mark_buffer_full(&mut self) {
        self.is_waiting_for_hardware = true;
    }

    /// Marca que el firmware ha confirmado (ok / wait / exec) y libera el buffer.
    pub fn mark_ack(&mut self) {
        self.is_waiting_for_hardware = false;
    }

    /// Consulta si el buffer de hardware está lleno.
    pub fn is_buffer_full(&self) -> bool {
        self.is_waiting_for_hardware
    }

    /// Reset de estado cuando el firmware señala M30 / ok 14.
    /// Devuelve el contador de líneas N a 0 y libera el buffer.
    pub fn on_hardware_reset(&mut self) {
        self.line_counter = 0;
        self.is_waiting_for_hardware = false;
    }

    /// Activa el modo Repetier tras la detección de firma.
    /// Deja el processor en estado limpio para empezar a traducir.
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

    // 🌟 1. MÉTODO DE ENVÍO CENTRALIZADO (Llamado tanto por mod.rs como por repetier_sender.rs)
    pub fn process_outgoing(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let raw_string = String::from_utf8_lossy(data);
        let cleaned_upper = raw_string.trim().to_ascii_uppercase();

        // Filtro contra comandos indexados ya formateados que vengan del propio repetier_sender.rs
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

        // Transmutación de Homing / Orígenes de la UI
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

        // Intercepción de Jog manual ($J=)
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

        // Comandos de consola manuales en crudo o líneas limpias de G-code
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

        // 1) Detección temprana de Resend para sincronizar el contador global.
        //    Usamos lower_text para que sea robusto ante "Resend:" / "resend:".
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

        // 2) Recorrido línea a línea.
        for line in text.lines() {
            let l = line.trim();

            // Ruido / ecos internos del firmware: no se muestran, no generan eventos.
            if l.is_empty()
                || l.contains("==")
                || l.contains("recv")
                || l.contains("exec")
                || l.contains("G11")
            {
                continue;
            }

            // 2a) Clasificación de la línea.
            //     - Ack: "ok", "wait", "exec" (exec ya filtrado arriba, pero por si acaso)
            //     - Resend(n): "Resend:N"
            //     - Resto: se muestra pero no genera evento.
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

            // 2b) Añadimos la línea al display tal cual, SIN inyectar `ok` extra.
            display.extend_from_slice(line.as_bytes());
            display.extend_from_slice(b"\r\n");
        }

        // 3) Cola final con el mock de status GRBL para que la UI siga teniendo
        //    con qué alimentar sus DROs mientras estamos en modo Repetier.
        display.extend_from_slice(&self.generate_mock_grbl_status());

        IncomingResult { display, events }
    }
}
