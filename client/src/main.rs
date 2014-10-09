#![feature(phase, link_args)]

extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate native;
extern crate shared;

use glfw::Context;

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
    use shared::entity::{Component,
        ComponentID,
        ComponentStore,
        EntityID,
        EntityStore
    };

    let mut estore = EntityStore::new();

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::ContextVersion(3, 2));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

    let (window, events) = glfw
        .create_window(640, 480, "NMIGP", glfw::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_key_polling(true);

    let (w, h) = window.get_framebuffer_size();
    let frame = gfx::Frame::new(w as u16, h as u16);

    let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));
    let mut graphics = gfx::Graphics::new(device);

    loop {
        // get input from user
        // networking
        //     get updates from server, update gamestate
        //     part of that is GC for component stores
        //     send input to server (no prediction yet, singleplayer)
        // render, sound, etc.
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) =>
                    {window.set_should_close(true); return},
                        _ => {},
                }
            }
            renderer::render(&window, &frame, &mut graphics);
       // unimplemented!()
    }
}
