use cgmath::{
    Vector3
};

/// ATM, this is just a bounding box.
pub struct CollisionComponent {
    maxes: Vector3<f32>,
    mins: Vector3<f32>
}
