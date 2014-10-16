use cgmath;
use cgmath::{Quaternion, Rotation, Rotation3, Vector, Vector2, Vector3};
use cgmath::{deg, rad};

pub struct MouseInputIntegrator {
    pub pitch: cgmath::Deg<f32>,
    pub yaw: cgmath::Deg<f32>,
    pub sensitivity: Vector2<f32>,
}

impl MouseInputIntegrator {
    pub fn new() -> MouseInputIntegrator {
        MouseInputIntegrator {
            pitch: deg(0.), yaw: deg(0.),
            sensitivity: Vector2::new(0.1, 0.1),
        }
    }

    pub fn input(&mut self, x: f32, y: f32) {
        self.pitch = deg(clamp_pitch(self.pitch.s + (y * self.sensitivity.y)));
        self.yaw = deg(wrap_yaw(self.yaw.s + (x * self.sensitivity.x)));
    }
}

/// Clamps pitch to +/-90deg.
fn clamp_pitch(pitch: f32) -> f32 {
    use std::cmp::{partial_max, partial_min};
    partial_max(partial_min(pitch, 90.).unwrap(), -90.).unwrap()
}

/// Wraps yaw around 360 degrees.
fn wrap_yaw(yaw: f32) -> f32 {
    (yaw + 360.0) % 360.0
}
