extern crate sdl2;

use std::cmp::min;
use std::f64::consts::PI;
use std::fs::read_to_string;

use astro::Engine;
use chrono::Utc;
use sdl2::event::{Event, WindowEvent};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf;
use sdl2::ttf::Font;
use sdl2::video::Window;

mod astro;

pub struct Star {
    name: Option<String>,
    ascension: f64,
    declination: f64,
    magnitude: f64,
}

pub struct Planet<'a> {
    name: String,
    semimajor: f64,
    sidereal: f64,
    phase: f64,
    texture: Option<Texture<'a>>,
}

struct Step {
    name: &'static str,
    value: i32,
}

enum Mode {
    Default,
    SetLatitude,
    SetLongitude,
}

const LAT: f64 = 51.4769 / 180.0 * PI; // greenwich
const LON: f64 = 0.0; // greenwich
const INITIAL_SIZE: u32 = 960;
const PANEL_SIZE: u32 = 30;
const STEPS: [Step; 11] = [
    Step {
        name: "-1 month",
        value: -2360584,
    },
    Step {
        name: "-1 day",
        value: -86164,
    },
    Step {
        name: "-6 hours",
        value: -21600,
    },
    Step {
        name: "-1 hour",
        value: -3600,
    },
    Step {
        name: "-10 minutes",
        value: -600,
    },
    Step {
        name: "1 second",
        value: 1,
    },
    Step {
        name: "10 minutes",
        value: 600,
    },
    Step {
        name: "1 hour",
        value: 3600,
    },
    Step {
        name: "6 hours",
        value: 21600,
    },
    Step {
        name: "1 day",
        value: 86164,
    },
    Step {
        name: "1 month",
        value: 2360584,
    },
];

fn read_stars(filename: &str) -> Vec<Star> {
    let mut stars = Vec::new();
    for line in read_to_string(filename).expect("Couldn't find stars.dat").lines() {
        let mut parts = line.split_whitespace();
        let hour = parts.next().unwrap().parse::<f64>().unwrap();
        let minute = parts.next().unwrap().parse::<f64>().unwrap();
        let declination = parts.next().unwrap().parse::<f64>().unwrap();
        let magnitude = parts.next().unwrap().parse::<f64>().unwrap();
        let name = parts.next().unwrap();
        let name = match name {
            "null" => None,
            _ => Some(name.to_string()),
        };
        stars.push(Star {
            name,
            ascension: (hour * 60.0 + minute) / 24.0 / 60.0 * 2.0 * PI,
            declination: declination / 180.0 * PI,
            magnitude,
        });
    }
    stars
}

