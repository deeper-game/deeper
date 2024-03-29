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

use bevy::render::color::Color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    pub fn to_color(self) -> Color {
        match self {
            Type::Number => Color::rgb_u8(220, 163, 163),
            Type::Match => Color::rgb_u8(38, 139, 210),
            Type::String => Color::rgb_u8(211, 54, 130),
            Type::Character => Color::rgb_u8(108, 113, 196),
            Type::Comment | Type::MultilineComment => Color::rgb_u8(133, 153, 0),
            Type::PrimaryKeywords => Color::rgb_u8(181, 137, 0),
            Type::SecondaryKeywords => Color::rgb_u8(42, 161, 152),
            _ => Color::rgb_u8(255, 255, 255),
        }
    }
}
