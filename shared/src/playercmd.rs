use component::{ComponentHandle, ComponentStore, PositionComponent};
use physics::PhysicsComponent;
use cgmath;
use cgmath::{Point, Vector, Vector3, Quaternion};

pub struct ControllableComponent {
    position: ComponentHandle<PositionComponent>,
    
    //pub topspeed: f32,

    //pub lastcmd: PlayerCommand
}
impl ControllableComponent {
    pub fn new(position: ComponentHandle<PositionComponent>) -> ControllableComponent {
        ControllableComponent {
            position: position
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
                   positions: &mut ComponentStore<PositionComponent>) {
    use cgmath::Rotation;

    let pos = positions.find_mut(controllable.position).unwrap();

    // TODO: validate angles and movoment
    pos.rot = cmd.angles;
    // TODO: collision check. there should be a function in physics::
    // for trying to move by a vector, with collisions.
    pos.pos = pos.pos.add_v(&cmd.angles.rotate_vector(&cmd.movement));
}
