#![feature(phase, link_args)]

extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate native;
extern crate shared;

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
    loop {
        // get input from user
        // networking
        //     get updates from server, update gamestate
        //     send input to server (no prediction yet, singleplayer)
        // render, sound, etc.
        unimplemented!()
    }
}
