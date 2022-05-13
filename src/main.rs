// Test file for marshmallow_fm

use std::time::Instant;

use glam::*;

use marshmallow_fm::*;

const VS: &str = include_str!("../shaders/vs.glsl");
const FS: &str = include_str!("../shaders/fs.glsl");

fn main() {
	let mut renderer = Renderer::new("marshmallow_fm", (1280, 720));

	let start = Instant::now();

	let mut mesh = renderer.create_mesh().triangle();
	let mut shader = renderer.compile_shader(VS, FS);
	let camera = renderer.create_camera().with_projection(camera::Projection::Orthographic(2.0)).build();

	'app: loop {
		let events = renderer.get_events();
		for event in &events {
			match event {
				WindowEvent::Quit => break 'app,
				WindowEvent::RequestRedraw => {
					let mut frame = renderer.begin_frame(&camera);
					frame.draw_with_shader(&mut shader, vec![&mesh]);
					frame.finish();
				},
				_ => {},
			}
		}

		let t = start.elapsed().as_secs_f32();
		mesh.transform.rotation = Quat::from_rotation_z((t*0.5) % 360.0);
	}
}
