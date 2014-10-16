use cgmath;
use cgmath::{Quaternion, Rotation, Rotation3, Vector, Vector2, Vector3};
use cgmath::rad;

pub struct MouseInputIntegrator {
    pub angles: Quaternion<f32>,
    pub sensitivity: Vector2<f32>,

    accum: Vector2<f32>
}

impl MouseInputIntegrator {
    pub fn new() -> MouseInputIntegrator {
        MouseInputIntegrator {
            angles: Rotation3::from_euler(rad(0.), rad(0.), rad(0.)),
            sensitivity: Vector2::new(0.01, 0.01),
            accum: Vector2::new(0.0, 0.0)
        }
    }

    pub fn input(&mut self, x: f32, y: f32) {
        self.accum = self.accum + Vector2::new(x, y);
        self.angles = Rotation3::from_euler(
            rad(self.accum.x * -1.0 * self.sensitivity.x),
            rad(0.),
            rad(self.accum.y * -1.0 * self.sensitivity.y),
        );
    }
}
  
