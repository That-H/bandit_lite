//! A custom widget for displaying additonal windows with buttons.

use super::*;
use ui::widgets::Button;
use std::io;

/// A button with an attached window.
#[derive(Clone, Debug)]
pub struct LinkedButton {
    button: Button,
    win: windowed::Window<StyleCh>,
    hovered: bool,
}

impl LinkedButton {
    /// Create a linked button.
    pub fn new(button: Button, win: windowed::Window<StyleCh>) -> Self {
        Self {
            button,
            win,
            hovered: false,
        }
    }
}

impl ui::UiElement for LinkedButton {
    fn activate(&mut self) -> Vec<ui::Event> {
        self.button.activate()
    }

    fn true_pos(&self) -> Point {
        self.button.true_pos()
    }

    fn receive(&mut self, data: &str) {
        if data == "clr" {
            let mut handle = io::stdout();
            let hgt = self.win.data.len();
            let wid = self.win.data[0].len();
            for y in 0..hgt {
                let y = y as u16 + self.win.top_left.y as u16;
                for x in 0..wid {
                    let x = x as u16 + self.win.top_left.x as u16;
                    let _ = queue!(
                        handle,
                        cursor::MoveTo(x, y),
                        style::Print(' '.stylize()),
                    );
                }
            }
            let _ = handle.flush();
        }
    }

    fn priority(&self) -> i32 {
        self.button.priority()
    }

    fn get_text(&self) -> String {
        self.button.get_text()
    }

    fn display_into(&self, win: &mut windowed::Window<StyleCh>, offset: Point) {
        self.button.display_into(win, offset);
    }

    fn toggle_hover(&mut self) {
        self.button.toggle_hover();
        self.hovered = !self.hovered;
        if !self.hovered {
            return;
        }
        let mut handle = io::stdout();
        for (y, row) in self.win.data.iter().enumerate() {
            let y = y as u16 + self.win.top_left.y as u16;
            for (x, &ch) in row.iter().enumerate() {
                let x = x as u16 + self.win.top_left.x as u16;
                let _ = queue!(
                    handle,
                    cursor::MoveTo(x, y),
                    style::Print(ch),
                );
            }
        }
        let _ = handle.flush();
    }
}
