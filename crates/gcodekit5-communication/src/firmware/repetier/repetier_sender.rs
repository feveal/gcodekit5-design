//! # Repetier-Specific Stream Sender with Block Control & Auto-Rewind
//!
//! Controls the streaming thread for 3D Printers using the Repetier protocol,
//! preventing buffer overflow with a safe 10-line window and processing
//! Resend requests in real-time by rolling back the file pointer.

use crate::communication::Communicator;
use crate::firmware::repetier::processor::RepetierEvent;
use crate::SerialCommunicator;
use gcodekit5_core::ThreadSafe;
use std::sync::mpsc::{channel, Receiver, Sender as MpscSender};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct RepetierSender {
    tx_control: MpscSender<SenderCommand>,
}

enum SenderCommand {
    SendGcode(String),
    Pause,
    Resume,
    Stop,
}

impl RepetierSender {
    pub fn new(
        communicator: ThreadSafe<SerialCommunicator>,
        is_streaming: ThreadSafe<bool>,
        is_paused: ThreadSafe<bool>,
        _waiting_for_ack: ThreadSafe<bool>,
    ) -> (Self, Receiver<String>) {
        let (tx_control, rx_control) = channel();
        let (tx_ui, rx_ui) = channel();
        let tx_ui_thread = tx_ui.clone();

        thread::spawn(move || {
            let mut file_lines: Vec<String> = Vec::new();
            let mut current_idx: usize = 0;
            let mut commands_in_flight: usize = 0;
            let mut local_streaming = false;
            let mut local_paused = false;
            let mut last_retry_idx: Option<usize> = None;

            loop {
                // 1. PROCESAR COMANDOS DE CONTROL DESDE LA UI
                while let Ok(cmd) = rx_control.try_recv() {
                    match cmd {
                        SenderCommand::SendGcode(content) => {
                            file_lines = content
                                .lines()
                                .map(|s| s.trim())
                                .filter(|s| {
                                    let s = s.trim();
                                    !s.is_empty()
                                        && !s.starts_with(';')
                                        && !s.starts_with('(')
                                })
                                .map(|s| s.to_string())
                                .collect();
                            tracing::debug!("📋 [SENDER] Cargadas {} líneas útiles (comandos reales)", file_lines.len());

                            // Limpiar flags de control de flujo del comunicador para que un Stop
                            // anterior no impida el envío del job actual.
                            if let Some(mut comm) = communicator.try_lock() {
                                comm.repetier_reset_flow_control();
                            }

                            current_idx = 0;
                            commands_in_flight = 0;
                            local_streaming = true;
                            local_paused = false;
                            *is_streaming.lock() = true;
                            *is_paused.lock() = false;
                        }
                        SenderCommand::Pause => {
                            local_paused = true;
                            *is_paused.lock() = true;
                        }
                        SenderCommand::Resume => {
                            local_paused = false;
                            *is_paused.lock() = false;
                        }
                        SenderCommand::Stop => {
                            local_streaming = false;
                            *is_streaming.lock() = false;
                            let _ = tx_ui_thread.send("Completed".to_string());
                        }
                    }
                }

                    if local_streaming && !local_paused {
                        // 2. COMPROBAR ESTADO DE CONTROL DE FLUJO DEL COMUNICADOR.
                        //    - Si el usuario ha pulsado Stop (0x18), terminamos el bucle.
                        //    - Si el usuario ha pulsado Pause (!), no enviamos nada nuevo.
                        let (flow_paused, flow_stopped) = if let Some(comm) = communicator.try_lock() {
                            (comm.repetier_is_paused(), comm.repetier_is_stopped())
                        } else {
                            (false, false)
                        };

                        if flow_stopped {
                            tracing::debug!("⏹️  [SENDER] Stop detectado. Terminando streaming.");
                            local_streaming = false;
                            *is_streaming.lock() = false;
                            let _ = tx_ui_thread.send("Completed".to_string());
                            // Nota: no salimos del bucle; el hilo sigue vivo por si llega otro job.
                        } else if flow_paused {
                            // En pausa: no enviamos comandos, pero seguimos leyendo del puerto
                            // para no acumular datos en el buffer serie.
                            // El bloque de lectura de eventos sigue ejecutándose más abajo.
                        } else if commands_in_flight < 1 && current_idx < file_lines.len() {
                        let raw_line = file_lines[current_idx].clone();

                        if let Some(mut comm) = communicator.try_lock() {
                            match comm.send(raw_line.as_bytes()) {
                                Ok(written) if written > 0 => {
                                    let _ = tx_ui_thread.send(format!("> {}", raw_line.trim()));
                                    commands_in_flight += 1;
                                    current_idx += 1;
                                    last_retry_idx = None;
                                }
                                Ok(_) => {
                                    // Descartado por el comunicador (comentario, vacío, ruido).
                                    current_idx += 1;
                                }
                                Err(e) => {
                                    if last_retry_idx != Some(current_idx) {
                                        tracing::debug!(
                                            "⏸️  [SENDER] Reintentando idx={} por: {}",
                                            current_idx, e
                                        );
                                        last_retry_idx = Some(current_idx);
                                    }
                                }
                            }
                        }

                        let progress = (current_idx as f64 / file_lines.len() as f64) * 100.0;
                        let _ = tx_ui_thread.send(format!("*{:.1}%", progress));
                    }

                    // 3. ESCUCHA DE CONFIRMACIONES REALES Y MANEJO DE OKs
                    if let Some(mut comm) = communicator.try_lock() {
                        // 1) Vaciamos la cola de eventos del comunicador
                        let events = comm.take_repetier_events();

                        // 2) Los consumimos sin parsear texto.
                        for event in events {
                            match event {
                                RepetierEvent::Ack => {
                                    commands_in_flight = commands_in_flight.saturating_sub(1);
                                    tracing::debug!("✅ [SENDER] Ack. in_flight={}", commands_in_flight);
                                }
                                RepetierEvent::Resend(target_line) => {
                                    tracing::debug!("🔄 [SENDER] Resend N{}. in_flight antes={}, current_idx antes={}",
                                        target_line, commands_in_flight, current_idx);

                                    let _ = tx_ui_thread.send(format!(
                                        "> ⚠️ [REPARACIÓN] Resend detectado para línea N{}",
                                        target_line
                                    ));

                                    // Vaciamos la estimación de vuelo
                                    commands_in_flight = 0;

                                    // Sincronizamos el contador N del comunicador con lo que
                                    // el firmware espera.
                                    comm.repetier_set_line_counter(target_line.saturating_sub(1));

                                    // Rebobinamos el puntero de archivo del sender para que
                                    // vuelva a enviar desde la línea pedida.
                                    if target_line as usize <= file_lines.len() {
                                        current_idx = (target_line as usize).saturating_sub(1);
                                    }
                                }
                            }
                        }

                        // 3) Drenamos el buffer de bytes para no acumularlo.
                        let _ = comm.receive();
                    }

                    if current_idx >= file_lines.len() && commands_in_flight == 0 {
                        local_streaming = false;
                        *is_streaming.lock() = false;
                        let _ = tx_ui_thread.send("Completed".to_string());
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }
        });

        (Self { tx_control }, rx_ui)
    }

    // ====================================================================
    // 🌟 MÉTODOS PÚBLICOS DE CONTROL (Conexión con los botones de la UI)
    // ====================================================================
    pub fn send_gcode(&self, content: &str) {
        let _ = self
            .tx_control
            .send(SenderCommand::SendGcode(content.to_string()));
    }

    pub fn pause(&self) {
        let _ = self.tx_control.send(SenderCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.tx_control.send(SenderCommand::Resume);
    }

    pub fn stop(&self) {
        let _ = self.tx_control.send(SenderCommand::Stop);
    }
}
