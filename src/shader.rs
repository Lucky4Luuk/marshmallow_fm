use luminance::shader::Program;
use luminance::context::GraphicsContext;
use luminance::UniformInterface;

use crate::mesh::VertexSemantics;

use crate::ShaderInterface;

pub struct Shader {
	pub(crate) program: Program<crate::LuminanceBackend, VertexSemantics, (), ShaderInterface>,
}

impl crate::Renderer {
	pub fn compile_shader(&mut self, vs: &str, fs: &str) -> Shader {
		let program = self.backend
			.new_shader_program::<VertexSemantics, (), ShaderInterface>()
			.from_strings(vs, None, None, fs)
			.expect("Failed to compile shader!")
			.ignore_warnings();
		Shader {
			program: program,
		}
	}
}
