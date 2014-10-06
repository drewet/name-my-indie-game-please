#![feature(phase)]

extern crate glutin;
#[phase(plugin, link)]
extern crate glium_core_macros;

extern crate glium_core;
#[vertex_format]
#[allow(non_snake_case)]
struct Vertex {
    iPosition: [f32, ..2],
    iTexCoords: [f32, ..2],
}

#[uniforms]
#[allow(non_snake_case)]
struct Uniforms<'a> {
    uMatrix: [[f32, ..4], ..4],
}




fn main() {
    use glium_core::DisplayBuild;

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("Hello world".to_string())
        .build_glium_core().unwrap();

    let vertex_buffer = glium_core::VertexBuffer::new(&display, vec![
    Vertex { iPosition: [-1.0, -1.0], iTexCoords: [0.0, 1.0] },
    Vertex { iPosition: [-1.0,  1.0], iTexCoords: [0.0, 0.0] },
    Vertex { iPosition: [ 1.0,  1.0], iTexCoords: [1.0, 0.0] },
    Vertex { iPosition: [ 1.0, -1.0], iTexCoords: [1.0, 1.0] }
]);

	let index_buffer = glium_core::IndexBuffer::new(&display, glium_core::TrianglesList,
    &[0u8, 1, 2, 0, 2, 3]);
static VERTEX_SRC: &'static str = "
    #version 110

    uniform mat4 uMatrix;

    attribute vec2 iPosition;
    attribute vec2 iTexCoords;

    varying vec2 vTexCoords;

    void main() {
        gl_Position = vec4(iPosition, 0.0, 1.0) * uMatrix;
        vTexCoords = iTexCoords;
    }
";

static FRAGMENT_SRC: &'static str = "
    #version 110
    varying vec2 vTexCoords;

    void main() {
        gl_FragColor = vec4(vTexCoords.x, vTexCoords.y, 0.0, 1.0);
    }
";

let program = glium_core::Program::new(&display, VERTEX_SRC, FRAGMENT_SRC, None).unwrap();

	let mut uniforms = Uniforms { uMatrix: [[1., 0., 0., 0.],
		[0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]] };
	
loop {
let mut target = display.draw();
target.clear_color(0.0, 0.0, 0.0, 0.0);
target.draw(glium_core::BasicDraw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default()));
target.finish();
}
}
