use std::io::{stdout, Write};

use glam::*;

pub use luminance_surfman_offscreen::*;

use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::tess::{Tess, TessIndex};
use luminance::shader::{
	Program,
	Uniform,
	types::{Mat44 as UniMat4, Vec3 as UniVec3},
};
use luminance::UniformInterface;
use luminance::vertex::{Vertex, Semantics};
use luminance::render_state::RenderState;

use crossterm::{
	ExecutableCommand, QueueableCommand,
	execute,
	terminal::{self, size},
	event::{read, poll, Event, KeyCode},
	cursor,
	style::{self, Stylize},
};

const PALETTE: [char; 8] = [' ','.',':','=','o','x','O','X'];

pub enum WindowEvent {
	Quit,
	RequestRedraw,
}

#[derive(UniformInterface)]
pub struct ShaderInterface {
	mvp: Uniform<UniMat4<f32>>,
}

pub struct Backend {
	surface: SurfmanSurface,
	pub resolution: (usize, usize),
	target_ms: f32,
}

impl Backend {
	pub fn new(_win_title: &str, _win_size: (usize, usize)) -> Self {
		let (width, height) = size().expect("Failed to query terminal size!");
		let surface = SurfmanSurface::offscreen((width as usize, height as usize)).expect("Failed to create surface!");

		execute!(
			stdout(),
			terminal::EnterAlternateScreen,
		).expect("Failed to switch to alternate mode!");
		terminal::enable_raw_mode().expect("Failed to enable raw mode!");
		execute!(
			stdout(),
			cursor::Hide,
		).expect("Failed to hide cursor!");

		Self {
			surface: surface,
			resolution: (width as usize, height as usize),
			target_ms: 1000.0/60.0,
		}
	}

	pub fn poll_events(&mut self) -> Vec<WindowEvent> {
		// TODO: Handle multiple events within the target ms.
		// 	  Measure elapsed ms since the first poll and calculate the next poll length
		let mut events = Vec::new();
		if poll(std::time::Duration::from_secs_f32(self.target_ms / 1000.0)).expect("Failed to poll for events!") {
			match read().expect("Failed to read event!") {
				Event::Resize(w, h) => {
					self.resolution = (w as usize, h as usize);
					self.surface.set_size([w as u32, h as u32]).expect("Failed to resize offscreen surface!");
				},
				Event::Key(key) => {
					if key.code == KeyCode::Esc {
						return vec![WindowEvent::Quit]; // Just return quit immediately, who cares about other events
					}
				},
				_ => {}
			}
		}
		events.push(WindowEvent::RequestRedraw);
		events
	}

	pub fn render<V: Vertex, I: TessIndex, Sem: Semantics>(&mut self,
		calls: Vec<(Vec<(&Tess<LuminanceBackend, V, I>, Mat4)>,
			   &mut Program<LuminanceBackend, Sem, (), ShaderInterface>)>
	) {
		{
			let back_buffer = self.surface.back_buffer().expect("Failed to acquire back buffer!");
			let render = self.surface
				.new_pipeline_gate()
				.pipeline(
					&back_buffer,
					&PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 0.0]),
					|_, mut shd_gate| {
						for (meshes, program) in calls {
							shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
								for (m, mvp) in meshes {
									iface.set(&uni.mvp, UniMat4::new(mvp.to_cols_array_2d()));
									rdr_gate.render(&RenderState::default(), |mut tess_gate| {
										tess_gate.render(m)
									})?;
								}
								Ok(())
							})?;
						}
						Ok(())
					},
				).assume();
		}

		let (pixels, (width, height)) = self.surface.read_buffer();
		execute!(stdout(), cursor::MoveTo(0,0)).expect("Failed to move cursor!");
		let mut stdout = stdout();
		for y in 0..height {
			for x in 0..width {
				let i = x + y * width;
				let pixel = &pixels[i*4..i*4+4];
				let r = pixel[0];
				let g = pixel[1];
				let b = pixel[2];
				let a = pixel[3];
				let luminance = (r as f32 / 255.0)*0.299 + (g as f32 / 255.0)*0.587 + (b as f32 / 255.0)*0.114;
				//let c = PALETTE[((luminance * 8.0) as u8).min(7) as usize];
				let c = PALETTE[a as usize * 7 / 255];
				stdout.queue(style::PrintStyledContent(c.with(style::Color::Rgb { r: r, g: g, b: b })));
			}
		}
		stdout.flush();
	}
}

impl Drop for Backend {
	fn drop(&mut self) {
		execute!(
			stdout(),
			terminal::LeaveAlternateScreen
		).expect("Failed to switch terminal back to regular mode!");
		terminal::disable_raw_mode().expect("Failed to disable raw mode!");
		execute!(
			stdout(),
			cursor::Show,
		).expect("Failed to show cursor!");
	}
}

impl std::ops::Deref for Backend {
	type Target = SurfmanSurface;
	fn deref(&self) -> &Self::Target {
		&self.surface
	}
}

impl std::ops::DerefMut for Backend {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.surface
	}
}

