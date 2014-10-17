use component::{ComponentHandle, ComponentStore, EntityComponent, EntityHandle};
use physics::PhysicsComponent;
use cgmath;
use cgmath::{Point, Vector, Vector3, Quaternion};

pub struct ControllableComponent {
    entity: EntityHandle
    
    //pub topspeed: f32,

    //pub lastcmd: PlayerCommand
}
impl ControllableComponent {
    pub fn new(entity: EntityHandle) -> ControllableComponent {
        ControllableComponent {
            entity: entity
        }
    }
}

/// An input from a player, roughly abstracting over their inputs
/// during the course of a single game tick.
pub struct PlayerCommand {
    pub angles: Quaternion<f32>,
    /// RELATIVE TO ANGLES!
    pub movement: Vector3<f32>,
}

/// Runs a player's command for a single game tick.
pub fn run_command(cmd: PlayerCommand,
                   controllable: &mut ControllableComponent,
                   entities: &mut ComponentStore<EntityComponent>) {
    use cgmath::Rotation;

    let ent = entities.find_mut(controllable.entity).unwrap();

    // TODO: validate angles and movoment
    ent.rot = cmd.angles;
    // TODO: collision check. there should be a function in physics::
    // for trying to move by a vector, with collisions.
    ent.pos = ent.pos.add_v(&cmd.angles.rotate_vector(&cmd.movement));
}
