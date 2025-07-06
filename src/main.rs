use crossbeam_channel::{unbounded, Receiver};
use eframe::egui;
use midir::{MidiInput, Ignore};
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, Duration};

// I'm sorry.

fn main() -> eframe::Result<()> {
    let (tx, rx) = unbounded();
    start_midi_listener(tx);

    let shared_state = Arc::new(Mutex::new(GameState::default()));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Notentrainer",
        options,
        Box::new(|_cc| Box::new(NotentrainerApp::new(rx, shared_state))),
    )
}

fn start_midi_listener(tx: crossbeam_channel::Sender<u8>) {
    thread::spawn(move || {
        let mut midi_in = MidiInput::new("Pitchplay").unwrap();
        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        if in_ports.is_empty() {
            println!("No MIDI keyboard found.");
            return;
        }

        let in_port = &in_ports[0];
        println!("Connected to {}", midi_in.port_name(in_port).unwrap());

        let _conn = midi_in.connect(
            in_port,
            "pitchplay-listener",
            move |_stamp, message, _| {
                if let Some(&status) = message.first() {
                    if status & 0xF0 == 0x90 && message.len() > 1 {
                        let note = message[1];
                        tx.send(note).ok();
                    }
                }
            },
            (),
        ).unwrap();

        loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
    });
}

struct NotentrainerApp {
    rx: Receiver<u8>,
    state: Arc<Mutex<GameState>>,
    note_texture_up: Option<egui::TextureHandle>,
    note_texture_down: Option<egui::TextureHandle>,
    clef_texture: Option<egui::TextureHandle>,
}

impl NotentrainerApp {
    fn new(rx: Receiver<u8>, state: Arc<Mutex<GameState>>) -> Self {
        Self {
            rx,
            state,
            note_texture_up: None,
            note_texture_down: None,
            clef_texture: None
        }
    }

    fn load_textures(&mut self, ctx: &egui::Context) {
        let img1 = include_bytes!("../assets/quarter_up.png");
        let image1 = image::load_from_memory(img1).unwrap().to_rgba8();
        let size1 = [image1.width() as usize, image1.height() as usize];
        let pixels1 = image1.as_flat_samples();
        let color_image1 = egui::ColorImage::from_rgba_unmultiplied(size1, pixels1.as_slice());
        self.note_texture_up = Some(ctx.load_texture(
            "quarter_up",
            color_image1,
            egui::TextureOptions::LINEAR,
        ));

        let img2 = include_bytes!("../assets/quarter_down.png");
        let image2 = image::load_from_memory(img2).unwrap().to_rgba8();
        let size2 = [image2.width() as usize, image2.height() as usize];
        let pixels2 = image2.as_flat_samples();
        let color_image2 = egui::ColorImage::from_rgba_unmultiplied(size2, pixels2.as_slice());
        self.note_texture_down = Some(ctx.load_texture(
            "quarter_down",
            color_image2,
            egui::TextureOptions::LINEAR,
        ));

        let img3 = include_bytes!("../assets/clef.png");
        let image3 = image::load_from_memory(img3).unwrap().to_rgba8();
        let size3 = [image3.width() as usize, image3.height() as usize];
        let pixels3 = image3.as_flat_samples();
        let color_image3 = egui::ColorImage::from_rgba_unmultiplied(size3, pixels3.as_slice());
        self.clef_texture = Some(ctx.load_texture(
            "clef",
            color_image3,
            egui::TextureOptions::LINEAR,
        ));
    }
}

impl eframe::App for NotentrainerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.note_texture_up.is_none() || self.note_texture_down.is_none() || self.clef_texture.is_none() {
            self.load_textures(ctx);
        }

        while let Ok(note) = self.rx.try_recv() {
            let mut state = self.state.lock().unwrap();
            println!("Eingehend: MIDI-Note {}", note);

            if note == state.expected_midi_note() {
                state.result = Some(true);
                state.correct_answer_time = Some(Instant::now());
            } else {
                state.result = Some(false);
            }
        }

        {
            let mut state = self.state.lock().unwrap();
            if let Some(t) = state.correct_answer_time {
                if t.elapsed() >= Duration::from_millis(500) {
                    state.generate_new_note();
                    state.correct_answer_time = None;
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.state.lock().unwrap().generate_new_note();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            {
                let state = self.state.lock().unwrap();

                ui.heading("🎵 Pitchplay");
                ui.label("Play note on attached MIDI keyboard");
                if let Some(result) = state.result {
                    if result {
                        ui.colored_label(egui::Color32::GREEN, "✅ Correct!");
                    } else {
                        ui.colored_label(egui::Color32::RED, "❌ Wrong!");
                    }
                }
            }

            if ui.button("SPACE to skip").clicked() {
                self.state.lock().unwrap().generate_new_note();
            }

            ui.group(|ui| {
                let desired_size = egui::Vec2::new(400.0, 150.0);
                let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
                let rect = response.rect;

                let line_spacing = 15.0;
                let top = rect.top() + 20.0;
                let left = rect.left() + 20.0;
                let right = rect.right() - 20.0;

                // 5 Notenlinien
                for i in 0..5 {
                    let y = top + i as f32 * line_spacing;
                    painter.line_segment(
                        [egui::pos2(left, y), egui::pos2(right, y)],
                        egui::Stroke::new(1.5, egui::Color32::BLACK),
                    );
                }

                // Key
                if let Some(tex_key) = &self.clef_texture {
                    let dest_rect = egui::Rect::from_center_size(
                        egui::pos2(
                            left + 5.0,
                            note_to_y("G4", top, line_spacing)
                        ),
                        tex_key.size_vec2(),
                    );
                    painter.image(
                        tex_key.id(),
                        dest_rect,
                        egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0),
                        ),
                        egui::Color32::WHITE,
                    );
                }

                let state = self.state.lock().unwrap();
                let note_y = note_to_y(&state.current_note, top, line_spacing);
                let note_x = (left + right) / 2.0;

                // Hilfslinie für C4
                if needs_ledger_line(&state.current_note) {
                    painter.line_segment(
                        [
                            egui::pos2(note_x - 15.0, note_y),
                            egui::pos2(note_x + 15.0, note_y),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                }

                // Notenkopf als Bild zeichnen
                if let (Some(tex_up), Some(tex_down)) = (&self.note_texture_up, &self.note_texture_down) {
                    let y_midline = top + line_spacing * 2.0;
                    let texture = if note_y <= y_midline {
                        tex_down
                    } else {
                        tex_up
                    };

                    let tex_size = texture.size_vec2();
                    let dest_rect = egui::Rect::from_center_size(
                        egui::pos2(note_x, note_y),
                        tex_size,
                    );
                    painter.image(
                        texture.id(),
                        dest_rect,
                        egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0),
                        ),
                        egui::Color32::WHITE,
                    );
                }
            });
        });

        ctx.request_repaint();
    }
}

