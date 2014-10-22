extern crate cgmath;
extern crate shared;
extern crate serialize;

use cgmath::{Point3, Point, Quaternion, Rotation, Rotation3};
use shared::EntityComponent;

fn main() {
    gameloop();
}

fn gameloop() {
    
    use std::io::net::ip::{Ipv4Addr, SocketAddr};
    use std::io::net::udp::UdpSocket;
    use shared::component::ComponentStore;

    let bindaddr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 18295 };
    let clientaddr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 18294 };
    let socket = match UdpSocket::bind(bindaddr) {
        Ok(s) => s,
        Err(e) => fail!("couldn't bind socket: {}", e),
    };
    let mut conn = socket.connect(clientaddr);

    let bindaddr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 18295 };
    let clientaddr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 18294 };
    
    let mut entities = ComponentStore::new();
    //let mut renderables = ComponentStore::new();
    //let mut physicals = ComponentStore::new();

    //let debugbox = EntityComponent::new(&mut entities, Point3::new(0.0, 0.01, 0.0), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
    
    
    let playerent = EntityComponent::new(&mut entities, Point3::new(0.0, 5., 0.0), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
    let mut playercontrollable = shared::playercmd::ControllableComponent::new(playerent);

    loop {
        use serialize::json;
        use shared::network::protocol::encode_full_update;
        
        let mut updatebuf = std::io::MemWriter::new();
        encode_full_update(&mut json::Encoder::new(&mut updatebuf),
            &entities,
            |e| e.to_nohandle()
        );
        let updatebuf = updatebuf.unwrap();
       // println!("{}", String::from_utf8(updatebuf.clone()).unwrap());
        conn.write(updatebuf.as_slice()).unwrap();

        let mut cmdbuf = [0u8, ..4000];
        match conn.read(&mut cmdbuf) {
            Ok(len) => {
                let cmdbuf = cmdbuf.as_slice().slice_to(len);
                let cmdstr = std::str::from_utf8(cmdbuf).unwrap();
                let cmd: shared::playercmd::PlayerCommand = json::decode(cmdstr).unwrap();

                shared::playercmd::run_command(cmd, &mut playercontrollable, &mut entities);
            },
            Err(_) => ()
        }
    }
}
