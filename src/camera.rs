pub struct WorldCamera {
    position: glam::Vec3,
    rotation: glam::Vec3, // (yaw, pitch, roll)
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
            fovy: 90f32.to_radians(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection(&self) -> glam::Mat4 {
        let rotation_matrix = glam::Mat4::from_euler(
            glam::EulerRot::ZXY,
            -self.rotation.z,
            -self.rotation.y,
            -self.rotation.x,
        );
        let translation_matrix = glam::Mat4::from_translation(-self.position);

        let view = rotation_matrix * translation_matrix;
        let proj = glam::Mat4::perspective_lh(self.fovy, self.aspect, self.z_near, self.z_far);

        proj * view
    }

    pub fn get_rotation_quat(&self) -> glam::Quat {
        glam::Quat::from_euler(
            glam::EulerRot::YXZ,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
        )
    }

    pub fn forward(&mut self) {
        self.position +=
            self.get_rotation_quat() * glam::Vec3::new(0.0, 0.0, 1.0) * Self::CAM_SPEED;
    }

    pub fn backward(&mut self) {
        self.position -=
            self.get_rotation_quat() * glam::Vec3::new(0.0, 0.0, 1.0) * Self::CAM_SPEED;
    }

    pub fn right(&mut self) {
        self.position +=
            self.get_rotation_quat() * glam::Vec3::new(1.0, 0.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn left(&mut self) {
        self.position -=
            self.get_rotation_quat() * glam::Vec3::new(1.0, 0.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn up(&mut self) {
        self.position +=
            self.get_rotation_quat() * glam::Vec3::new(0.0, 1.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn down(&mut self) {
        self.position -=
            self.get_rotation_quat() * glam::Vec3::new(0.0, 1.0, 0.0) * Self::CAM_SPEED;
    }

    pub fn mouse_moved(&mut self, delta: egui::Vec2) {
        const MOUSE_SENSETIVITY: f32 = 0.002;

        self.rotation.x -= delta.x * MOUSE_SENSETIVITY;
        self.rotation.y += delta.y * MOUSE_SENSETIVITY;
    }

    pub fn resize(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
