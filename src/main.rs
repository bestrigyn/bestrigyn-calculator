#![windows_subsystem = "windows"]
// Подключаем только то, что точно есть в стандартных зависимостях
use eframe::egui;
use std::time::{Instant, Duration};
use rodio::{OutputStream, Sink, source::Source};
use std::sync::{Arc, Mutex};

// Хранилище для звука, чтобы он не "засыпал"
struct AudioState {
    sink: Sink,
    _stream: OutputStream, 
}

fn main() -> eframe::Result {
    // Инициализируем аудио при старте - это уберет лаги в 15 секунд
    let audio = match OutputStream::try_default() {
        Ok((stream, handle)) => {
            if let Ok(sink) = Sink::try_new(&handle) {
                Some(Arc::new(Mutex::new(AudioState { sink, _stream: stream })))
            } else { None }
        }
        Err(_) => None,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([340.0, 520.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "bestrigyn calculator 1.0",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(5, 5, 5);
            cc.egui_ctx.set_visuals(visuals);
            Ok(Box::new(MyCalc::new(audio)))
        }),
    )
}

struct MyCalc {
    display: String,
    first_num: f64,
    current_op: Option<char>,
    new_entry: bool,
    start_time: Instant,
    audio: Option<Arc<Mutex<AudioState>>>,
}

const CPUNK_GREEN: egui::Color32 = egui::Color32::from_rgb(0, 255, 60);
const CPUNK_BLACK: egui::Color32 = egui::Color32::from_rgb(10, 10, 10);

impl MyCalc {
    fn new(audio: Option<Arc<Mutex<AudioState>>>) -> Self {
        Self {
            display: "0".to_string(),
            first_num: 0.0,
            current_op: None,
            new_entry: true,
            start_time: Instant::now(),
            audio,
        }
    }

    fn play_click(&self) {
        if let Some(audio_arc) = &self.audio {
            if let Ok(state) = audio_arc.lock() {
                let source = rodio::source::SineWave::new(800.0)
                    .take_duration(Duration::from_millis(50))
                    .amplify(0.15);
                state.sink.append(source);
            }
        }
    }

    fn button_v1(&mut self, ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
        let resp = ui.add_sized(
            [70.0, 70.0], 
            egui::Button::new(egui::RichText::new(text).size(28.0).color(color))
                .fill(egui::Color32::from_rgb(40, 40, 40))
        );
        if resp.clicked() {
            self.play_click();
        }
        resp
    }
}

impl eframe::App for MyCalc {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ЛИМИТЕР FPS: Теперь проц будет отдыхать 16мс между кадрами
        ctx.request_repaint_after(Duration::from_millis(16)); 
        
        let elapsed_us = self.start_time.elapsed().as_micros() as u64;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // Экран с твоим крутым неоновым глитчем
            egui::Frame::canvas(ui.style())
                .fill(CPUNK_BLACK)
                .stroke(egui::Stroke::new(2.0, CPUNK_GREEN))
                .show(ui, |ui| {
                    ui.set_min_height(80.0);
                    ui.set_width(ui.available_width());
                    
                    ui.vertical_centered(|ui| {
                        ui.add_space(15.0);
                        let offset_x = ((elapsed_us % 300) as f32 / 150.0) - 1.0;
                        let glitch_color = if elapsed_us % 300 < 20 { egui::Color32::RED } else { CPUNK_GREEN };

                        let painter = ui.painter();
                        let rect = ui.max_rect();
                        let text_pos = ui.next_widget_position() + egui::vec2(offset_x, 0.0);
                        
                        painter.text(text_pos, egui::Align2::CENTER_TOP, &self.display, egui::FontId::monospace(50.0), glitch_color);

                        // Полоски помех
                        if elapsed_us % 400 < 150 {
                            for i in 0..3 {
                                let y = rect.top() + (elapsed_us.wrapping_mul(i + 2) % (rect.height() as u64)) as f32;
                                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(rect.left(), y), egui::pos2(rect.right(), y + 2.0)), 0.0, egui::Color32::from_black_alpha(150));
                            }
                        }
                    });
                });

            ui.add_space(20.0);

            // Сетка кнопок
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                egui::Grid::new("btns").spacing([10.0, 10.0]).show(ui, |ui| {
                    if self.button_v1(ui, "7", CPUNK_GREEN).clicked() { self.add_digit("7"); }
                    if self.button_v1(ui, "8", CPUNK_GREEN).clicked() { self.add_digit("8"); }
                    if self.button_v1(ui, "9", CPUNK_GREEN).clicked() { self.add_digit("9"); }
                    if self.button_v1(ui, "/", egui::Color32::WHITE).clicked() { self.set_op('/'); }
                    ui.end_row();

                    if self.button_v1(ui, "4", CPUNK_GREEN).clicked() { self.add_digit("4"); }
                    if self.button_v1(ui, "5", CPUNK_GREEN).clicked() { self.add_digit("5"); }
                    if self.button_v1(ui, "6", CPUNK_GREEN).clicked() { self.add_digit("6"); }
                    if self.button_v1(ui, "*", egui::Color32::WHITE).clicked() { self.set_op('*'); }
                    ui.end_row();

                    if self.button_v1(ui, "1", CPUNK_GREEN).clicked() { self.add_digit("1"); }
                    if self.button_v1(ui, "2", CPUNK_GREEN).clicked() { self.add_digit("2"); }
                    if self.button_v1(ui, "3", CPUNK_GREEN).clicked() { self.add_digit("3"); }
                    if self.button_v1(ui, "-", egui::Color32::WHITE).clicked() { self.set_op('-'); }
                    ui.end_row();

                    if self.button_v1(ui, "C", egui::Color32::RED).clicked() { self.clear(); }
                    if self.button_v1(ui, "0", CPUNK_GREEN).clicked() { self.add_digit("0"); }
                    
                    let eq_btn = ui.add_sized([70.0, 70.0], egui::Button::new(egui::RichText::new("=").size(28.0).color(CPUNK_BLACK)).fill(CPUNK_GREEN));
                    if eq_btn.clicked() { self.play_click(); self.calc_result(); }

                    if self.button_v1(ui, "+", egui::Color32::WHITE).clicked() { self.set_op('+'); }
                    ui.end_row();
                });
            });
        });
    }
}

impl MyCalc {
    fn add_digit(&mut self, digit: &str) {
        if self.new_entry { self.display = digit.to_string(); self.new_entry = false; }
        else { if self.display == "0" { self.display = digit.to_string(); } else { self.display.push_str(digit); } }
    }
    fn set_op(&mut self, op: char) { self.first_num = self.display.parse().unwrap_or(0.0); self.current_op = Some(op); self.new_entry = true; }
    fn calc_result(&mut self) {
        if let Some(op) = self.current_op {
            let second_num: f64 = self.display.parse().unwrap_or(0.0);
            let res = match op { '+' => self.first_num + second_num, '-' => self.first_num - second_num, '*' => self.first_num * second_num, '/' => if second_num != 0.0 { self.first_num / second_num } else { 0.0 }, _ => 0.0, };
            self.display = if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) };
            self.current_op = None; self.new_entry = true;
        }
    }
    fn clear(&mut self) { self.display = "0".to_string(); self.first_num = 0.0; self.current_op = None; self.new_entry = true; }
}