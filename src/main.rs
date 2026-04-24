#![windows_subsystem = "windows"]
#![allow(warnings)]
#![allow(future_incompatible)]

use eframe::egui;
use std::time::{Instant, Duration};
use rodio::{OutputStream, Sink, source::Source};
use std::sync::{Arc, Mutex};

struct AudioState {
    sink: Sink,
    _stream: OutputStream,
}

fn main() -> eframe::Result {
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
        "bestrigyn calculator V1.1",
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
    expression: String,
    start_time: Instant,
    audio: Option<Arc<Mutex<AudioState>>>,
    is_result: bool,
}

const CPUNK_GREEN: egui::Color32 = egui::Color32::from_rgb(0, 255, 60);

impl MyCalc {
    fn new(audio: Option<Arc<Mutex<AudioState>>>) -> Self {
        Self {
            expression: "0".to_string(),
            start_time: Instant::now(),
            audio,
            is_result: false,
        }
    }

    fn play_click(&self) {
        if let Some(audio_arc) = &self.audio {
            if let Ok(state) = audio_arc.lock() {
                let source = rodio::source::SineWave::new(1000.0)
                    .take_duration(Duration::from_millis(40)) 
                    .amplify(0.10);
                state.sink.append(source);
            }
        }
    }

    fn input(&mut self, s: &str) {
        let trimmed = s.trim();
        let is_op = trimmed == "+" || trimmed == "-" || trimmed == "*" || trimmed == "/";
        let last_char_is_op = self.expression.trim().ends_with(|c| c == '+' || c == '-' || c == '*' || c == '/');

        if is_op && last_char_is_op { return; }

        self.play_click();
        if self.expression == "0" || self.is_result {
            self.expression = trimmed.to_string();
            self.is_result = false;
        } else {
            if is_op {
                self.expression.push_str(&format!(" {} ", trimmed));
            } else {
                self.expression.push_str(trimmed);
            }
        }
    }

    // Удаление одного символа
    fn backspace(&mut self) {
        if self.is_result {
            self.expression = "0".to_string();
            self.is_result = false;
            return;
        }
        self.play_click();
        self.expression.pop();
        if self.expression.is_empty() || self.expression == " " {
            self.expression = "0".to_string();
        }
        // Убираем лишний пробел, если удалили знак
        self.expression = self.expression.trim_end().to_string();
    }

    fn clear(&mut self) {
        self.play_click();
        self.expression = "0".to_string();
        self.is_result = false;
    }

    fn eval_expression(&mut self) {
        self.play_click();
        let to_eval = self.expression.replace(' ', "").replace(',', ".");
        match meval::eval_str(&to_eval) {
            Ok(res) => {
                let res_str = if res.fract() == 0.0 { format!("{:.0}", res) } else { format!("{:.2}", res) };
                self.expression = res_str;
                self.is_result = true;
            }
            Err(_) => {
                self.expression = "ERROR".to_string();
                self.is_result = true;
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Читаем ввод текста (для цифр, точек, запятых и знаков на нумпаде)
            for char in &i.events {
                if let egui::Event::Text(t) = char {
                    if "0123456789+-*/".contains(t) {
                        self.input(t);
                    } else if t == "," || t == "." {
                        self.input("."); // Превращаем запятую в точку
                    }
                }
                
                // Читаем спец-клавиши (Enter, Backspace, Esc)
                if let egui::Event::Key { key, pressed: true, .. } = char {
                    match key {
                        egui::Key::Enter => self.eval_expression(),
                        egui::Key::Backspace => self.backspace(),
                        egui::Key::Escape => self.clear(),
                        _ => {}
                    }
                }
            }
        });
    }

    fn button_v1(&mut self, ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
        ui.add_sized([70.0, 70.0], 
            egui::Button::new(egui::RichText::new(text).size(28.0).color(color))
                .fill(egui::Color32::from_rgb(40, 40, 40)))
    }
}

impl eframe::App for MyCalc {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keys(ctx);
        ctx.request_repaint_after(Duration::from_millis(32)); 
        let elapsed_ms = self.start_time.elapsed().as_millis() as u64;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(5.0);
            ui.label(egui::RichText::new("STATION ONLINE").color(CPUNK_GREEN).size(10.0).monospace());
            
            egui::Frame::canvas(ui.style())
                .fill(egui::Color32::BLACK).stroke(egui::Stroke::new(2.0, CPUNK_GREEN))
                .show(ui, |ui| {
                    ui.set_min_height(80.0);
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(15.0);
                        let glitch_tick = elapsed_ms / 100; 
                        let offset_x = if glitch_tick % 9 == 0 { (elapsed_ms % 4) as f32 - 2.0 } else { 0.0 };
                        let painter = ui.painter();
                        let text_pos = ui.next_widget_position() + egui::vec2(offset_x, 0.0);
                        let font_size = if self.expression.len() > 18 { 16.0 } else if self.expression.len() > 12 { 24.0 } else { 38.0 };
                        painter.text(text_pos, egui::Align2::CENTER_TOP, &self.expression, egui::FontId::monospace(font_size), CPUNK_GREEN);
                    });
                });

            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                egui::Grid::new("btns").spacing([10.0, 10.0]).show(ui, |ui| {
                    if self.button_v1(ui, "7", CPUNK_GREEN).clicked() { self.input("7"); }
                    if self.button_v1(ui, "8", CPUNK_GREEN).clicked() { self.input("8"); }
                    if self.button_v1(ui, "9", CPUNK_GREEN).clicked() { self.input("9"); }
                    if self.button_v1(ui, "/", egui::Color32::WHITE).clicked() { self.input("/"); }
                    ui.end_row();

                    if self.button_v1(ui, "4", CPUNK_GREEN).clicked() { self.input("4"); }
                    if self.button_v1(ui, "5", CPUNK_GREEN).clicked() { self.input("5"); }
                    if self.button_v1(ui, "6", CPUNK_GREEN).clicked() { self.input("6"); }
                    if self.button_v1(ui, "*", egui::Color32::WHITE).clicked() { self.input("*"); }
                    ui.end_row();

                    if self.button_v1(ui, "1", CPUNK_GREEN).clicked() { self.input("1"); }
                    if self.button_v1(ui, "2", CPUNK_GREEN).clicked() { self.input("2"); }
                    if self.button_v1(ui, "3", CPUNK_GREEN).clicked() { self.input("3"); }
                    if self.button_v1(ui, "-", egui::Color32::WHITE).clicked() { self.input("-"); }
                    ui.end_row();

                    if self.button_v1(ui, "C", egui::Color32::RED).clicked() { self.clear(); }
                    if self.button_v1(ui, "0", CPUNK_GREEN).clicked() { self.input("0"); }
                    if ui.add_sized([70.0, 70.0], egui::Button::new(egui::RichText::new("=").size(28.0).color(egui::Color32::BLACK)).fill(CPUNK_GREEN)).clicked() { self.eval_expression(); }
                    if self.button_v1(ui, "+", egui::Color32::WHITE).clicked() { self.input("+"); }
                    ui.end_row();
                });
            });
            
            // Кнопка запятой (точки) в интерфейсе
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.add_sized([70.0, 30.0], egui::Button::new(".")).clicked() { self.input("."); }
            });
        });
    }
}