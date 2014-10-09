use cgmath::FixedArray;
use cgmath::{Matrix4, Point3, Vector3};
use cgmath::{Transform, AffineMatrix3};
use gfx;
use glfw;
use gfx::{DeviceHelper, ToSlice};
use glfw::Context;

#[shader_param(DebugBox)]
struct Params {
    #[name = "u_MVP"]
    mvp: [[f32, ..4], ..4],
}

#[vertex_format]
struct Vertex {
    #[name = "a_Pos"]
    pos: [f32, ..2],

    #[name = "a_Color"]
    color: [f32, ..3],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    
    uniform mat4 u_MVP;

    in vec2 a_Pos;
    in vec3 a_Color;
    out vec4 v_Color;

    void main() {
        v_Color = vec4(a_Color, 1.0);
        gl_Position = u_MVP * vec4(a_Pos, -10.0, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core

    in vec4 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = v_Color;
    }
"
};
// FIXME: There's a memory leak somewhere in here...
pub fn render(window: &glfw::Window, frame: &gfx::Frame, graphics: &mut gfx::Graphics<gfx::GlDevice, gfx::GlCommandBuffer>) { 
    let vertex_data = [
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
    ];

    let mesh = graphics.device.create_mesh(vertex_data);
    let slice = mesh.to_slice(gfx::TriangleList);

    let program = graphics.device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
                        .unwrap();

    let data = Params { mvp: ::cgmath::perspective(::cgmath::deg(60.0f32), 640.0/480.0,
                                  0.1, 1000.0).into_fixed() };

    let batch: DebugBox = graphics.make_batch(
        &program, &mesh, slice, &gfx::DrawState::new()).unwrap();

    let clear_data = gfx::ClearData {
        color: [0.3, 0.3, 0.3, 1.0],
        depth: 1.0,
        stencil: 0,
    };

    graphics.clear(clear_data, gfx::Color, frame);
    graphics.draw(&batch, &data, frame);
    graphics.end_frame();

    window.swap_buffers();
}
