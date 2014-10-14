use cgmath;
use cgmath::ToMatrix4;
use cgmath::Matrix4;
use cgmath::Point;

pub struct PositionComponent {
    pub pos: cgmath::Point3<f32>,
    pub rot: cgmath::Quaternion<f32>
}

impl ToMatrix4<f32> for PositionComponent {
    /// Create a transformation matrix.
    fn to_matrix4(&self) -> Matrix4<f32> {
        Matrix4::from_translation(&self.pos.to_vec()) * self.rot.to_matrix4()
    }
}
