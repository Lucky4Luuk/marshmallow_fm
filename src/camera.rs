use glam::*;

pub enum Projection {
	/// Fov in degrees
	Perspective(f32),
	/// Size
	Orthographic(f32),
}

impl Projection {
	// TODO: Don't hardcode near/far values
	fn mat(&self, aspect_ratio: f32) -> Mat4 {
		match self {
			Self::Perspective(fov_y_deg) => Mat4::perspective_rh_gl(fov_y_deg * std::f32::consts::PI / 180.0, aspect_ratio, 0.02, 1024.0),
			Self::Orthographic(scale) => {
				let half_scale = scale / 2.0;
				Mat4::orthographic_rh_gl(-half_scale, half_scale, -half_scale, half_scale, 0.0, 1024.0)
			}
		}
	}
}

pub struct Camera {
	pub eye: Vec3,
	pub target: Vec3,
	pub up: Vec3,

	pub projection: Projection,
}

impl Camera {
	fn new() -> Self {
		Self {
			eye: vec3(0.0, 0.0, -1.0),
			target: vec3(0.0, 0.0, 0.0),
			up: vec3(0.0, 1.0, 0.0),

			projection: Projection::Orthographic(1.0),
		}
	}

	pub(crate) fn view_proj(&self, aspect_ratio: f32) -> Mat4 {
		Mat4::look_at_rh(self.eye, self.target, self.up) * self.projection.mat(aspect_ratio)
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

	pub fn with_up(mut self, up: Vec3) -> Self {
		self.camera.up = up;
		self
	}

	pub fn with_pos_target_up(self, eye: Vec3, target: Vec3, up: Vec3) -> Self {
		self.with_position(eye).with_target(target).with_up(up)
	}

	pub fn with_projection(mut self, projection: Projection) -> Self {
		self.camera.projection = projection;
		self
	}
}

impl crate::Renderer {
	pub fn create_camera(&self) -> CameraBuilder {
		CameraBuilder::new()
	}
}
