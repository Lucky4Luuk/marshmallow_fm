use luminance::{Semantics, Vertex};
use luminance::context::GraphicsContext;
use luminance::tess::{Tess, Mode, TessIndex};

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
	#[sem(name = "pos", repr = "[f32; 3]", wrapper = "VertexPosition")]
	Position,
	#[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexRgb")]
	Color
}

#[derive(Copy, Clone, Debug, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
	#[allow(dead_code)]
	position: VertexPosition,

	#[allow(dead_code)]
	#[vertex(normalized = "true")]
	color: VertexRgb,
}

const TRI_VERTICES: [Vertex; 3] = [
    Vertex::new(
    	VertexPosition::new([-0.5, -0.5, 0.0]),
    	VertexRgb::new([255, 0, 0]),
    ),
    Vertex::new(
    	VertexPosition::new([0.5, -0.5, 0.0]),
    	VertexRgb::new([0, 255, 0]),
    ),
    Vertex::new(
    	VertexPosition::new([0.0, 0.5, 0.0]),
    	VertexRgb::new([0, 0, 255])
    ),
];

pub struct Mesh {
	pub(crate) tess: Tess<crate::LuminanceBackend, Vertex, u32>,
}

impl Mesh {
	fn from_vertices_indices(backend: &mut crate::Backend, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
		let tess = backend
			.new_tess()
			.set_vertices(&vertices[..])
			.set_indices(&indices[..])
			.set_mode(Mode::Triangle)
			.build().expect("Failed to construct mesh tesselation!");
		Self {
			tess: tess,
		}
	}

	fn from_vertices(backend: &mut crate::Backend, vertices: Vec<Vertex>) -> Self {
		let indices: Vec<u32> = (0..vertices.len()).into_iter().rev().map(|i| i as u32).collect();
		Self::from_vertices_indices(backend, vertices, indices)
	}
}

pub struct MeshBuilder<'b> {
	backend: &'b mut crate::Backend,
}

impl<'b> MeshBuilder<'b> {
	pub fn triangle(&mut self) -> Mesh {
		Mesh::from_vertices(&mut self.backend, TRI_VERTICES.to_vec())
	}
}

impl crate::Renderer {
	pub fn create_mesh(&mut self) -> MeshBuilder {
		MeshBuilder {
			backend: &mut self.backend,
		}
	}
}
