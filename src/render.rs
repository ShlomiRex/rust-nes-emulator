extern crate sdl2; 
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect};
use std::time::Duration;

const WIDTH: u32 = 10;
const HEIGHT: u32 = 10;

pub fn sdl2_setup() {
	let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
		.resizable()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
 
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;

	let (mut win_width, mut win_height) = canvas.window_mut().size();

    'running: loop {
        i = (i + 1) % 255;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
				Event::Window {..} => {
					(win_width, win_height) = canvas.window_mut().size();
					//println!("Window size changed");
				}
                _ => {}
            }
        }

		let pixel_size_width: u32 = win_width / WIDTH;
		let pixel_size_height: u32 = win_height / HEIGHT;
		for y in 0..HEIGHT {
			for x in 0..WIDTH {
				let pixel_x = x * pixel_size_width;
				let pixel_y = y * pixel_size_height;
				let rect = Rect::new(pixel_x as i32, pixel_y as i32, pixel_size_width as u32, pixel_size_height as u32);

				canvas.set_draw_color(Color::RGB(200, 0, 0));
				canvas.fill_rect(rect).unwrap();
				canvas.set_draw_color(Color::RGB(0, 0, 200));
				canvas.draw_rect(rect).unwrap();
			}
		}

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}