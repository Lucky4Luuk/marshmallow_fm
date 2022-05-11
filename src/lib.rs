#[macro_use] extern crate log;

use glam::*;

use luminance::shader::Program;
use luminance::tess::{Tess, TessIndex};

use gpu_terminal::Backend;
pub use gpu_terminal::WindowEvent;
pub(crate) use gpu_terminal::LuminanceBackend;

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

	pub fn begin_frame(&mut self) -> Frame {
		Frame {
			renderer: self,
			consumed: false,
			calls: Vec::new(),
		}
	}
}

pub struct Frame<'rnd> {
	renderer: &'rnd mut Renderer,
	consumed: bool,
	calls: Vec<(Vec<&'rnd mesh::Mesh>, &'rnd mut shader::Shader)>,
}

impl<'rnd> Frame<'rnd> {
	pub fn draw_with_shader(&mut self, shader: &'rnd mut shader::Shader, meshes: Vec<&'rnd mesh::Mesh>) {
		self.calls.push((meshes, shader));
	}

	pub fn finish(mut self) {
		let meshes_conv: Vec<(Vec<&Tess<_,_,_>>, &mut Program<_,_,_,_>)> = {
			self.calls.iter_mut().map(|(meshes, shader)| {
				let tesses: Vec<&Tess<_,_,_>> = meshes.iter().map(|m| &m.tess).collect();
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
