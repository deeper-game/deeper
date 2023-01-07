// MIT License
//
// Copyright (c) 2021 Matthew Blode
// Copyright (c) 2023 Remy Goldschmidt
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::editor::Position;
use std::collections::HashMap;
use bevy::render::color::Color;

#[derive(PartialEq, Eq, Hash, Clone)]
struct Formatting {
    foreground_color: u32,
    background_color: u32,
    bold: bool,
}

impl Formatting {
    fn new(
        foreground_color: Color,
        background_color: Color,
        bold: bool,
    ) -> Formatting {
        Formatting {
            foreground_color: foreground_color.as_rgba_u32(),
            background_color: background_color.as_rgba_u32(),
            bold,
        }
    }

    fn foreground_color(&self) -> Color {
        let a: u8 = ((self.foreground_color >>  0) & 0xFF) as u8;
        let b: u8 = ((self.foreground_color >>  8) & 0xFF) as u8;
        let g: u8 = ((self.foreground_color >> 16) & 0xFF) as u8;
        let r: u8 = ((self.foreground_color >> 24) & 0xFF) as u8;
        Color::rgba_u8(r, g, b, a)
    }

    fn set_foreground_color(&mut self, color: Color) {
        self.foreground_color = color.as_rgba_u32();
    }

    fn background_color(&self) -> Color {
        let a: u8 = ((self.background_color >>  0) & 0xFF) as u8;
        let b: u8 = ((self.background_color >>  8) & 0xFF) as u8;
        let g: u8 = ((self.background_color >> 16) & 0xFF) as u8;
        let r: u8 = ((self.background_color >> 24) & 0xFF) as u8;
        Color::rgba_u8(r, g, b, a)
    }

    fn set_background_color(&mut self, color: Color) {
        self.background_color = color.as_rgba_u32();
    }

    fn bold(&self) -> bool {
        self.bold
    }
}

impl Default for Formatting {
    fn default() -> Formatting {
        Formatting::new(Color::WHITE, Color::BLACK, false)
    }
}

#[derive(Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct Rasterized {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
}

