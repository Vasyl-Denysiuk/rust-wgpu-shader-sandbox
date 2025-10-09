pub struct WorldCamera {
    position: glam::Vec3,
    rotation: glam::Vec3,
    aspect: f32,
    fovy: f32,
    z_near: f32,
    z_far: f32,
}

impl WorldCamera {
    const CAM_SPEED: f32 = 0.03;

    pub fn new() -> WorldCamera {
        WorldCamera {
            position: glam::vec3(0.0, 0.0, 0.0),
            rotation: glam::vec3(0.0, 0.0, 0.0),
            aspect: 1.,
            fovy: 45f32.to_radians(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection(&self) -> glam::Mat4 {
        let rotation_matrix = glam::Mat4::from_euler(
            glam::EulerRot::ZYX, 
            self.rotation.z, 
            self.rotation.y, 
            self.rotation.x
        );
        let translation_matrix = glam::Mat4::from_translation(-self.position);

        let view = rotation_matrix * translation_matrix;
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.z_near, self.z_far);

        proj * view
    }

    pub fn forward(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position += rot * glam::Vec3::new(0.0, 0.0, -1.0) * Self::CAM_SPEED;
    }

    pub fn backward(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position -= rot * glam::Vec3::new(0.0, 0.0, -1.0) * Self::CAM_SPEED;
    }

    pub fn right(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position += rot * glam::Vec3::new(1.0, 0.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn left(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position -= rot * glam::Vec3::new(1.0, 0.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn up(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position += rot * glam::Vec3::new(0.0, 1.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn down(&mut self) {
        let rot = glam::Quat::from_euler(glam::EulerRot::ZYX, self.rotation.z, self.rotation.y, self.rotation.x);
        self.position -= rot * glam::Vec3::new(0.0, 1.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn resize(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}