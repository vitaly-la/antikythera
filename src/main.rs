extern crate sdl2;

use astro::{Engine, Star};
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::f64::consts::PI;
use std::fs::read_to_string;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod astro;

const CANVAS_SIZE: u32 = 960;

fn read_stars(filename: &str) -> Vec<Star> {
    let mut stars = Vec::new();
    for line in read_to_string(filename).unwrap().lines() {
        let mut parts = line.split_whitespace();
        let hour = parts.next().unwrap().parse::<f64>().unwrap();
        let minute = parts.next().unwrap().parse::<f64>().unwrap();
        let declination = parts.next().unwrap().parse::<f64>().unwrap();
        let magnitude = parts.next().unwrap().parse::<f64>().unwrap();
        stars.push(Star {
            ascension: (hour * 60.0 + minute) / 24.0 / 60.0 * 2.0 * PI,
            declination: declination / 180.0 * PI,
            magnitude: magnitude,
        });
    }
    stars
}

fn get_now() -> f64 {
    let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs() as f64 + since_the_epoch.subsec_nanos() as f64 * 1e-9
}

fn horizontal_to_canvas(alt: f64, az: f64, size: u32) -> (i16, i16) {
    let r = 1.0 - alt * 2.0 / PI;
    let x = i16::try_from(size).unwrap() / 2 - (size as f64 / 2.0 * r * az.sin()).round() as i16;
    let y = i16::try_from(size).unwrap() / 2 - (size as f64 / 2.0 * r * az.cos()).round() as i16;
    (x, y)
}

fn magnitude_to_sz_bri(magnitude: f64) -> (i16, u8) {
    if magnitude < -1.0 {
        (5, 255)
    } else if magnitude < 0.0 {
        (4, 255)
    } else if magnitude < 1.0 {
        (3, 255)
    } else if magnitude < 2.0 {
        (2, 255)
    } else if magnitude < 3.0 {
        (1, 255)
    } else {
        (1, 127)
    }
}

fn main() {
    let stars = read_stars("stars.dat");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Antikythera", CANVAS_SIZE, CANVAS_SIZE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(5, 5, 5));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(5, 5, 5));
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
        let size = (CANVAS_SIZE / 2).try_into().unwrap();
        _ = canvas.filled_circle(size, size, size, Color::RGB(0, 0, 0));

        let engine = Engine::new(get_now());

        for star in &stars {
            let (alt, az) = engine.get_star_position(star);
            let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
            let (sz, bri) = magnitude_to_sz_bri(star.magnitude);
            _ = canvas.filled_circle(x, y, sz, Color::RGB(bri, bri, bri));
        }

        let (alt, az) = engine.get_sun_position();
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.filled_circle(x, y, 15, Color::RGB(255, 255, 255));

        let (alt, az) = engine.get_moon_position();
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.filled_circle(x, y, 15, Color::RGB(127, 127, 127));

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

        assert_eq!(horizontal_to_canvas(0.0, 0.0, 640), (320, 0));
        assert_eq!(horizontal_to_canvas(0.0, PI / 2.0, 640), (0, 320));
        assert_eq!(horizontal_to_canvas(0.0, PI, 640), (320, 640));
        assert_eq!(horizontal_to_canvas(0.0, 3.0 * PI / 2.0, 640), (640, 320));

        assert_eq!(horizontal_to_canvas(-PI / 2.0, 0.0, 640), (320, -320));
    }

    #[test]
    fn test_magnitude_to_sz_bri() {
        assert_eq!(magnitude_to_sz_bri(-1.5), (5, 255));
        assert_eq!(magnitude_to_sz_bri(-0.5), (4, 255));
        assert_eq!(magnitude_to_sz_bri(0.5), (3, 255));
        assert_eq!(magnitude_to_sz_bri(1.5), (2, 255));
        assert_eq!(magnitude_to_sz_bri(2.5), (1, 255));
        assert_eq!(magnitude_to_sz_bri(3.5), (1, 127));
    }
}