impl Rasterized {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data: Vec<u32> = Vec::new();
        data.resize(width * height, 0u32);
        Rasterized { width, height, data }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.data[y * self.width + x]
    }

    pub fn get_data(&self) -> &[u32] {
        &self.data[..]
    }

    pub fn set(&mut self, x: usize, y: usize, color: u32) {
        self.data[y * self.width + x] = color;
    }

    pub fn blit(&mut self, sprite: &Rasterized, upper_left: (usize, usize)) {
        let (upper_left_x, upper_left_y) = upper_left;
        for delta_x in 0 .. sprite.width {
            for delta_y in 0 .. sprite.height {
                let x = upper_left_x + delta_x;
                let y = upper_left_y + delta_y;
                self.set(x, y, sprite.get(delta_x, delta_y));
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct TerminalTile {
    character: char,
    formatting: Formatting,
}

impl Default for TerminalTile {
    fn default() -> Self {
        TerminalTile {
            character: ' ',
            formatting: Formatting::default(),
        }
    }
}

impl TerminalTile {
    pub fn rasterize(&self, inverted: bool) -> Option<Rasterized> {
        use noto_sans_mono_bitmap as noto;
        let rasterized = noto::get_raster(
            self.character,
            if self.formatting.bold() {
                noto::FontWeight::Bold
            } else {
                noto::FontWeight::Regular
            },
            noto::RasterHeight::Size32, // TODO: customizability
        )?;

        let [mut fr, mut fg, mut fb, mut fa] =
            self.formatting.foreground_color().as_linear_rgba_f32();
        let [mut br, mut bg, mut bb, mut ba] =
            self.formatting.background_color().as_linear_rgba_f32();
        if inverted {
            std::mem::swap(&mut fr, &mut br);
            std::mem::swap(&mut fg, &mut bg);
            std::mem::swap(&mut fb, &mut bb);
            std::mem::swap(&mut fa, &mut ba);
        }

        let width = rasterized.width();
        let height = rasterized.height();
        let mut result = Rasterized::new(width, height);

        for y in 0 .. height {
            for x in 0 .. width {
                let scale_u8: u8 = rasterized.raster()[y][x];
                let scale = (scale_u8 as f32) / 255.0;
                let convex_combine = |a: f32, b: f32| {
                    (scale * a) + ((1.0 - scale) * b)
                };
                // if self.character != ' ' {
                //     print!("{}", if scale_u8 > 50 { '#' } else { ' ' });
                // }
                let r = convex_combine(fr, br);
                let g = convex_combine(fg, bg);
                let b = convex_combine(fb, bb);
                let a = convex_combine(fa, ba);
                result.set(x, y, Color::rgba_linear(r, g, b, a).as_rgba_u32());
            }
            // if self.character != ' ' {
            //     println!("");
            // }
        }

        Some(result)
    }
}

#[derive(Clone)]
pub struct Terminal {
    size: Size,
    screen: Vec<TerminalTile>,
    cursor_position: Position,
    cursor_visible: bool,
    formatting: Formatting,
}

impl Default for Terminal {
    fn default() -> Self {
        let width = 80;
        let height = 24;
        let mut screen = Vec::new();
        screen.resize(width * height, TerminalTile::default());
        Terminal {
            size: Size { width, height },
            screen: screen,
            cursor_position: Position { x: 0, y: 0 },
            cursor_visible: true,
            formatting: Formatting::default(),
        }
    }
}

impl Terminal {
    pub fn size(&self) -> Size {
        Size {
            width: self.size.width,
            height: self.size.height.saturating_sub(2)
        }
    }

    pub fn rasterize(&self) -> Option<Rasterized> {
        let mut cache: HashMap<TerminalTile, Rasterized> = HashMap::new();
        for tile in self.screen.iter() {
            cache.insert(tile.clone(), tile.rasterize(false)?);
        }

        let (tile_width, tile_height) = {
            let rasterized_tile = &cache[&self.screen[0]];
            (rasterized_tile.width, rasterized_tile.height)
        };

        let mut result = Rasterized::new(self.size.width * tile_width,
                                         self.size.height * tile_height);
        for tile_y in 0 .. self.size.height {
            for tile_x in 0 .. self.size.width {
                let upper_left_x = tile_x * tile_width;
                let upper_left_y = tile_y * tile_height;
                let tile = &self.screen[tile_y * self.size.width + tile_x];
                let Position { x: cx, y: cy } = self.cursor_position;
                let rasterized = if (tile_x == cx) && (tile_y == cy) {
                    tile.rasterize(true)?
                } else {
                    cache[tile].clone()
                };
                result.blit(&rasterized, (upper_left_x, upper_left_y));
            }
        }

        Some(result)
    }

    pub fn clear_screen(&mut self) {
        self.screen.clear();
        self.screen.resize(self.size.width * self.size.height,
                           TerminalTile::default());
    }

    pub fn set_cursor_position(&mut self, position: &Position) {
        self.cursor_position = position.clone();
    }

    pub fn write(&mut self, string: &str) {
        let mut index =
            self.cursor_position.y * self.size.width + self.cursor_position.x;
        for c in string.chars() { // TODO: use unicode segmentation
            self.screen[index] = TerminalTile {
                character: c,
                formatting: self.formatting.clone(),
            };
            index += 1;
            if index >= self.size.width * self.size.height {
                index = (self.size.width * self.size.height) - 1;
            }
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor_position.x = 0;
    }

    pub fn newline(&mut self) {
        self.cursor_position.x = 0;
        self.cursor_position.y += 1;
    }

    pub fn cursor_hide(&mut self) {
        self.cursor_visible = false;
    }

    pub fn cursor_show(&mut self) {
        self.cursor_visible = true;
    }

    pub fn clear_current_line(&mut self) {
        for x in 0 .. self.size.width {
            let index = self.cursor_position.y * self.size.width + x;
            self.screen[index] = TerminalTile::default();
        }
    }

    pub fn set_bg_color(&mut self, color: Color) {
        self.formatting.set_background_color(color);
    }

    pub fn reset_bg_color(&mut self) {
        self.formatting.set_background_color(Color::BLACK);
    }

    pub fn set_fg_color(&mut self, color: Color) {
        self.formatting.set_foreground_color(color);
    }

    pub fn reset_fg_color(&mut self) {
        self.formatting.set_foreground_color(Color::WHITE);
    }
}
