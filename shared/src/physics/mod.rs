use cgmath::{Vector, Vector3, Point};
use {ComponentStore, EntityHandle, EntityComponent};
use TICK_LENGTH;

pub mod collision;

pub struct PhysicsComponent {
    pub velocity: Vector3<f32>,
    entity: EntityHandle
}
impl PhysicsComponent {
    pub fn new(entity: EntityHandle) -> PhysicsComponent {
        PhysicsComponent {
            velocity: Vector3::new(0., 0., 0.),
            entity: entity
        }
    }
}

/// Runs one tick of simulation.
pub fn simulate_tick(physics: &mut ComponentStore<PhysicsComponent>, entities: &mut ComponentStore<EntityComponent>) {
    let mut dead = Vec::new();
    'l: for (handle, physical) in physics.iter_mut() {
        // FIXME: unwrap
        let ent = entities.find_mut(physical.entity);
        
        match ent {
            Some(ent) => {
                ent.pos = ent.pos.add_v(&physical.velocity.mul_s(TICK_LENGTH));
            },
            None => dead.push(handle)
        };
    }
    for handle in dead.into_iter() {
        physics.remove(handle);
    }

}
