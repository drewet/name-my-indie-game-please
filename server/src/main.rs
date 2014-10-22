extern crate cgmath;
extern crate shared;
extern crate serialize;

use cgmath::{Point3, Point, Quaternion, Rotation, Rotation3};
use shared::{ComponentHandle, EntityComponent, EntityHandle};
use shared::network::{ClientToServer, Connect, Playercmd};
use std::collections::HashMap;
fn main() {
    gameloop();
}

pub struct Client {
    addr: std::io::net::ip::SocketAddr,
    entity: EntityHandle,
    controllable: ComponentHandle<shared::playercmd::ControllableComponent>
}

fn gameloop() {
    
    use std::io::net::ip::{Ipv4Addr, SocketAddr};
    use std::io::net::udp::UdpSocket;
    use shared::component::ComponentStore;

    let bindaddr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 18295 };
    let mut socket = match UdpSocket::bind(bindaddr) {
        Ok(s) => s,
        Err(e) => fail!("couldn't bind socket: {}", e),
    };
    socket.set_read_timeout(Some(0));

    let mut entities = ComponentStore::new();
    let mut controllables = ComponentStore::new();
    //let mut physicals = ComponentStore::new();

    //let debugbox = EntityComponent::new(&mut entities, Point3::new(0.0, 0.01, 0.0), Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.)));
    
    let mut clients: HashMap<SocketAddr, Client> = HashMap::new();

    loop {
        use serialize::json;
        use shared::network::protocol::encode_full_update;
        
        let mut updatebuf = std::io::MemWriter::new();
        encode_full_update(&mut json::Encoder::new(&mut updatebuf),
            &entities,
            |e: &EntityComponent| e.to_nohandle()
        );
        let update_data = updatebuf.unwrap();
        let update = shared::network::Update(String::from_utf8(update_data).unwrap());
        let update = json::encode(&update).into_bytes();

        // incoming packets
        let mut recvbuf = [0u8, ..8192];
        match socket.recv_from(&mut recvbuf) {
            Ok((len, addr)) => {
                let data = recvbuf.as_slice().slice_to(len);
                // borrow checker hack
                let is_new = match clients.find_mut(&addr) {
                    Some(client) => {
                        let cmdstr = std::str::from_utf8(data).unwrap();
                        let cmd: shared::network::ClientToServer = json::decode(cmdstr).unwrap();
                        match cmd {
                            Playercmd(cmd) => {
                                shared::playercmd::run_command(cmd,controllables.find_mut(client.controllable).unwrap(), &mut entities);
                                false
                            }
                            Connect => true,
                            Disconnect => unimplemented!()
                        }
                    },
                    None => {
                        true // is new
                    }
                };
                if is_new {
                    println!("Got connect!");
                    let playerent = EntityComponent::new(&mut entities,
                                                         Point3::new(0.0, 5., 0.0),
                                                         Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.))
                                                        );
                    let controllable = controllables.add(shared::playercmd::ControllableComponent::new(playerent));

                    clients.swap(addr, Client {
                        addr: addr,
                        entity: playerent,
                        controllable: controllable
                    });

                    let signon = shared::network::Signon(shared::network::SignonPacket {
                        handle: playerent.to_raw()
                    });
                    let signon = json::encode(&signon).into_bytes();
                    socket.send_to(signon.as_slice(), addr).unwrap();
                }
            },
            Err(e) => ()
        }
        // outgoing
        for client in clients.values() {
            socket.send_to(update.as_slice(), client.addr).unwrap();
        }
    }
}
