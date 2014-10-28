pub use playercmd::PlayerCommand;
use component::{RawComponentHandle};
use component::components::NoHandleEntityComponent;
use renderer_2d::RawRenderComponent;

pub mod channel;
pub mod protocol;
pub mod delta;

#[deriving(Encodable, Decodable)]
pub enum ClientToServer {
    Connect,
    Playercmd(PlayerCommand),
    Disconnect
}

#[deriving(Encodable, Decodable)]
pub enum ServerToClient {
    Signon(SignonPacket),
    Update(UpdatePacket)
}

#[deriving(Encodable, Decodable)]
pub struct UpdatePacket {
    pub tick: u64,
    pub entity_updates: Vec<ComponentUpdate<NoHandleEntityComponent>>,
    pub render_updates: Vec<ComponentUpdate<RawRenderComponent>>
}

#[deriving(Encodable, Decodable)]
pub struct SignonPacket {
    pub handle: RawComponentHandle
}

#[deriving(Encodable, Decodable)]
pub struct ComponentUpdate<MarshalledComponent> {
    target: RawComponentHandle,
    data: ComponentUpdateType<MarshalledComponent>
}

#[deriving(Encodable, Decodable)]
pub enum ComponentUpdateType<MarshalledComponent> {
    Change(MarshalledComponent),
    Destroy
}

