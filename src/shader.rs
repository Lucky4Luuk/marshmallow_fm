use luminance::shader::Program;
use luminance::context::GraphicsContext;

use crate::mesh::VertexSemantics;

pub struct Shader {
	pub(crate) program: Program<crate::LuminanceBackend, VertexSemantics, (), ()>,
}

impl crate::Renderer {
	pub fn compile_shader(&mut self, vs: &str, fs: &str) -> Shader {
		let program = self.backend
			.new_shader_program::<VertexSemantics, (), ()>()
			.from_strings(vs, None, None, fs)
			.expect("Failed to compile shader!")
			.ignore_warnings();
		Shader {
			program: program,
		}
	}
}
