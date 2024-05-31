use std::sync::{Arc, Mutex};

use eframe::{
    egui::{self, Id},
    egui_glow,
    epaint::Pos2,
    glow::HasContext,
};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native("DrawBox", options, Box::new(|cc| Box::new(MyApp::new(cc))));

    Ok(())
}

struct Square {
    pos: Pos2,
    size: Pos2,
    color: [f32; 3],
    radius: f32,
}

impl Default for Square {
    fn default() -> Self {
        Self {
            pos: Pos2 { x: 50.0, y: 50.0 },
            size: Pos2 { x: 100.0, y: 100.0 },
            color: [1.0, 1.0, 1.0],
            radius: 0.2,
        }
    }
}

struct Circle {
    pos: Pos2,
    radius: f32,
    color: [f32; 3],
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            pos: Pos2 { x: 0.0, y: 0.0 },
            radius: 100.0,
            color: [1.0, 1.0, 1.0],
        }
    }
}

enum Shape {
    Square(Square, f32, f32, bool),
    Circle(Circle, f32, f32, bool),
}

impl Shape {
    fn default_square() -> Self {
        Self::Square(Square::default(), 0.5, 0.5, false)
    }

    fn default_circle() -> Self {
        Self::Circle(Circle::default(), 0.0, 0.0, false)
    }
}

struct MyApp {
    shapes: Arc<Mutex<Vec<Shape>>>,
    selected_index: Option<usize>,
    renderer: Arc<Mutex<PlaneRenderer>>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

struct PlaneRenderer {
    program: egui_glow::glow::Program,
    vertex_array: egui_glow::glow::VertexArray,
}

impl PlaneRenderer {
    fn new(
        gl: &egui_glow::glow::Context,
        shapes: Arc<Mutex<Vec<Shape>>>,
        min_dim: f32,
        canvas_pos: Pos2,
    ) -> PlaneRenderer {
        use egui_glow::glow::HasContext as _;
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        let mut fragment = String::new();
        for (idx, shape) in shapes.lock().unwrap().iter().enumerate() {
            match shape {
                &Shape::Circle(Circle { pos, radius, color }, blob, color_spread, subtract) => {
                    todo!()
                }
                &Shape::Square(
                    Square {
                        pos,
                        size,
                        color,
                        radius,
                    },
                    blob,
                    color_spread,
                    subtract,
                ) => {
                    if idx != 0 {
                        fragment.push_str(&format!(
                            "
                         d2 = rectangle(p, vec2({:?}, {:?}), vec2({:?}, {:?}), {:?});
                         d = smin(d, {}*d2, {:?});
                         w = max(1.0-(10-10*{:?})*d2, 0.0001);
                         dcol += vec4({:?}, {:?}, {:?}, 1.0)*w; 
                        ",
                            pos.x,
                            pos.y,
                            size.x / 2.,
                            size.y / 2.,
                            radius,
                            {
                                if subtract {
                                    "-1.0"
                                } else {
                                    "1.0"
                                }
                            },
                            blob,
                            color_spread,
                            color[0],
                            color[1],
                            color[2],
                        ))
                    } else {
                        fragment.push_str(&format!(
                            "
                             d = rectangle(p, vec2({:?}, {:?}), vec2({:?}, {:?}), {:?});
                             w = max(1.0-(10-10*{:?})*d, 0.0001);
                             dcol += vec4({:?},{:?},{:?},1.0)*w;
                            ",
                            pos.x,
                            pos.y,
                            size.x / 2.0,
                            size.y / 2.0,
                            radius,
                            color_spread,
                            color[0],
                            color[1],
                            color[2],
                        ))
                    }
                }
            }
        }

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vs_source, fs_source) = (
                r#"
                    const vec2 verts[6] =  vec2[6](
                        vec2(-1.0,-1.0),
                        vec2(1.0, -1.0),
                        vec2(1.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(-1.0,1.0),
                        vec2(1.0, 1.0)
                );

                void main() {
                    gl_Position = vec4(verts[gl_VertexID],0.0,1.0);
                }
                "#,
                &format!(
                    r#"
                    precision mediump float;
                    out vec4 out_color;

                    layout(origin_upper_left) in vec4 gl_FragCoord;

                    float smin(float a, float b, float k) {{
    float h = clamp(0.5 + 0.5*(a-b)/k, 0.0, 1.0);
    return mix(a, b, h) - k*h*(1.0-h);
}}

                    float rectangle(vec2 samplePosition, vec2 position, vec2 halfSize, float radius){{
    vec2 componentWiseEdgeDistance = abs(samplePosition - position / {min_dim:?}) - halfSize/{min_dim:?} + vec2(radius,radius)*min(halfSize.x, halfSize.y)/{min_dim:?};
    float outsideDistance = length(max(componentWiseEdgeDistance, 0));
    float insideDistance = min(max(componentWiseEdgeDistance.x, componentWiseEdgeDistance.y), 0);
    return outsideDistance + insideDistance - radius*min(halfSize.x,halfSize.y)/{min_dim:?};
}}


                    void main() {{

                        vec2 p = vec2(gl_FragCoord.x - {:?}, gl_FragCoord.y -{:?}) / {min_dim:?};

                        vec3 col = vec3(0.0,0.0,0.0);
                        vec4 dcol = vec4(0.0,0.0,0.0,0.0);
                        float d2;
                        float w;
                        float d = 1;
                        {}

                        dcol.xyz /= dcol.w;

    
                        col = mix( col, dcol.xyz, 1.0-smoothstep(0.0,0.01,d*10.0) ); 
                        
                        out_color = vec4(col, 1.0);
                     
                    }}
                "#,
                    canvas_pos.x, canvas_pos.y, fragment,
                ),
            );

            let shader_sources = [
                (egui_glow::glow::VERTEX_SHADER, vs_source),
                (egui_glow::glow::FRAGMENT_SHADER, fs_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Couldn't compile shader: {}",
                        gl.get_shader_info_log(shader),
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);

            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Couldn't create vertex array");

            Self {
                program,
                vertex_array,
            }
        }
    }

    fn destroy(&self, gl: &egui_glow::glow::Context) {
        use egui_glow::glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(
        &mut self,
        gl: &egui_glow::glow::Context,
        shapes: Arc<Mutex<Vec<Shape>>>,
        selected_idx: Option<usize>,
        min_dim: f32,
        canvas_pos: Pos2,
    ) {
        self.destroy(gl);
        *self = Self::new(gl, shapes, min_dim, canvas_pos);
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(egui_glow::glow::TRIANGLES, 0, 6);
        }
    }
}
