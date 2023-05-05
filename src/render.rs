extern crate sdl2; 
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect};
use std::time::Duration;
use sdl2::rect::Point;

const HORIZONTAL_TILES: u32 = 32;
const VERTICAL_TILES: u32 = 30;
const TILE_WIDTH: u32 = 10;
const TILE_HEIGHT: u32 = 10;

pub fn sdl2_setup() {
	let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("NES Emulator - by Shlomi Domnenko", 800, 800)
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

		let tile_width: u32 = win_width / TILE_WIDTH;
		let tile_height: u32 = win_height / TILE_HEIGHT;

		// Loop over tiles
		for y in 0..VERTICAL_TILES {
			for x in 0..HORIZONTAL_TILES {
				let tile_x = x * tile_width;
				let tile_y = y * tile_height;
				let rect = Rect::new(tile_x as i32, tile_y as i32, tile_width as u32, tile_height as u32);

				canvas.set_draw_color(Color::RGB(100, 100, 100));
				canvas.fill_rect(rect).unwrap();
				canvas.set_draw_color(Color::RGB(230, 230, 230));
				canvas.draw_rect(rect).unwrap();

				// Loop over pixels per tile
				canvas.set_draw_color(Color::RGB(0, 200, 0));
				for pyi in 0..8 {
					for pxi in 0..8 {
						// Draw horizontal lines
						let px: i32 = (tile_x + (tile_width / 8) * pxi) as i32;
						let p1 = Point::new(px, tile_y as i32);
						let p2 = Point::new(px, (tile_y + tile_height) as i32);
						canvas.draw_line(p1, p2).unwrap();

						// Draw vertical lines
						let py: i32 = (tile_y + (tile_height / 8) * pyi) as i32;
						let p1 = Point::new(tile_x as i32, py);
						let p2 = Point::new((tile_x + tile_width) as i32, py);
						canvas.draw_line(p1, p2).unwrap();
					}
				}
			}
		}

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}