use std::f64::consts::PI;

use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::{horizontal_to_canvas, PANEL_SIZE};
pub trait Painter {
    fn text(&mut self, text: &str, font: &Font, x: i16, y: i16, obj_size: i16);
    fn aa_filled_circle(&mut self, x: i16, y: i16, rad: i16, color: Color);
    fn draw_azimuthal_grid(&mut self);
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

impl Painter for Canvas<Window> {
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

    fn draw_azimuthal_grid(&mut self) {
        let color = Color::RGB(30, 30, 30);

        for i in 0..12 {
            let angle = i as f64 / 12.0 * PI;
            let (x1, y1) = horizontal_to_canvas(0.0, angle, self.logical_size());
            let (x2, y2) = horizontal_to_canvas(0.0, angle + PI, self.logical_size());
            self.aa_line(x1, y1, x2, y2, color).unwrap();
        }

        for i in (20..=80).step_by(20) {
            let (_, y) = horizontal_to_canvas(i as f64 / 180.0 * PI, 0.0, self.logical_size());
            let (width, height) = self.logical_size();
            let radius = <u32 as std::convert::TryInto<i16>>::try_into((height - PANEL_SIZE) / 2).unwrap() - y;
            self.aa_circle(
                (width / 2).try_into().unwrap(),
                ((height - PANEL_SIZE) / 2).try_into().unwrap(),
                radius,
                color,
            )
            .unwrap();
        }
    }
}
