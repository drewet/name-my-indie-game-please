use cgmath::{Vector, Vector3, Point};
use {ComponentHandle, ComponentStore, PositionComponent};
use TICK_LENGTH;

pub mod collision;

pub struct PhysicsComponent {
    pub velocity: Vector3<f32>,
    position: ComponentHandle<PositionComponent>
}
impl PhysicsComponent {
    pub fn new(position: ComponentHandle<PositionComponent>) -> PhysicsComponent {
        PhysicsComponent {
            velocity: Vector3::new(0., 0., 0.),
            position: position
        }
    }
}

/// Runs one tick of simulation.
pub fn simulate_tick(physics: &mut ComponentStore<PhysicsComponent>, positions: &mut ComponentStore<PositionComponent>) {
    for physical in physics.iter_mut() {
        // FIXME: unwrap
        let pos = positions.find_mut(physical.position).unwrap();
        pos.pos = pos.pos.add_v(&physical.velocity.mul_s(TICK_LENGTH));
    }
}
