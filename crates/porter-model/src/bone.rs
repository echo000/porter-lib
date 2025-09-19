use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;

use porter_utils::SanitizeExt;

/// Cleans a bone name.
fn sanitize_bone_name(name: String) -> String {
    let mut name = name.replace(' ', "_").sanitized();

    if name == "default" || name.is_empty() {
        name = String::from("_default");
    } else if name.as_bytes()[0].is_ascii_digit() {
        name = format!("_{name}");
    }

    name
}

/// Represents a bone in a skeleton of a model.
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: Option<String>,
    pub parent: i32,
    pub segment_scale_compensate: bool,
    pub local_position: Vector3,
    pub local_rotation: Quaternion,
    pub local_scale: Vector3,
    pub world_position: Vector3,
    pub world_rotation: Quaternion,
    pub world_scale: Vector3,
}

impl Bone {
    /// Constructs a new instance of a bone.
    pub fn new(name: Option<String>, parent: i32) -> Self {
        Self {
            name: name.map(sanitize_bone_name),
            parent,
            segment_scale_compensate: true,
            local_position: Vector3::zero(),
            local_rotation: Quaternion::identity(),
            local_scale: Vector3::one(),
            world_position: Vector3::zero(),
            world_rotation: Quaternion::identity(),
            world_scale: Vector3::one(),
        }
    }

    /// Sets the name.
    #[inline]
    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name.map(sanitize_bone_name);
        self
    }

    /// Sets segment scale compensate.
    #[inline]
    pub fn segment_scale_compensate(mut self, ssc: bool) -> Self {
        self.segment_scale_compensate = ssc;
        self
    }

    /// Sets the local position.
    #[inline]
    pub fn local_position(mut self, position: Vector3) -> Self {
        self.local_position = position;
        self
    }

    /// Sets the local rotation.
    #[inline]
    pub fn local_rotation(mut self, rotation: Quaternion) -> Self {
        self.local_rotation = rotation;
        self
    }

    /// Sets the scale.
    #[inline]
    pub fn local_scale(mut self, scale: Vector3) -> Self {
        self.local_scale = scale;
        self
    }

    /// Sets the world position.
    #[inline]
    pub fn world_position(mut self, position: Vector3) -> Self {
        self.world_position = position;
        self
    }

    /// Sets the world rotation.
    #[inline]
    pub fn world_rotation(mut self, rotation: Quaternion) -> Self {
        self.world_rotation = rotation;
        self
    }

    /// Sets the world scale.
    #[inline]
    pub fn world_scale(mut self, scale: Vector3) -> Self {
        self.world_scale = scale;
        self
    }

    /// Gets the local matrix (T * R * S).
    pub fn local_matrix(&self) -> Matrix4x4 {
        Matrix4x4::create_position(self.local_position)
            * Matrix4x4::create_rotation(self.local_rotation)
            * Matrix4x4::create_scale(self.local_scale)
    }

    /// Gets the world matrix (T * R * S).
    pub fn world_matrix(&self) -> Matrix4x4 {
        Matrix4x4::create_position(self.world_position)
            * Matrix4x4::create_rotation(self.world_rotation)
            * Matrix4x4::create_scale(self.world_scale)
    }
}
