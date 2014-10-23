#![feature(phase, link_args)]

extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate flate;
extern crate native;
extern crate serialize;
extern crate shared;
extern crate time;

use cgmath::{Point, Point3};
use cgmath::Rotation3;
use cgmath::Rotation;
use cgmath::Vector3;
use cgmath::Vector;
use cgmath::ToRad;
use glfw::Context;
use renderer::RenderComponent;
use shared::EntityComponent;
use std::io::net::udp::UdpSocket;
use serialize::json;

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

fn find_free_port(start_port: u16) -> UdpSocket {
    use std::io::net::ip::{Ipv4Addr, SocketAddr};

    for port in range(start_port, std::u16::MAX) {
        let bindaddr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: port };
        
        match UdpSocket::bind(bindaddr) {
            Ok(s) => return s,
            Err(std::io::IoError{ kind: std::io::ConnectionRefused, ..}) => (),
            Err(e) => {
                fail!("couldn't bind socket: {}", e)
            }
        }
    } 
    unreachable!()
}
fn gameloop() {
    use shared::component::ComponentStore;

    use std::io::net::ip::{Ipv4Addr, SocketAddr};
    use std::io::net::udp::UdpSocket;

    let serveraddr = SocketAddr { ip: Ipv4Addr(162, 243, 139, 73), port: 18295 };

    let mut socket = find_free_port(18296);
    socket.set_read_timeout(Some(0));
    let mut conn = socket.connect(serveraddr);
    
    let mut entities = ComponentStore::new();
    let mut renderables = ComponentStore::new();
    let mut cam = None;

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
    let mut hdict = std::collections::HashMap::new();

    let mut last_command = 0.;
    let mut localplayer = None;

    let mut signedon = false;
    let mut servertick = 0;
    while !window.should_close() {
        use shared::network::protocol::apply_update;
        
        let framestart_ns = time::precise_time_ns();

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
    
        // shared::physics::simulate_tick(&mut physicals, &mut entities);
        // networking
        //     get updates from server, update gamestate
        //     part of that is GC for component stores
        //     send input to server (no prediction yet, singleplayer)
        // sound, etc.
        //
        let motion = motion.unwrap_or(Vector3::new(0., 0., 0.,)).mul_s(0.1);

        let cmd = shared::playercmd::PlayerCommand {
            tick: servertick,
            angles: cgmath::Rotation3::from_euler(cgmath::rad(0.), input_integrator.yaw.to_rad(), input_integrator.pitch.to_rad()),
            movement: motion
        };
        if last_command + (shared::TICK_LENGTH as f64) < (framestart_ns as f64 / 1000. / 1000. / 1000.) {
            last_command = (framestart_ns as f64 / 1000. / 1000. / 1000.);
            let pkt = flate::deflate_bytes_zlib(if cam.is_some() {
                json::encode(&shared::network::Playercmd(cmd)).into_bytes()
            } else {
                json::encode(&shared::network::Connect).into_bytes()
            }.as_slice()).unwrap();
            conn.write(pkt.as_slice());
        }

        let mut buf = [0u8, ..16000];

        loop { match conn.read(&mut buf) {
            Ok(len) => {
                use shared::network::{ServerToClient, Signon, Update};
                let buf = flate::inflate_bytes_zlib(buf.slice_to(len)).expect("Decompression!");
                let msg = std::str::from_utf8(buf.as_slice()).unwrap();
                let packet: ServerToClient = json::decode(msg).unwrap();
                match packet {
                    Update(update) => {
                        servertick = update.tick;
                        apply_update(update.entity_updates.into_iter(), &mut hdict, &mut entities, |e, h| EntityComponent::from_nohandle(&e, h), |e, store| {
                            println!("Adding new entity.");
                            let handle = store.add_with_handle(|handle| EntityComponent::from_nohandle(&e, handle));
                            renderables.add(RenderComponent{entity: handle});
                            handle
                        });
                    },
                    Signon(signon) if localplayer.is_none() => {
                        println!("Got signon packet.");
                        signedon = true;
                        let playerent = EntityComponent::new(&mut entities, Point3::new(0., 0., 0.,), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
                        hdict.insert(signon.handle, playerent);
                        cam = Some(renderer::CameraComponent::new(playerent));
                        localplayer = Some(playerent);
                    }
                    Signon(_) => ()
                }
            },
            Err(e) => break,
        } };
        
        match cam {
            Some(ref cam) => renderer.render(cam, &mut renderables, &entities),
            None => ()
        };

        window.swap_buffers();

        let frameend_ns = time::precise_time_ns();
        let frametime_ns = frameend_ns - framestart_ns;
        let fps = 1000 * 1000 * 1000 / frametime_ns;
        window.set_title(format!("{}FPS, frametime: {}ns", fps, frametime_ns).as_slice());
        if fps < 200 {
            println!("SLOW FRAME! {}FPS, frametime: {}ns", fps, frametime_ns);
        }
    }
}
