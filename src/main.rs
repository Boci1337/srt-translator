#![windows_subsystem = "windows"]

use eframe::egui;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 260.0])
            .with_resizable(false)
            .with_title("SRT Translator"),
        ..Default::default()
    };
    eframe::run_native(
        "SRT Translator",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

// ── Shared state between UI and worker thread ─────────────────────────────────

#[derive(Default)]
struct Shared {
    progress: f32,
    status: String,
    done: bool,
    error: Option<String>,
}

// ── Main app state ────────────────────────────────────────────────────────────

struct App {
    input_path: String,
    output_path: String,
    running: bool,
    shared: Arc<Mutex<Shared>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::new(),
            running: false,
            shared: Arc::new(Mutex::new(Shared {
                status: "Select input and output files to begin.".to_string(),
                ..Default::default()
            })),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            ctx.request_repaint();
            let done = self.shared.lock().unwrap().done;
            if done {
                self.running = false;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("\u{1F3AC} SRT Translator  \u{2014}  English \u{2192} Hungarian");
            ui.separator();
            ui.add_space(4.0);

            // ── Input file row ─────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Input SRT: ");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_path)
                        .desired_width(420.0)
                        .hint_text("path/to/subtitle.srt"),
                );
                if resp.changed() {
                    self.maybe_suggest_output();
                }
                if ui.button("Browse\u{2026}").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("SRT subtitles", &["srt"])
                        .pick_file()
                    {
                        self.input_path = path.to_string_lossy().to_string();
                        self.maybe_suggest_output();
                    }
                }
            });

            ui.add_space(4.0);

            // ── Output file row ────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Output SRT:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.output_path)
                        .desired_width(420.0)
                        .hint_text("path/to/subtitle.hu.srt"),
                );
                if ui.button("Browse\u{2026}").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("SRT subtitles", &["srt"])
                        .save_file()
                    {
                        self.output_path = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // ── Progress bar ───────────────────────────────────────────────
            let (progress, status, error) = {
                let s = self.shared.lock().unwrap();
                (s.progress, s.status.clone(), s.error.clone())
            };

            ui.add(
                egui::ProgressBar::new(progress)
                    .show_percentage()
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(4.0);

            if let Some(err) = &error {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 60, 60),
                    format!("\u{274C} {}", err),
                );
            } else {
                ui.label(&status);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Start button ───────────────────────────────────────────────
            let can_start = !self.running
                && !self.input_path.is_empty()
                && !self.output_path.is_empty();

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        can_start,
                        egui::Button::new("\u{25B6}  Start Translation")
                            .min_size(egui::vec2(200.0, 32.0)),
                    )
                    .clicked()
                {
                    self.start_translation();
                }
            });
        });
    }
}

impl App {
    fn maybe_suggest_output(&mut self) {
        if !self.output_path.is_empty() {
            return;
        }
        let p = std::path::Path::new(&self.input_path);
        let stem = p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dir = p
            .parent()
            .map(|d| d.to_string_lossy().to_string() + "/")
            .unwrap_or_default();
        self.output_path = format!("{}{}.hu.srt", dir, stem);
    }

    fn start_translation(&mut self) {
        self.running = true;
        let shared = Arc::clone(&self.shared);
        {
            let mut s = shared.lock().unwrap();
            *s = Shared {
                status: "Starting\u{2026}".to_string(),
                ..Default::default()
            };
        }
        let input = self.input_path.clone();
        let output = self.output_path.clone();
        thread::spawn(move || {
            if let Err(e) = run_translation(&input, &output, &shared) {
                let mut s = shared.lock().unwrap();
                s.error = Some(e.to_string());
                s.done = true;
            }
        });
    }
}

// ── SRT parsing / composing ───────────────────────────────────────────────────

struct Sub {
    index: String,
    timestamp: String,
    text: String,
}

fn parse_srt(raw: &str) -> Vec<Sub> {
    let mut subs = Vec::new();
    for block in raw.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let mut lines = block.lines();
        let index = lines.next().unwrap_or("").to_string();
        let timestamp = lines.next().unwrap_or("").to_string();
        if !timestamp.contains("-->") {
            continue;
        }
        let text = lines.collect::<Vec<_>>().join("\n");
        subs.push(Sub { index, timestamp, text });
    }
    subs
}

fn compose_srt(subs: &[Sub]) -> String {
    subs.iter()
        .map(|s| format!("{}", format_args!("{0}\n{1}\n{2}\n\n", s.index, s.timestamp, s.text)))
        .collect()
}

// ── Translation (Google Translate free endpoint) ──────────────────────────────

const SEP: &str = " ||| ";

fn translate_batch(texts: &[&str]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let joined = texts.join(SEP);
    let translated = call_google(&joined)?;

    let parts: Vec<String> = translated
        .split(SEP)
        .map(|s| s.trim().to_string())
        .collect();

    if parts.len() == texts.len() {
        return Ok(parts);
    }

    // Fallback: translate individually
    let mut results = Vec::with_capacity(texts.len());
    for text in texts {
        results.push(call_google(text)?);
    }
    Ok(results)
}

fn call_google(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = ureq::get("https://translate.googleapis.com/translate_a/single")
        .query("client", "gtx")
        .query("sl", "en")
        .query("tl", "hu")
        .query("dt", "t")
        .query("q", text)
        .call()?
        .into_string()?;

    let json: serde_json::Value = serde_json::from_str(&resp)?;
    let mut result = String::new();
    if let Some(arr) = json[0].as_array() {
        for segment in arr {
            if let Some(t) = segment[0].as_str() {
                result.push_str(t);
            }
        }
    }
    Ok(result)
}

// ── Worker (runs on background thread) ────────────────────────────────────────

fn run_translation(
    input: &str,
    output: &str,
    shared: &Arc<Mutex<Shared>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let raw = std::fs::read_to_string(input)
        .map_err(|e| format!("Cannot read input file: {}", e))?;

    let mut subs = parse_srt(&raw);
    let total = subs.len();

    {
        let mut s = shared.lock().unwrap();
        s.status = format!("0 / {} subtitles translated\u{2026}", total);
    }

    let chunk_size: usize = 20;
    let mut done = 0usize;

    for chunk in subs.chunks_mut(chunk_size) {
        let texts: Vec<&str> = chunk.iter().map(|s| s.text.as_str()).collect();

        match translate_batch(&texts) {
            Ok(results) => {
                for (sub, tr) in chunk.iter_mut().zip(results.into_iter()) {
                    if !tr.is_empty() {
                        sub.text = tr;
                    }
                }
            }
            Err(_) => {} // keep originals on network error
        }

        done += chunk.len();
        {
            let mut s = shared.lock().unwrap();
            s.progress = done as f32 / total as f32;
            s.status = format!("{} / {} subtitles translated\u{2026}", done, total);
        }

        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    std::fs::write(output, compose_srt(&subs))
        .map_err(|e| format!("Cannot write output file: {}", e))?;

    {
        let mut s = shared.lock().unwrap();
        s.progress = 1.0;
        s.status = format!("\u{2705} Done! {} subtitles saved.", total);
        s.done = true;
    }

    Ok(())
}
