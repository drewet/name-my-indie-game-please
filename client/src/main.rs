#![feature(phase, link_args)]

extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate native;
extern crate shared;

use cgmath::{Point, Point3};
use cgmath::Rotation3;
use cgmath::Rotation;
use cgmath::Vector3;
use cgmath::ToRad;
use glfw::Context;
use renderer::RenderComponent;
use shared::EntityComponent;

mod input;
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

    let mut entities = ComponentStore::new();
    let mut renderables = ComponentStore::new();
    let mut physicals = ComponentStore::new();

    let ent = EntityComponent::new(&mut entities, Point3::new(0.0, 0.01, 0.0), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
    renderables.add(RenderComponent { entity: ent });
    
    let cament = EntityComponent::new(&mut entities, Point3::new(0.0, 0.01, 0.0), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
    let cam = renderer::CameraComponent::new(cament);
    let mut controllable = shared::playercmd::ControllableComponent::new(cament);
    let camphys = physicals.add(shared::physics::PhysicsComponent::new(cament));

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::ContextVersion(3, 2));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

    let (mut window, events) = glfw
        .create_window(800, 600, "NMIGP", glfw::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorDisabled);

    let mut renderer = renderer::Renderer::new(&mut window);
    let mut input_integrator = input::MouseInputIntegrator::new();

    let mut motion = None;
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => { window.set_should_close(true) }
                glfw::KeyEvent(glfw::KeyUp, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, 0.5, 0.0))}
                glfw::KeyEvent(glfw::KeyDown, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, -0.5, 0.0))}
                glfw::KeyEvent(glfw::KeyLeft, _, glfw::Press, _) => {motion = Some(Vector3::new(-0.5, 0.0, 0.0))}
                glfw::KeyEvent(glfw::KeyRight, _, glfw::Press, _) => {motion = Some(Vector3::new(0.5, 0.0, 0.0))},
                glfw::KeyEvent(glfw::KeyPageUp, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, 0.0, 0.5))}
                glfw::KeyEvent(glfw::KeyPageDown, _, glfw::Press, _) => {motion = Some(Vector3::new(0.0, 0.0, -0.5))}
                glfw::KeyEvent(_, _, glfw::Release, _) => {motion = None}
                glfw::CursorPosEvent(xpos, ypos) => {
                    window.set_cursor_pos(0., 0.);
                    input_integrator.input(xpos as f32, ypos as f32);
                }
                _ => {},
            }
        }
    
        shared::physics::simulate_tick(&mut physicals, &mut entities);
        // networking
        //     get updates from server, update gamestate
        //     part of that is GC for component stores
        //     send input to server (no prediction yet, singleplayer)
        // sound, etc.
        //
        let motion = motion.unwrap_or(Vector3::new(0., 0., 0.,));

        let cmd = shared::playercmd::PlayerCommand {
            angles: cgmath::Rotation3::from_euler(cgmath::rad(0.), input_integrator.yaw.to_rad(), input_integrator.pitch.to_rad()),
            movement: motion
        };
        shared::playercmd::run_command(cmd, &mut controllable, &mut entities);

        renderer.render(&cam, &renderables, &entities);
        window.swap_buffers();
    }
}
