#![feature(phase, link_args)]

extern crate cgmath;
extern crate engine;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate flate;
extern crate native;
extern crate serialize;
extern crate time;

use cgmath::{Point, Point3};
use cgmath::Rotation3;
use cgmath::Vector3;
use cgmath::Vector;
use cgmath::ToRad;
use cgmath::rad;
use glfw::Context;
use engine::renderer::RenderComponent;
use engine::EntityComponent;
use serialize::json;

use engine::renderer;
use engine::prediction;
use engine::input;
use engine::component::ComponentStore;
use engine::network::channel::NetChannel;

use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::net::udp::{UdpSocket, UdpStream};

use engine::network::{ServerToClient, Signon, Update, SignonPacket};

// A weird hack to get arguments to the linker.
/*#[cfg(target_family="windows")]
  mod windows_subsystem_hack {
  #[link_args="-Wl,--subsystem,windows -mwindows"]
  extern {}
  }*/

// We need to run on the main thread for GLFW, so ensure we are using the `native` runtime. This is
// technically not needed, since this is the default, but it's not guaranteed.
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    connect(SocketAddr {
        ip: Ipv4Addr(162,243,139,73),
        port: 18295
    }, 10)
}

fn connect(serveraddr: SocketAddr, mut retries: u32) {
    let mut stream = UdpSocket::bind(SocketAddr { ip: Ipv4Addr(0,0,0,0), port: 0}).unwrap().connect(serveraddr);
    let mut netchan = NetChannel::new();

    while retries > 0 {
        let encoded_packet = json::encode(&engine::network::Connect).into_bytes();
        let compressed_packet = flate::deflate_bytes_zlib(encoded_packet.as_slice()).unwrap();
        let packet = netchan.send_unreliable(compressed_packet.as_slice()).unwrap();
        stream.write(packet.as_slice()).unwrap();

        let mut recvbuf = [0u8, ..16384];
        // 100ms timeout
        stream.as_socket(|sock| sock.set_read_timeout(Some(100)));
        match stream.read(recvbuf) {
            Ok(0) => continue,
            Ok(len) => {
                let datagram = recvbuf.as_slice().slice_to(len);
                let compressed_packet = netchan.recv_unreliable(datagram).unwrap();
                let packet = flate::inflate_bytes_zlib(compressed_packet.as_slice()).unwrap();
                let msg: ServerToClient = json::decode(std::str::from_utf8(packet.as_slice()).unwrap()).unwrap();

                match msg {
                    Signon(signon) => {
                        gameloop(stream, netchan, signon);
                        return;
                    },
                    _ => ()
                }
            },
            Err(std::io::IoError { kind: std::io::TimedOut, ..}) => (),
            Err(e) => fail!("Network error connecting to server: {}", e)
        }
        retries -= 1;
    }

    fail!("Out of retries while connecting to server!")
}

fn gameloop(mut stream: UdpStream, mut netchan: NetChannel, signon: SignonPacket) {
    stream.as_socket(|sock| sock.set_read_timeout(Some(0)));


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

    let mut entities = ComponentStore::new();
    let mut renderables = ComponentStore::new();

    let mut renderer = renderer::Renderer::new(&mut window);
    let mut input_integrator = input::MouseInputIntegrator::new();

    let mut motion = None;
    let mut hdict = std::collections::HashMap::new();

    let localplayer = EntityComponent::new(&mut entities, Point3::new(0., 0., 0.),
        Rotation3::from_euler(rad(0.), rad(0.), rad(0.)));

    hdict.insert(signon.handle, localplayer);

    let cam = renderer::CameraComponent::new(localplayer);

    let mut last_command = 0.;
    let mut servertick = 0;

    let mut prediction = prediction::Prediction::new(engine::playercmd::ControllableComponent::new(localplayer));

    while !window.should_close() {
        use engine::network::protocol::apply_update;

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

        // networking
        //     get updates from server, update gamestate
        //     part of that is GC for component stores
        //     send input to server (no prediction yet, singleplayer)
        // sound, etc.
        //
        let motion = motion.unwrap_or(Vector3::new(0., 0., 0.,)).mul_s(0.1);


        let mut buf = [0u8, ..8192];

        loop { match stream.read(buf) {
            Ok(0) => (),
            Ok(len) => {

                let packet = netchan.recv_unreliable((buf.slice_to(len))).unwrap();
                let packet = flate::inflate_bytes_zlib(packet.as_slice()).expect("Decompression!");
                let packet = std::str::from_utf8(packet.as_slice()).unwrap();
                let packet: ServerToClient = json::decode(packet).unwrap();
                match packet {
                    Update(update) => {
                        servertick = update.tick;
                        apply_update(update.entity_updates.into_iter(), &mut hdict, &mut entities, |e, h| EntityComponent::from_nohandle(&e, h), |e, store| {
                            println!("Adding new entity.");
                            let handle = store.add_with_handle(|handle| EntityComponent::from_nohandle(&e, handle));
                            renderables.add(RenderComponent{entity: handle});
                            handle
                        });
                        prediction.update(netchan.get_acked_outgoing_sequencenr(), &entities);
                    },
                    Signon(_) => ()
                }
            },
            Err(ref e) if e.kind == std::io::TimedOut => break,
            Err(e) => fail!("Network error: {}", e)
        } };

        if last_command + (engine::TICK_LENGTH as f64) < (framestart_ns as f64 / 1000. / 1000. / 1000.) { 
            last_command = framestart_ns as f64 / 1000. / 1000. / 1000.;

            let cmd = engine::playercmd::PlayerCommand {
                tick: servertick,
                angles: cgmath::Rotation3::from_euler(cgmath::rad(0.), input_integrator.yaw.to_rad(), input_integrator.pitch.to_rad()),
                movement: motion
            };


            let encoded_packet = json::encode(&engine::network::Playercmd(cmd)).into_bytes();
            let compressed_packet = flate::deflate_bytes_zlib(encoded_packet.as_slice()).unwrap();
            let packet = netchan.send_unreliable(compressed_packet.as_slice()).unwrap();
            stream.write(packet.as_slice()).unwrap();

            prediction.predict(cmd, netchan.get_outgoing_sequencenr());
        }

        renderer.render(&cam, &mut renderables, prediction.get_entities().unwrap_or(&entities));

        //println!("{}", netchan.get_latency());

        window.swap_buffers();

        let frameend_ns = time::precise_time_ns();
        let frametime_ns = frameend_ns - framestart_ns;
        let fps = 1000 * 1000 * 1000 / frametime_ns;
        window.set_title(format!("{}FPS, frametime: {}ns", fps, frametime_ns).as_slice());
    }
}
