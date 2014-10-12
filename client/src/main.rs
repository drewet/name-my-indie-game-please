#![feature(phase, link_args)]

extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate native;
extern crate shared;

use cgmath::Vector3;
use glfw::Context;
use renderer::RenderComponent;
use shared::PositionComponent;

mod renderer;

// A weird hack to get arguments to the linker.
#[cfg(target_family="windows")]
mod windows_subsystem_hack {
    #[link_args="-Wl,--subsystem,windows -mwindows"]
    extern {}
}

// We need to run on the main thread for GLFW, so ensure we are using the `native` runtime. This is
// technically not needed, since this is the default, but it's not guaranteed.
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    gameloop()
}

fn gameloop() {
    use shared::component::ComponentStore;

    let mut positions = ComponentStore::new();
    let mut renderables = ComponentStore::new();

    let pos = positions.add(PositionComponent { pos: cgmath::Vector3::new(1.0, 0.0, -5.0) } );
    let renderable = renderables.add(RenderComponent { pos: pos });
    
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::ContextVersion(3, 2));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

    let (window, events) = glfw
        .create_window(640, 480, "NMIGP", glfw::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_key_polling(true);

    let mut renderer = renderer::Renderer::new(&mut glfw, window);

    loop {
        // get input from user
        // networking
        //     get updates from server, update gamestate
        //     part of that is GC for component stores
        //     send input to server (no prediction yet, singleplayer)
        // render, sound, etc.
        glfw.poll_events();
        let mut motion = None;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => {return;}
                glfw::KeyEvent(glfw::KeyUp, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, 0.5, 0.0))}
                glfw::KeyEvent(glfw::KeyDown, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, -0.5, 0.0))}
                glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) => {motion = Some(Vector3::new(-0.5, 0.0, 0.0))}
                glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) => {motion = Some(Vector3::new(0.5, 0.0, 0.0))}
                _ => {},
            }
        }
        positions.find_mut(pos).map(|comp| {
            comp.pos = comp.pos + motion.unwrap_or(Vector3::new(0., 0., 0.));
        });

        renderer.render(&renderables, &positions);
        // unimplemented!()
    }
}
