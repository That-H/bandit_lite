//! A counter that can be incremented or decremented.

use super::*;

/// A text label that supports numeric modification.
#[derive(Clone, Debug, Default)]
pub struct Counter {
    /// Current number stored in the counter.
    pub value: i32,
    /// Optional minimum.
    pub min: Option<i32>,
    /// Optional maximum.
    pub max: Option<i32>,
    /// Identifier of this counter.
    pub id: usize,
    /// Internally used to display this as a UiElement.
    button: ui::widgets::Button,
}

impl Counter {
    /// Construct an empty counter.
    pub fn new(id: usize, pos: Point) -> Self {
        Self {
            id,
            button: basic_button().set_screen_pos(pos).set_static_len(false),
            ..Default::default()
        }
    }

    /// Sets the value of the counter. Will not change the value if it is below the minimum or
    /// above the maximum.
    pub fn set_value(&mut self, value: i32) {
        if let Some(m) = self.min && value < m {
            return;
        } else if let Some(m) = self.max && value > m {
            return;
        }

        self.value = value;
        self.button = self.button.clone().set_txt(self.value.to_string());
    }

    /// Sets the maximum of the counter.
    pub fn with_max(self, max: i32) -> Self {
        Self {
            max: Some(max),
            ..self
        }
    }

    /// Sets the minimum of the counter.
    pub fn with_min(self, min: i32) -> Self {
        Self {
            min: Some(min),
            ..self
        }
    }
}

impl ui::UiElement for Counter {
    fn activate(&mut self) -> Vec<ui::Event> {
        Vec::new() 
    }

    fn priority(&self) -> i32 {
        self.button.priority()
    }

    fn get_text(&self) -> String {
        self.value.to_string()
    }

    fn true_pos(&self) -> Point {
        self.button.true_pos()
    }

    fn display_into(&self, win: &mut windowed::Window<StyleCh>, offset: Point) {
        self.button.display_into(win, offset);
    }

    fn toggle_hover(&mut self) {
        self.button.toggle_hover();
    }

    fn receive_text(&mut self, _ev: crossterm::event::KeyCode) -> bool {
        false
    }

    fn receive(&mut self, data: &str) {
        for (n, part) in data.split(":").enumerate() {
            if n == 0 {
                if let Ok(id) = part.parse::<usize>() && id == self.id {
                    continue;
                }
            } else {
                self.set_value(self.value + part.parse::<i32>().unwrap());
            }
            return;
        }
    }
}

