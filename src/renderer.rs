use std::sync::{Arc, Mutex};

use eframe::{egui_glow, epaint::Pos2};

use crate::shapes::*;

pub struct PlaneRenderer {
    pub program: egui_glow::glow::Program,
    pub vertex_array: egui_glow::glow::VertexArray,
}

impl PlaneRenderer {
    pub fn new(
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

    pub fn destroy(&self, gl: &egui_glow::glow::Context) {
        use egui_glow::glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    pub fn paint(
        &mut self,
        gl: &egui_glow::glow::Context,
        shapes: Arc<Mutex<Vec<Shape>>>,
        selected_idx: Option<usize>,
        min_dim: f32,
        canvas_pos: Pos2,
    ) {
        use egui_glow::glow::HasContext as _;
        self.destroy(gl);
        *self = Self::new(gl, shapes, min_dim, canvas_pos);
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(egui_glow::glow::TRIANGLES, 0, 6);
        }
    }
}
