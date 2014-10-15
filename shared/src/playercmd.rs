use component::{ComponentHandle, PositionComponent};
use cgmath;
use cgmath::{Point, Vector, Vector3, Quaternion};

pub struct ControllableComponent {
    position: ComponentHandle<PositionComponent>
}

/// An input from a player, roughly abstracting over their inputs
/// during the course of a single game tick.
pub struct PlayerCommand {
    angles: Quaternion<f32>,
    movement: Vector3<f32>
}
