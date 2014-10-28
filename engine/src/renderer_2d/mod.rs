use cgmath;
use cgmath::{FixedArray, Matrix, Matrix4, Point, Point3, Quaternion, Rotation, ToMatrix4, Transform, Vector, Vector2, Vector3 };
use glfw;
use gfx;
use gfx::{Device, DeviceHelper, ToSlice};
use component::{ComponentStore,
    EntityComponent,
    EntityHandle,
    RawComponentHandle
};

#[shader_param(Quad)]
struct Params {
    #[name = "u_MVP"]
    mvp: [[f32, ..4], ..4],
    
    #[name = "u_Color"]
    color: [f32, ..3],

    #[name = "u_Size"]
    size: [f32, ..2]
}

#[vertex_format]
struct Vertex {
    #[name = "a_Pos"]
    #[as_float]
    pos: [i8, ..3],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    
    uniform mat4 u_MVP;
    uniform vec3 u_Color;
    uniform vec2 u_Size;

    in vec3 a_Pos;
    out vec4 v_Color;

    void main() {
        v_Color = vec4(u_Color, 1.0);
        gl_Position = u_MVP * vec4(a_Pos * vec3(u_Size.x, 0.0, u_Size.y), 1.0);
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

pub struct RenderComponent {
    pub entity: EntityHandle,
    pub color: Vector3<f32>,
    pub size: Vector2<f32>
}
impl RenderComponent {
    pub fn from_raw(raw: RawRenderComponent, entity_lookup: |RawComponentHandle| -> EntityHandle) -> RenderComponent {
        RenderComponent {
            entity: entity_lookup(raw.entity),
            color: raw.color,
            size: raw.size
        }
    }
}

#[deriving(Encodable, Decodable, Clone, PartialEq)]
pub struct RawRenderComponent {
    pub entity: RawComponentHandle,
    pub color: Vector3<f32>,
    pub size: Vector2<f32>
}
pub struct CameraComponent {
    entity: EntityHandle,
}
impl CameraComponent {
    pub fn new(entity: EntityHandle) -> CameraComponent {
        CameraComponent {
            entity: entity,
        }
    }
}

pub struct Renderer {
    frame: gfx::Frame,
    graphics: gfx::Graphics<gfx::GlDevice, gfx::GlCommandBuffer>,

    shader: gfx::ProgramHandle,
    mesh: gfx::Mesh,
    indices: gfx::BufferHandle<u8>
}
impl Renderer {
    /// Quickly open a new window
    /// and begin rendering.
    pub fn new(window: &mut glfw::Window) -> Renderer {
        let (w, h) = window.get_framebuffer_size();
        let frame = gfx::Frame::new(w as u16, h as u16);

        let device = gfx::GlDevice::new(|s| window.get_proc_address(s));
        let mut graphics = gfx::Graphics::new(device);

        let shader = graphics.device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
            .unwrap();
        
        let vertex_data = [
            Vertex { pos: [-1, 0, -1]},
            Vertex { pos: [ 1, 0, -1]},
            Vertex { pos: [ 1, 0, 1]},
            Vertex { pos: [-1, 0,  1]},
        ];

        let index_data: &[u8] = [
            0,  1,  2,  2,  3,  0,
        ];
        let mesh = graphics.device.create_mesh(vertex_data);
        let indices = graphics.device.create_buffer_static::<u8>(index_data);
        
        Renderer {
            frame: frame,
            graphics: graphics,
            shader: shader,
            mesh: mesh,
            indices: indices
        }
    }

    pub fn render(&mut self, cam: &CameraComponent, renderables: &mut ComponentStore<RenderComponent>, entities: &ComponentStore<EntityComponent>) {
        let drawstate = gfx::DrawState::new().depth(gfx::state::LessEqual, true);

        let batch: Quad = self.graphics.make_batch(
            &self.shader, &self.mesh, self.indices.to_slice(gfx::TriangleList), &drawstate).unwrap();

        let clear_data = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        self.graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &self.frame);
        
        let cament = entities.find(cam.entity).unwrap();
        let proj = cgmath::ortho(-1., 1., -1., 1., -1., 1.);
        
        let view = calc_view_matrix(cament.pos, cament.rot);
        let mut dead = Vec::new();
        for (handle, &renderable) in renderables.iter() {
            let ent = entities.find(renderable.entity);
            match ent {
                Some(ent) => {
                    if ent.handle == cament.handle {
                        continue;
                    };

                    let model = ent.make_matrix();
                    self.graphics.draw(&batch, &Params { color: renderable.color.into_fixed(), mvp: (proj * view * model).into_fixed(), size: renderable.size.into_fixed()}, &self.frame);
                },
                None => dead.push(handle)
            }
        }
        for handle in dead.into_iter() {
            renderables.remove(handle);
        }
        self.graphics.end_frame();
    }
}

fn calc_view_matrix(pos: Point3<f32>, rot: Quaternion<f32>) -> Matrix4<f32> {
    use cgmath::ToMatrix4;
    let rot = rot.invert().to_matrix4();
    
    let xlate = pos.to_vec().mul_s(-1.0);
    // z-up to y-up
    let coordswap = Matrix4::new(
        1., 0., 0., 0.,
        0., 0., 1., 0.,
        0., 1., 0., 0.,
        0., 0., 0., 1.,
    );

    coordswap * rot * Matrix4::from_translation(&xlate)
}
