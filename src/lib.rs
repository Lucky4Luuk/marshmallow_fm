#[macro_use] extern crate log;

use glam::*;

use luminance::shader::Program;
use luminance::tess::{Tess, TessIndex};

use gpu_terminal::Backend;
pub use gpu_terminal::WindowEvent;
pub(crate) use gpu_terminal::LuminanceBackend;
pub(crate) use gpu_terminal::ShaderInterface;

pub mod mesh;
pub mod shader;
pub mod camera;

pub struct Renderer {
	pub(crate) backend: Backend,
}

impl Renderer {
	pub fn new(win_title: &str, win_size: (usize, usize)) -> Self {
		Self {
			backend: Backend::new(win_title, win_size),
		}
	}

	pub fn get_events(&mut self) -> Vec<WindowEvent> {
		self.backend.poll_events() // TODO: Perhaps handle events the renderer specifically needs?
	}

	pub fn size(&self) -> (usize, usize) {
		self.backend.resolution
	}

	pub fn begin_frame<'frm>(&'frm mut self, camera: &'frm camera::Camera) -> Frame<'frm> {
		let (width, height) = self.backend.resolution;
		let aspect_ratio = width as f32 / height as f32;
		Frame {
			renderer: self,
			consumed: false,
			camera: camera,
			aspect_ratio: aspect_ratio,
			calls: Vec::new(),
		}
	}
}

pub struct Frame<'rnd> {
	renderer: &'rnd mut Renderer,
	consumed: bool,
	camera: &'rnd camera::Camera,
	aspect_ratio: f32,
	calls: Vec<(Vec<&'rnd mesh::Mesh>, &'rnd mut shader::Shader)>,
}

impl<'rnd> Frame<'rnd> {
	pub fn draw_with_shader(&mut self, shader: &'rnd mut shader::Shader, meshes: Vec<&'rnd mesh::Mesh>) {
		self.calls.push((meshes, shader));
	}

	pub fn finish(mut self) {
		let vp = self.camera.view_proj(self.aspect_ratio);
		let meshes_conv: Vec<(Vec<(&Tess<_,_,_>, Mat4)>, &mut Program<_,_,_,_>)> = {
			self.calls.iter_mut().map(|(meshes, shader)| {
				let tesses: Vec<(&Tess<_,_,_>, Mat4)> = meshes.iter().map(|m| (&m.tess, vp * m.transform.mat())).collect();
				(tesses, &mut shader.program)
			}).collect()
		};
		self.renderer.backend.render(meshes_conv);
		self.consumed = true;
	}
}

impl<'rnd> Drop for Frame<'rnd> {
	fn drop(&mut self) {
		if !self.consumed {
			error!("Dropping frame before finishing it!");
		}
	}
}
