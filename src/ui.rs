use eframe::{
    egui::{self, Id},
    egui_glow,
    epaint::Pos2,
};

use crate::{renderer::PlaneRenderer, shapes::*};
use std::sync::{Arc, Mutex};

pub struct MyApp {
    pub shapes: Arc<Mutex<Vec<Shape>>>,
    pub selected_index: Option<usize>,
    pub renderer: Arc<Mutex<PlaneRenderer>>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.as_ref().expect("Use glow backend");
        Self {
            shapes: Arc::new(Mutex::new(Vec::new())),
            selected_index: None,
            renderer: Arc::new(Mutex::new(PlaneRenderer::new(
                gl,
                Arc::new(Mutex::new(Vec::new())),
                0.0,
                Pos2 { x: 0.0, y: 0.0 },
            ))),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left(Id::new("Settings")).show(ctx, |ui| {
            if ui.button("Add Square").clicked() {
                self.shapes.lock().unwrap().push(Shape::default_square());
            }
            if ui.button("Add Circle").clicked() {
                self.shapes.lock().unwrap().push(Shape::default_circle());
            }
            if let Some(idx) = self.selected_index {
                let mut shapes = self.shapes.lock().unwrap();
                let shape = shapes.get_mut(idx).unwrap();
                match shape {
                    &mut Shape::Square(
                        ref mut square,
                        ref mut blob,
                        ref mut color_spread,
                        ref mut subtract,
                    ) => {
                        ui.add(egui::Slider::new(&mut square.pos.x, 0_f32..=1000_f32).text("x"));
                        ui.add(egui::Slider::new(&mut square.pos.y, 0_f32..=1000_f32).text("y"));
                        ui.add(
                            egui::Slider::new(&mut square.size.x, 0_f32..=1000_f32).text("width"),
                        );
                        ui.add(
                            egui::Slider::new(&mut square.size.y, 0_f32..=1000_f32).text("height"),
                        );
                        ui.add(egui::Slider::new(&mut square.radius, 0_f32..=1_f32).text("radius"));
                        ui.add(egui::Slider::new(blob, -10_f32..=10_f32).text("blob"));
                        ui.add(egui::Slider::new(color_spread, 0_f32..=1_f32).text("color spread"));

                        egui::color_picker::color_edit_button_rgb(ui, &mut square.color);

                        ui.checkbox(subtract, "Subtract");

                        if ui.button("delete").clicked() {
                            shapes.remove(idx);
                            self.selected_index = None;
                        }
                    }
                    _ => (),
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        if let Some(gl) = gl {
            self.renderer.lock().unwrap().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(800.0), egui::Sense::click_and_drag());

        let selected_idx = self.selected_index.clone();
        let shapes = self.shapes.clone();
        let renderer = self.renderer.clone();
        let width = rect.width();
        let min_dimension = width.min(rect.height());
        let canvas_position = rect.min;

        if response.clicked() {
            let click_pos = response.interact_pointer_pos().unwrap() - canvas_position;
            let click_pos = Pos2::new(click_pos.x, click_pos.y);
            println!("{click_pos:?}");
            for (idx, shape) in shapes.lock().unwrap().iter().enumerate() {
                match shape {
                    &Shape::Square(
                        Square {
                            pos,
                            size,
                            color: _,
                            radius,
                        },
                        _,
                        _,
                        _,
                    ) => {
                        let d = click_pos - pos;

                        if d.x.abs() < size.x / 2.0
                            && d.y.abs() < size.y / 2.0
                            && selected_idx != Some(idx)
                        {
                            println!("{d:?}, {size:?}");
                            self.selected_index = Some(idx);
                        }
                    }
                    _ => (),
                }
            }
        }
        let selected_idx = self.selected_index.clone();

        if let Some(idx) = selected_idx {
            if let Some(shape) = self.shapes.lock().unwrap().get_mut(idx) {
                match shape {
                    &mut Shape::Square(ref mut square, _, _, _) => {
                        square.pos += response.drag_delta();
                    }
                    _ => (),
                }
            }
        }

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                renderer.lock().unwrap().paint(
                    painter.gl(),
                    shapes.clone(),
                    selected_idx,
                    min_dimension,
                    canvas_position,
                );
            })),
        };

        ui.painter().add(callback);
    }
}
