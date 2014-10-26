extern crate flate;
extern crate cgmath;
extern crate shared;
extern crate serialize;
extern crate time;

use cgmath::{Point3, Rotation3};
use shared::{ComponentHandle, EntityComponent, EntityHandle};
use shared::component::components::NoHandleEntityComponent;
use shared::network::{ClientToServer, Connect, Disconnect, Playercmd};
use shared::network::channel::NetChannel;
use std::collections::HashMap;

fn main() {
    gameloop();
}

struct Client {
    addr: std::io::net::ip::SocketAddr,
    channel: NetChannel,

    entity: EntityHandle,
    controllable: ComponentHandle<shared::playercmd::ControllableComponent>,
    connstate: ConnectionState,
    last_acked_tick: u64,
}

#[deriving(PartialEq, Eq)]
enum ConnectionState {
    SigningOn,
    Playing,
    TimingOut
}

fn gameloop() {
    
    use std::io::net::ip::{Ipv4Addr, SocketAddr};
    use std::io::net::udp::UdpSocket;
    use shared::component::ComponentStore;

    let bindaddr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 18295 };
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
    
    let mut current_tick = 0u64;

    let mut ent_deltas: shared::network::delta::DeltaEncoder<EntityComponent, NoHandleEntityComponent> = shared::network::delta::DeltaEncoder::new(64);

    let mut next_tick_time = time::precise_time_s();
    loop {
        use serialize::json;
        
        'timing: loop {
            let starttime = time::precise_time_s();
            let time_until_next = next_tick_time - starttime;

            if time_until_next <= 0. {
                next_tick_time = next_tick_time + (shared::TICK_LENGTH as f64);
                //println!("{}FPS", 1.0 / (starttime - last_frame_start));
                
                break 'timing;
            /* } else if time_until_next < 0.002 {
                continue 'timing; */
            } else {
                std::io::timer::sleep(std::time::Duration::milliseconds(1));
            }
        }

        current_tick = current_tick + 1;
        

        // incoming packets
        let mut recvbuf = [0u8, ..8192]; 
        loop { match socket.recv_from(&mut recvbuf) {
            Ok((len, addr)) => {
                let data = recvbuf.as_slice().slice_to(len);

                // borrow checker hack
                let is_new = match clients.find_mut(&addr) {
                    Some(client) => {

                        let prevseq = client.channel.get_incoming_sequencenr();

                        let data = client.channel.recv_unreliable(data).unwrap();
                        let dropped_packets = client.channel.get_incoming_sequencenr() - prevseq;
                        if dropped_packets > 0 {
                            println!("Lost {} client packets...", dropped_packets)
                        }

                        let data = flate::inflate_bytes_zlib(data.as_slice()).unwrap();
                        let cmdstr = std::str::from_utf8(data.as_slice()).unwrap();
                        let cmd: shared::network::ClientToServer = json::decode(cmdstr).unwrap();

                        match cmd {

                            Playercmd(cmd) => {
                                client.last_acked_tick = cmd.tick;
                                for _ in range(0, dropped_packets + 1) {
                                    shared::playercmd::run_command(cmd,controllables.find_mut(client.controllable).unwrap(), &mut entities);
                                }
                                client.connstate = Playing;
                                false
                            }
                            Connect => false,
                            Disconnect => unimplemented!()
                        }
                    },
                    None => true
                };

                if is_new {
                    println!("Got connect from {}!", addr);
                    let playerent = EntityComponent::new(&mut entities,
                                                         Point3::new(0.0, 5., 0.0),
                                                         Rotation3::from_euler(cgmath::rad(0.), cgmath::rad(0.), cgmath::rad(0.))
                                                        );
                    let controllable = controllables.add(shared::playercmd::ControllableComponent::new(playerent));

                    clients.remove(&addr);
                    clients.insert(addr, Client {
                        addr: addr,
                        channel: NetChannel::new(),
                        entity: playerent,
                        controllable: controllable,
                        connstate: SigningOn,
                        last_acked_tick: 0
                    });
                }
            },
            Err(_) => break,
        }}

        ent_deltas.add_state(&entities, |ent| ent.to_nohandle());

        // outgoing
        for (_, client) in clients.iter_mut() {
            if client.last_acked_tick != 0 && client.last_acked_tick + 512 < current_tick {
                client.connstate = TimingOut;
            };
            match client.connstate {
                Playing => {
                    let update = shared::network::Update(shared::network::UpdatePacket {
                        tick: current_tick,
                        entity_updates: ent_deltas.create_delta(9999)
                    });
                    let update = json::encode(&update);
                    let update = update.into_bytes();

                    let update = flate::deflate_bytes_zlib(update.as_slice()).unwrap();
                    let datagram = client.channel.send_unreliable(update.as_slice());
                    socket.send_to(datagram.unwrap().as_slice(), client.addr).unwrap();
                },
                SigningOn => {
                    let signon = shared::network::Signon(shared::network::SignonPacket {
                        handle: client.entity.to_raw()
                    });
                    let signon = json::encode(&signon).into_bytes();
                    let signon = flate::deflate_bytes_zlib(signon.as_slice()).unwrap();
                    let datagram = client.channel.send_unreliable(signon.as_slice());
                    socket.send_to(datagram.unwrap().as_slice(), client.addr).unwrap();
                },
                TimingOut => ()
            }
        }
    }
}