struct GameState {
    current_note: &'static str,
    result: Option<bool>,
    last_note_change: Instant,
    correct_answer_time: Option<Instant>,
}

impl GameState {

    /// Returns the MIDI note number corresponding to the current note.
    fn expected_midi_note(&self) -> u8 {
        note_to_midi(self.current_note)
    }

    /// Generates a new random note from C4 to H4, resets result and timer state.
    fn generate_new_note(&mut self) {
        let notes = ["C4", "D4", "E4", "F4", "G4", "A4", "H4"];
        self.current_note = notes.choose(&mut rand::thread_rng()).unwrap();
        self.result = None;
        self.last_note_change = Instant::now();
        self.correct_answer_time = None;
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut gs = GameState {
            current_note: "C4",
            result: None,
            last_note_change: Instant::now(),
            correct_answer_time: None,
        };
        gs.generate_new_note();
        gs
    }
}

/// Converts a note name like "C4" or "H4" to the corresponding MIDI note number.
/// Returns 0 for unrecognized input.
fn note_to_midi(note: &str) -> u8 {
    match note {
        "C4" => 60, "D4" => 62, "E4" => 64, "F4" => 65,
        "G4" => 67, "A4" => 69, "H4" => 71,
        _ => 0,
    }
}

/// Calculates the vertical y-position of the given note on the staff,
/// based on the top position of the staff and the spacing between lines.
fn note_to_y(note: &str, top: f32, line_spacing: f32) -> f32 {
    let ref_steps = note_to_steps("E4");
    let note_steps = note_to_steps(note) - ref_steps;

    let y_e4 = top + line_spacing * 4.0;
    y_e4 - note_steps as f32 * (line_spacing / 2.0)
}

/// Converts a note name like "C4" or "H4" to its absolute note step value.
/// Each octave advances by 7 steps (for 7 natural notes).
fn note_to_steps(note: &str) -> i32 {
    let letter_idx = match &note[..1] {
        "C" => 0, "D" => 1, "E" => 2, "F" => 3,
        "G" => 4, "A" => 5, "H" => 6,
        _ => 0,
    };
    let octave: i32 = note[1..].parse().unwrap();
    octave * 7 + letter_idx
}

/// Determines if a given note requires a ledger line below the staff.
/// Currently only returns true for C4.
fn needs_ledger_line(note: &str) -> bool {
    note == "C4"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_midi() {
        assert_eq!(note_to_midi("C4"), 60);
        assert_eq!(note_to_midi("D4"), 62);
        assert_eq!(note_to_midi("E4"), 64);
        assert_eq!(note_to_midi("F4"), 65);
        assert_eq!(note_to_midi("G4"), 67);
        assert_eq!(note_to_midi("A4"), 69);
        assert_eq!(note_to_midi("H4"), 71);
        assert_eq!(note_to_midi("Invalid"), 0); // test fallback
    }

    #[test]
    fn test_note_to_steps() {
        assert_eq!(note_to_steps("C4"), 28);
        assert_eq!(note_to_steps("D4"), 29);
        assert_eq!(note_to_steps("E4"), 30);
        assert_eq!(note_to_steps("F4"), 31);
        assert_eq!(note_to_steps("G4"), 32);
        assert_eq!(note_to_steps("A4"), 33);
        assert_eq!(note_to_steps("H4"), 34);
    }

    #[test]
    fn test_note_to_y_position() {
        let top = 0.0;
        let spacing = 20.0;

        let y_e4 = note_to_y("E4", top, spacing);
        let y_g4 = note_to_y("G4", top, spacing);
        let y_c4 = note_to_y("C4", top, spacing);

        // E4 is reference → should be at line 1
        assert!((y_e4 - (top + spacing * 4.0)).abs() < 1e-6);

        // G4 is higher → should have smaller y (above E4)
        assert!(y_g4 < y_e4);

        // C4 is lower → should have bigger y (below E4)
        assert!(y_c4 > y_e4);
    }

    #[test]
    fn test_needs_ledger_line() {
        assert!(needs_ledger_line("C4"));
        assert!(!needs_ledger_line("D4"));
        assert!(!needs_ledger_line("E4"));
    }
}
