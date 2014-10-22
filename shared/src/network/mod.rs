pub use playercmd::PlayerCommand;
use component::{EntityComponent, RawComponentHandle};
use component::components::NoHandleEntityComponent;

pub mod protocol;

#[deriving(Encodable, Decodable)]
pub enum ClientToServer {
    Connect,
    Playercmd(PlayerCommand),
    Disconnect
}

#[deriving(Encodable, Decodable)]
pub enum ServerToClient {
    Signon(SignonPacket),
    Update(String)
}

#[deriving(Encodable, Decodable)]
pub struct SignonPacket {
    pub handle: RawComponentHandle
}
