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
//const OUTLINE: [char; 8] = ['|','/','-','\\','|','/','-','\\'];
const OUTLINE: [char; 8] = ['0','=','/','|','\\','=','6','7'];

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
				let pixel = sample(&pixels,width,height, x,y);
				let (edge_rot, is_edge) = if pixel.w > 0.0 { sobel(&pixels,width,height, x,y) } else { (0.0, false) };
				let r = pixel.x;
				let g = pixel.y;
				let b = pixel.z;
				let a = pixel.w;
				//let c = PALETTE[((luminance * 8.0) as u8).min(7) as usize];
				let mut c = PALETTE[(a * 7.0) as usize];
				if is_edge {
					c = OUTLINE[((edge_rot + 1.0)*3.5).clamp(0.0,7.0) as usize];
				}
				stdout.queue(style::PrintStyledContent(c.with(style::Color::Rgb { r: (r*255.0)as u8, g: (g*255.0)as u8, b: (b*255.0)as u8 })));
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

fn sample(pixels: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) -> Vec4 {
	let i = x.clamp(0,width-1) + y.clamp(0,height-1) * width;
	let pixel = &pixels[i*4..i*4+4];
	vec4(pixel[0] as f32 / 255.0, pixel[1] as f32 / 255.0, pixel[2] as f32 / 255.0, pixel[3] as f32 / 255.0)
}

fn sample_lumi(pixels: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) -> f32 {
	let pixel = sample(pixels,width,height, x,y);
	pixel.x*0.299 + pixel.y*0.587 + pixel.z*0.114
}

fn sobel_kernel(pixels: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) -> [f32; 9] {
	let mut n = [0.0; 9];
	n[0] = sample_lumi(pixels,width,height, x-1,y-1);
	n[1] = sample_lumi(pixels,width,height, x  ,y-1);
	n[2] = sample_lumi(pixels,width,height, x+1,y-1);
	n[3] = sample_lumi(pixels,width,height, x-1,y  );
	n[4] = sample_lumi(pixels,width,height, x  ,y  );
	n[5] = sample_lumi(pixels,width,height, x+1,y  );
	n[6] = sample_lumi(pixels,width,height, x-1,y+1);
	n[7] = sample_lumi(pixels,width,height, x  ,y+1);
	n[8] = sample_lumi(pixels,width,height, x+1,y+1);
	n
}

fn sobel(pixels: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) -> (f32,bool) {
	let n = sobel_kernel(pixels,width,height, x,y);
	let sobel_edge_h = n[2] + (2.0*n[5]) + n[8] - (n[0] + (2.0*n[3]) + n[6]);
	let sobel_edge_v = n[0] + (2.0*n[1]) + n[2] - (n[6] + (2.0*n[7]) + n[8]);
	//((sobel_edge_h * sobel_edge_h) + (sobel_edge_v * sobel_edge_v)).powf(0.5).clamp(vec4(0.0,0.0,0.0,0.0), vec4(1.0,1.0,1.0,1.0))
	//sobel_edge_v.clamp(vec4(0.0,0.0,0.0,0.0), vec4(1.0,1.0,1.0,1.0))
	//(sobel_edge_h + sobel_edge_v).normalize().clamp(vec4(0.0,0.0,0.0,0.0), vec4(1.0,1.0,1.0,1.0))
	let is_edge = if sobel_edge_h.abs() + sobel_edge_v.abs() > 0.75 { true } else { false };
	let h = sobel_edge_h;
	let v = sobel_edge_v;
	//let d = v.atan2(h) / 3.15;
	//let d = (d + 1.0);
	//let d = (d + 0.5) % 2.0;
	//let d = (d - 1.0);
	//let d = vec2(h,v).dot(vec2(0.0,1.0));
	let mut d = (v/h).atan() / 3.14;
	(d,is_edge)
}
