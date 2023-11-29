extern crate sdl2;

use std::f64::consts::PI;
use std::fs::read_to_string;
use std::time::Duration;

use astro::{Engine, Star, LAT, LON};
use chrono::Utc;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf;
use sdl2::ttf::Font;

mod astro;

const CANVAS_SIZE: u32 = 960;

fn read_stars(filename: &str) -> Vec<Star> {
    let mut stars = Vec::new();
    for line in read_to_string(filename).expect("Couldn't find stars.dat").lines() {
        let mut parts = line.split_whitespace();
        let hour = parts.next().unwrap().parse::<f64>().unwrap();
        let minute = parts.next().unwrap().parse::<f64>().unwrap();
        let declination = parts.next().unwrap().parse::<f64>().unwrap();
        let magnitude = parts.next().unwrap().parse::<f64>().unwrap();
        stars.push(Star {
            ascension: (hour * 60.0 + minute) / 24.0 / 60.0 * 2.0 * PI,
            declination: declination / 180.0 * PI,
            magnitude,
        });
    }
    stars
}

fn load_moon_phases<T>(texture_creator: &TextureCreator<T>) -> Vec<Texture> {
    let mut moon_phases = Vec::new();
    for i in 0..24 {
        moon_phases.push(
            texture_creator
                .load_texture(format!("moon_phases/{:02}.png", i).as_str())
                .expect("Couldn't find textures in moon_phases/"),
        );
    }
    moon_phases
}

fn horizontal_to_canvas(alt: f64, az: f64, size: u32) -> (i16, i16) {
    let r = 1.0 - alt * 2.0 / PI;
    let x = i16::try_from(size).unwrap() / 2 - (size as f64 / 2.0 * r * az.sin()).round() as i16;
    let y = i16::try_from(size).unwrap() / 2 - (size as f64 / 2.0 * r * az.cos()).round() as i16;
    (x, y)
}

fn magnitude_to_size_and_brightness(magnitude: f64) -> (i16, u8) {
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
        (0, 255)
    }
}

fn render_text<'a, T>(
    font: &'a Font<'a, 'a>,
    texture_creator: &'a TextureCreator<T>,
    text: &'a str,
) -> (Texture<'a>, u32, u32) {
    let texture = font
        .render(text)
        .blended(Color::RGB(0, 255, 0))
        .unwrap()
        .as_texture(texture_creator)
        .unwrap();
    let (x, y) = font.size_of(text).unwrap();
    (texture, x, y)
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Antikythera", CANVAS_SIZE, CANVAS_SIZE + 50)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let stars = read_stars("stars.dat");
    let texture_creator = canvas.texture_creator();
    let moon_phases = load_moon_phases(&texture_creator);
    let planet = texture_creator
        .load_texture("jupiter.png")
        .expect("Couldn't find jupiter.png");
    let ttf_context = ttf::init().unwrap();
    let font = ttf_context
        .load_font("NotoSansMono-Light.ttf", 24)
        .expect("Couldn't find NotoSansMono-Light.ttf");

    canvas.set_draw_color(Color::RGB(12, 12, 12));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(12, 12, 12));
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

        let size = (CANVAS_SIZE / 2).try_into().unwrap();
        _ = canvas.filled_circle(size, size, size, Color::RGB(0, 0, 0));

        let engine = Engine::new(Utc::now());

        for star in &stars {
            let (alt, az) = engine.get_star_position(star);
            let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
            let (size, brightness) = magnitude_to_size_and_brightness(star.magnitude);
            _ = match size {
                0 => canvas.pixel(x, y, Color::RGB(brightness, brightness, brightness)),
                _ => canvas.filled_circle(x, y, size, Color::RGB(brightness, brightness, brightness)),
            }
        }

        let (alt, az) = engine.get_sun_position();
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.filled_circle(x, y, 15, Color::RGB(255, 255, 255));

        let (alt, az, phase, angle) = engine.get_moon_position();
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.copy_ex(
            &moon_phases[(phase / 2.0 / PI * 24.0).round() as usize % 24],
            None,
            Rect::new((x - 15).into(), (y - 15).into(), 30, 30),
            angle / PI * 180.0,
            None,
            false,
            false,
        );

        let (alt, az) = engine.get_planet_position();
        let (x, y) = horizontal_to_canvas(alt, az, CANVAS_SIZE);
        _ = canvas.copy(&planet, None, Rect::new((x - 10).into(), (y - 10).into(), 20, 20));

        _ = canvas.box_(0, 960, 960, 1060, Color::RGB(0, 0, 0));

        let text = format!(
            "lat: {:.4}; lon: {:.4}; {}",
            LAT / PI * 180.0,
            LON / PI * 180.0,
            engine.time.format("%Y-%b-%d %H:%M:%S %Z")
        );
        let (texture, x, y) = render_text(&font, &texture_creator, &text);
        _ = canvas.copy(&texture, None, Rect::new(10, 967, x, y));

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
    fn test_magnitude_to_size_and_brightness() {
        assert_eq!(magnitude_to_size_and_brightness(-1.5), (5, 255));
        assert_eq!(magnitude_to_size_and_brightness(-0.5), (4, 255));
        assert_eq!(magnitude_to_size_and_brightness(0.5), (3, 255));
        assert_eq!(magnitude_to_size_and_brightness(1.5), (2, 255));
        assert_eq!(magnitude_to_size_and_brightness(2.5), (1, 255));
        assert_eq!(magnitude_to_size_and_brightness(3.5), (0, 255));
    }
}
