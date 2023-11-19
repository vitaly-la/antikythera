extern crate sdl2;

use astro::get_sun_position;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::f64::consts::PI;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod astro;

const CANVAS_SIZE: u32 = 640;

fn horizontal_to_canvas(alt: f64, az: f64, size: u32) -> (i16, i16) {
    let r = 1.0 - alt * 2.0 / PI;
    let x = i16::try_from(size).unwrap() / 2 + (size as f64 / 2.0 * r * az.sin()).round() as i16;
    let y = i16::try_from(size).unwrap() / 2 + (size as f64 / 2.0 * r * az.cos()).round() as i16;
    (x, y)
}

fn now() -> f64 {
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs() as f64 + since_the_epoch.subsec_nanos() as f64 * 1e-9
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Antikythera", CANVAS_SIZE, CANVAS_SIZE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(127, 127, 127));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(127, 127, 127));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        _ = canvas.filled_circle(320, 320, 320, Color::RGB(0, 0, 0));
        let (alt, az) = get_sun_position(now());
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.filled_circle(x, y, 10, Color::RGB(255, 255, 255));

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_to_canvas() {
        assert_eq!(horizontal_to_canvas(PI / 2.0, 0.0, 640), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, PI / 2.0, 640), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, PI, 640), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, 3.0 * PI / 2.0, 640), (320, 320));

        assert_eq!(horizontal_to_canvas(0.0, 0.0, 640), (320, 640));
        assert_eq!(horizontal_to_canvas(0.0, PI / 2.0, 640), (640, 320));
        assert_eq!(horizontal_to_canvas(0.0, PI, 640), (320, 0));
        assert_eq!(horizontal_to_canvas(0.0, 3.0 * PI / 2.0, 640), (0, 320));

        assert_eq!(horizontal_to_canvas(-PI / 2.0, 0.0, 640), (320, 960));
    }
}
