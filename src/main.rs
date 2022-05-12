// Test file for marshmallow_fm

use marshmallow_fm::*;

const VS: &str = include_str!("../shaders/vs.glsl");
const FS: &str = include_str!("../shaders/fs.glsl");

fn main() {
	let mut renderer = Renderer::new("marshmallow_fm", (1280, 720));

	let mesh = renderer.create_mesh().triangle();
	let mut shader = renderer.compile_shader(VS, FS);
	let camera = renderer.create_camera().build();

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
	}
}