fn read_planets<'a, T>(texture_creator: &'a TextureCreator<T>, filename: &'a str) -> Vec<Planet<'a>> {
    let mut planets = Vec::new();
    for line in read_to_string(filename).expect("Couldn't find planets.dat").lines() {
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap().to_string();
        let semimajor = parts.next().unwrap().parse::<f64>().unwrap();
        let sidereal = parts.next().unwrap().parse::<f64>().unwrap();
        let phase = parts.next().unwrap().parse::<f64>().unwrap();
        let texture = parts.next().unwrap();
        planets.push(Planet {
            name,
            semimajor,
            sidereal,
            phase,
            texture: match texture {
                "null" => None,
                _ => Some(
                    texture_creator
                        .load_texture(texture)
                        .unwrap_or_else(|_| panic!("Couldn't find {}", texture)),
                ),
            },
        });
    }
    planets
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

fn horizontal_to_canvas(alt: f64, az: f64, size: (u32, u32)) -> (i16, i16) {
    let r = 1.0 - alt * 2.0 / PI;
    let msize = min(size.0, size.1 - PANEL_SIZE);
    let x = i16::try_from(size.0).unwrap() / 2 - (msize as f64 / 2.0 * r * az.sin()).round() as i16;
    let y = i16::try_from(size.1 - PANEL_SIZE).unwrap() / 2 - (msize as f64 / 2.0 * r * az.cos()).round() as i16;
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

trait Astro {
    fn text(&mut self, text: &str, font: &Font, x: i16, y: i16, obj_size: i16);
    fn aa_filled_circle(&mut self, x: i16, y: i16, rad: i16, color: Color);
}

impl Astro for Canvas<Window> {
    fn text(&mut self, text: &str, font: &Font, x: i16, y: i16, obj_size: i16) {
        let texture_creator = self.texture_creator();
        let (texture, xsize, ysize) = render_text(font, &texture_creator, text);
        self.copy(
            &texture,
            None,
            Rect::new(
                (x - i16::try_from(xsize / 2).unwrap()).into(),
                (y - obj_size - i16::try_from(ysize).unwrap()).into(),
                xsize,
                ysize,
            ),
        )
        .unwrap();
    }

    fn aa_filled_circle(&mut self, x: i16, y: i16, rad: i16, color: Color) {
        self.filled_circle(x, y, rad, color).unwrap();
        if rad >= 2 {
            self.aa_circle(x, y, rad, color).unwrap();
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Antikythera", INITIAL_SIZE, INITIAL_SIZE + PANEL_SIZE)
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let stars = read_stars("stars.dat");
    let texture_creator = canvas.texture_creator();
    let moon_phases = load_moon_phases(&texture_creator);
    let planets = read_planets(&texture_creator, "planets.dat");
    let ttf_context = ttf::init().unwrap();
    let font = ttf_context
        .load_font("NotoSansMono-Light.ttf", 20)
        .expect("Couldn't find NotoSansMono-Light.ttf");
    let small_font = ttf_context
        .load_font("NotoSansMono-Light.ttf", 14)
        .expect("Couldn't find NotoSansMono-Light.ttf");

    canvas
        .set_logical_size(INITIAL_SIZE, INITIAL_SIZE + PANEL_SIZE)
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut real_time = Utc::now();
    let mut current_time = real_time;
    let mut step = 5;
    let mut mode = Mode::Default;
    let mut latitude = LAT;
    let mut longitude = LON;
    let mut buffer = String::new();

    'running: loop {
        canvas.set_draw_color(Color::RGB(12, 12, 12));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    canvas.set_logical_size(width as u32, height as u32).unwrap();
                },
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode), ..
                } => match mode {
                    Mode::Default => match keycode {
                        Keycode::Left => {
                            step = if step > 0 { step - 1 } else { step };
                        },
                        Keycode::Right => {
                            step = if step < STEPS.len() - 1 { step + 1 } else { step };
                        },
                        Keycode::A => {
                            mode = Mode::SetLatitude;
                            buffer = String::new();
                        },
                        Keycode::O => {
                            mode = Mode::SetLongitude;
                            buffer = String::new();
                        },
                        _ => {},
                    },
                    Mode::SetLatitude => match keycode {
                        Keycode::Return => {
                            if let Ok(new_latitude) = buffer.parse::<f64>() {
                                if (-90.0..=90.0).contains(&new_latitude) {
                                    latitude = new_latitude / 180.0 * PI
                                }
                            }
                            mode = Mode::Default;
                        },
                        Keycode::Escape => {
                            buffer = String::new();
                            mode = Mode::Default;
                        },
                        _ => {
                            buffer.push_str(&keycode.to_string());
                        },
                    },
                    Mode::SetLongitude => match keycode {
                        Keycode::Return => {
                            if let Ok(new_longitude) = buffer.parse::<f64>() {
                                if (0.0..=360.0).contains(&new_longitude) {
                                    longitude = new_longitude / 180.0 * PI
                                }
                            }
                            mode = Mode::Default;
                        },
                        Keycode::Escape => {
                            buffer = String::new();
                            mode = Mode::Default;
                        },
                        _ => {
                            buffer.push_str(&keycode.to_string());
                        },
                    },
                },
                _ => {},
            }
        }

        let now = Utc::now();
        let elapsed = now - real_time;
        real_time = now;
        current_time += elapsed * STEPS[step].value;
        let engine = Engine::new(current_time, latitude, longitude);

        let (width, height) = canvas.logical_size();
        let radius = min(width, height - PANEL_SIZE) / 2;
        canvas.aa_filled_circle(
            (width / 2).try_into().unwrap(),
            ((height - PANEL_SIZE) / 2).try_into().unwrap(),
            radius.try_into().unwrap(),
            Color::RGB(0, 0, 0),
        );

        for star in &stars {
            let (alt, az) = engine.get_star_position(star);
            let (x, y) = horizontal_to_canvas(alt, az, canvas.logical_size());
            let (size, brightness) = magnitude_to_size_and_brightness(star.magnitude);
            match size {
                0 => canvas
                    .pixel(x, y, Color::RGB(brightness, brightness, brightness))
                    .unwrap(),
                _ => canvas.aa_filled_circle(x, y, size, Color::RGB(brightness, brightness, brightness)),
            }
            if let Some(name) = &star.name {
                canvas.text(name, &small_font, x, y, 5)
            }
        }

        let (alt, az) = engine.get_sun_position();
        let (x, y) = horizontal_to_canvas(alt, az, canvas.logical_size());
        canvas.aa_filled_circle(x, y, 15, Color::RGB(255, 255, 255));
        canvas.text("Sun", &small_font, x, y, 15);

        for planet in &planets {
            let (alt, az) = engine.get_planet_position(planet);
            let (x, y) = horizontal_to_canvas(alt, az, canvas.logical_size());
            match planet.texture {
                Some(_) => canvas
                    .copy(
                        planet.texture.as_ref().unwrap(),
                        None,
                        Rect::new((x - 8).into(), (y - 8).into(), 16, 16),
                    )
                    .unwrap(),
                None => canvas.aa_filled_circle(x, y, 8, Color::RGB(255, 255, 255)),
            }
            canvas.text(&planet.name, &small_font, x, y, 10);
        }

        let (alt, az, phase, angle) = engine.get_moon_position();
        let (x, y) = horizontal_to_canvas(alt, az, canvas.logical_size());
        canvas
            .copy_ex(
                &moon_phases[(phase / 2.0 / PI * 24.0).round() as usize % 24],
                None,
                Rect::new((x - 15).into(), (y - 15).into(), 30, 30),
                angle / PI * 180.0,
                None,
                false,
                false,
            )
            .unwrap();
        canvas.text("Moon", &small_font, x, y, 15);

        canvas.text(
            "N",
            &font,
            (width / 2).try_into().unwrap(),
            ((height - PANEL_SIZE) / 2 - radius + 25).try_into().unwrap(),
            0,
        );
        canvas.text(
            "E",
            &font,
            (width / 2 - radius + 10).try_into().unwrap(),
            ((height - PANEL_SIZE) / 2 + 10).try_into().unwrap(),
            0,
        );
        canvas.text(
            "S",
            &font,
            (width / 2).try_into().unwrap(),
            ((height - PANEL_SIZE) / 2 + radius).try_into().unwrap(),
            0,
        );
        canvas.text(
            "W",
            &font,
            (width / 2 + radius - 10).try_into().unwrap(),
            ((height - PANEL_SIZE) / 2 + 10).try_into().unwrap(),
            0,
        );

        canvas
            .box_(
                0,
                (height - PANEL_SIZE).try_into().unwrap(),
                width.try_into().unwrap(),
                height.try_into().unwrap(),
                Color::RGB(0, 0, 0),
            )
            .unwrap();
        let text = match mode {
            Mode::Default => {
                format!(
                    "lat: {:.4}; lon: {:.4}; {}; Step: {}",
                    latitude / PI * 180.0,
                    longitude / PI * 180.0,
                    engine.time.format("%Y-%b-%d %H:%M:%S %Z"),
                    STEPS[step].name
                )
            },
            Mode::SetLatitude => {
                format!("Set latitude: {}", buffer)
            },
            Mode::SetLongitude => {
                format!("Set longitude: {}", buffer)
            },
        };
        let (texture, x, y) = render_text(&font, &texture_creator, &text);
        canvas
            .copy(
                &texture,
                None,
                Rect::new(10, (height - PANEL_SIZE).try_into().unwrap(), x, y),
            )
            .unwrap();

        canvas.present();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal_to_canvas() {
        assert_eq!(horizontal_to_canvas(PI / 2.0, 0.0, (640, 670)), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, PI / 2.0, (640, 670)), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, PI, (640, 670)), (320, 320));
        assert_eq!(horizontal_to_canvas(PI / 2.0, 3.0 * PI / 2.0, (640, 670)), (320, 320));

        assert_eq!(horizontal_to_canvas(0.0, 0.0, (640, 670)), (320, 0));
        assert_eq!(horizontal_to_canvas(0.0, PI / 2.0, (640, 670)), (0, 320));
        assert_eq!(horizontal_to_canvas(0.0, PI, (640, 670)), (320, 640));
        assert_eq!(horizontal_to_canvas(0.0, 3.0 * PI / 2.0, (640, 670)), (640, 320));

        assert_eq!(horizontal_to_canvas(-PI / 2.0, 0.0, (640, 670)), (320, -320));
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
