use cgmath;
use cgmath::{Matrix4, Point, Point3, ToMatrix4, Quaternion};
use component::{ComponentHandle, ComponentStore};

pub type EntityHandle = ComponentHandle<EntityComponent>;

/// Represents an entity in the world.
pub struct EntityComponent {
    pub handle: EntityHandle,

    pub pos: Point3<f32>,
    pub rot: Quaternion<f32>
}
#[deriving(Encodable, Decodable, Clone, PartialEq)]
pub struct NoHandleEntityComponent {
    pub pos: Point3<f32>,
    pub rot: Quaternion<f32>
}
impl EntityComponent {
    pub fn get_handle(&self) -> EntityHandle {
        self.handle
    }
    pub fn to_nohandle(&self) -> NoHandleEntityComponent {
        NoHandleEntityComponent { pos: self.pos, rot: self.rot }
    }
    pub fn from_nohandle(e: &NoHandleEntityComponent, handle: EntityHandle) -> EntityComponent {
        EntityComponent {
            handle: handle,
            pos: e.pos,
            rot: e.rot
        }
    }
    
    /// Constructs an EntityComponent inside a
    pub fn new(ents: &mut ComponentStore<EntityComponent>,
              pos: Point3<f32>,
              rot: Quaternion<f32>) -> EntityHandle {
        ents.add_with_handle(|handle| EntityComponent {
            handle: handle,
            pos: pos,
            rot: rot
        })
    }
    /// Create a transformation matrix from entity-space
    /// to world-space.
    pub fn make_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(&self.pos.to_vec()) * self.rot.to_matrix4()
    }
}
