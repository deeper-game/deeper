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

use crate::editor::Document;
use crate::editor::Row;
use crate::editor::Terminal;
use crate::editor::Rasterized;
use std::time::Duration;
use std::time::Instant;
use std::collections::HashMap;
use bevy::input::Input;
use bevy::input::keyboard::KeyCode;
use bevy::time::Time;
use bevy::render::color::Color;
use termion::event::Key;

const STATUS_FG_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
const STATUS_BG_COLOR: Color = Color::rgb(0.94, 0.94, 0.94);
const QUIT_TIMES: u8 = 3;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self::from("")
    }
}

impl StatusMessage {
    fn from(message: &str) -> Self {
        Self {
            time: Instant::now(),
            text: message.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    quit_times: u8,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    highlighted_word: Option<String>,
    open_file: String,
    filesystem: HashMap<String, Document>,
    // prompt_callback: Option<Box<for<'a> Fn(&'a Editor)>>,
}

impl Editor {
    // pub fn run(&mut self) {
    //     self.refresh_screen();
    //     if self.should_quit {
    //         // leave editor mode
    //     }
    //     self.process_keypress();
    // }

    pub fn new() -> Self {
        let mut initial_status =
            "HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit";

        Self {
            should_quit: false,
            terminal: Terminal::default(),
            document: Document::default(),
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
            open_file: "foo".to_string(),
            filesystem: HashMap::new(),
            // prompt_callback: None,
        }
    }

    pub fn rasterize(&self) -> Option<Rasterized> {
        self.terminal.rasterize()
    }

    pub fn refresh_screen(&mut self) {
        self.terminal.cursor_hide();
        self.terminal.set_cursor_position(&Position::default());
        if self.should_quit {
            self.terminal.clear_screen();
            self.terminal.write("Goodbye.");
            self.terminal.carriage_return();
            self.terminal.newline();
        } else {
            self.document.highlight(
                &self.highlighted_word,
                Some(
                    self.offset
                        .y
                        .saturating_add(self.terminal.size().height as usize)),
            );
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            self.terminal.set_cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        self.terminal.cursor_show();
    }

    fn save(&mut self) {
        // let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
        // if new_name.is_none() {
        //     self.status_message = StatusMessage::from("Save aborted.");
        //     return;
        // }
        // self.open_file = new_name;
        self.filesystem.insert(self.open_file.clone(), self.document.clone());
        self.status_message = StatusMessage::from("File saved successfully.");
    }

    // fn search(&mut self) {
    //     let old_position = self.cursor_position.clone();
    //     let mut direction = SearchDirection::Forward;
    //     let query = self
    //         .prompt(
    //             "Search (ESC to cancel, arrows to navigate): ",
    //             |editor, key, query| {
    //                 let mut moved = false;
    //                 match key {
    //                     Key::Right | Key::Down => {
    //                         direction = SearchDirection::Forward;
    //                         editor.move_cursor(Key::Right);
    //                         moved = true;
    //                     }
    //                     Key::Left | Key::Up =>
    //                         direction = SearchDirection::Backward,
    //                     _ => direction = SearchDirection::Forward,
    //                 }
    //                 if let Some(position) =
    //                     editor
    //                         .document
    //                         .find(&query, &editor.cursor_position, direction)
    //                 {
    //                     editor.cursor_position = position;
    //                     editor.scroll();
    //                 } else if moved {
    //                     editor.move_cursor(Key::Left);
    //                 }
    //                 editor.highlighted_word = Some(query.to_string());
    //             },
    //         )
    //         .unwrap_or(None);
    //
    //     if query.is_none() {
    //         self.cursor_position = old_position;
    //         self.scroll();
    //     }
    //     self.highlighted_word = None;
    // }

    pub fn process_keypress(&mut self, pressed_key: Key) {
        match pressed_key {
            Key::Ctrl('q') => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(&format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return;
                }
                self.should_quit = true;
            }
            Key::Ctrl('s') => self.save(),
            // Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            }
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from("");
        }
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut x, mut y } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    fn draw_welcome_message(&mut self) {
        let mut welcome_message = format!("Micro editor version 4.33");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        self.terminal.write(&welcome_message);
        self.terminal.carriage_return();
        self.terminal.newline();
    }

    pub fn draw_row(&mut self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        self.terminal.write(&row);
        self.terminal.carriage_return();
        self.terminal.newline();
    }

    fn draw_rows(&mut self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            self.terminal.clear_current_line();
            let optional_row = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
                .cloned();
            if let Some(row) = optional_row {
                self.draw_row(&row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                self.terminal.write("~");
                self.terminal.carriage_return();
                self.terminal.newline();
            }
        }
    }

    fn draw_status_bar(&mut self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = self.open_file.to_string();
        file_name.truncate(20);
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        self.terminal.set_bg_color(STATUS_BG_COLOR);
        self.terminal.set_fg_color(STATUS_FG_COLOR);
        self.terminal.write(&status);
        self.terminal.carriage_return();
        self.terminal.newline();
        self.terminal.reset_fg_color();
        self.terminal.reset_bg_color();
    }

    fn draw_message_bar(&mut self) {
        self.terminal.clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            self.terminal.write(&text);
        }
    }

    // fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    // where
    //     C: FnMut(&mut Self, Key, &String),
    // {
    //     let mut result = String::new();
    //     loop {
    //         self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
    //         self.refresh_screen()?;
    //         let key = self.terminal.read_key()?;
    //         match key {
    //             Key::Backspace => result.truncate(result.len().saturating_sub(1)),
    //             Key::Char('\n') => break,
    //             Key::Char(c) => {
    //                 if !c.is_control() {
    //                     result.push(c);
    //                 }
    //             }
    //             Key::Esc => {
    //                 result.truncate(0);
    //                 break;
    //             }
    //             _ => (),
    //         }
    //         callback(self, key, &result);
    //     }
    //     self.status_message = StatusMessage::from("");
    //     if result.is_empty() {
    //         return Ok(None);
    //     }
    //     Ok(Some(result))
    // }
}
