use glam::*;

pub struct Camera {
	pub eye: Vec3,
	pub target: Vec3,
	pub up: Vec3,

	pub fov_y_deg: f32,
}

impl Camera {
	fn new() -> Self {
		Self {
			eye: vec3(0.0, 0.0, -1.0),
			target: vec3(0.0, 0.0, 0.0),
			up: vec3(0.0, 1.0, 0.0),

			fov_y_deg: 60.0,
		}
	}
}

pub struct CameraBuilder {
	camera: Camera,
}

impl CameraBuilder {
	fn new() -> Self {
		Self {
			camera: Camera::new(),
		}
	}

	pub fn build(self) -> Camera {
		self.camera
	}

	pub fn with_position(mut self, eye: Vec3) -> Self {
		self.camera.eye = eye;
		self
	}

	pub fn with_target(mut self, target: Vec3) -> Self {
		self.camera.target = target;
		self
	}
}

impl crate::Renderer {
	pub fn create_camera(&self) -> CameraBuilder {
		CameraBuilder::new()
	}
}
