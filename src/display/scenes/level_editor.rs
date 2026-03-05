//! User friendly widget for creating puzzles.

use crate::entity::IMMOVABLE_CLR;

use super::*;
use crossterm::event;

pub const DEFAULT_POS: Point = Point::new(TERMINAL_WID as i32 / 2, 5);
const CURSOR_CLR: style::Color = style::Color::DarkCyan;

/// Allows the user to create a puzzle and save it locally.
pub struct LevelEditor<'a> {
    data: &'a mut bn::Map<Ent>,
    objs: &'a Vec<Vec<Ent>>,
    cursor: Point,
    /// Current object index into objs.
    pub cur_idx: usize,
    cur_rot: usize,
    just_deleted: bool,
}

impl<'a> LevelEditor<'a> {
    /// Create a new level editor.
    pub fn new(objs: &'a Vec<Vec<Ent>>, data: &'a mut bn::Map<Ent>) -> Self {
        let editor = Self {
            objs,
            cursor: Point::new(1, 1),
            data,
            cur_idx: 0,
            cur_rot: 0,
            just_deleted: false,
        };
        editor
    }

    /// Change the size of this level editor. 
    pub fn resize(&mut self, new_wid: usize, new_hgt: usize) {
        self.outline();
        self.data.wid = new_wid;
        self.data.hgt = new_hgt;
    }

    /// Draws this level editor into the window.
    pub fn draw(&self, win: &mut windowed::Window<StyleCh>, show_cursor: bool) {
        win.data.clear();
        for y in (1..=self.data.hgt-2).rev() {
            let mut row = Vec::new();
            for x in 1..=self.data.wid-2 {
                let p = Point::new(x as i32, y as i32);

                let mut ch = if let Some(e) = self.data.get_ent(p) {
                    let mut ch = e.ch.clone();
                    if !e.movable {
                        ch = ch.on(IMMOVABLE_CLR);
                    }
                    ch
                } else if let Some(t) = self.data.get_map(p) {
                    t.ch.clone()
                } else {
                    ' '.stylize()
                };

                if p == self.cursor && show_cursor {
                    let content = if self.just_deleted { '.' } else { *self.get_obj().ch.content() };
                    ch = content.on(CURSOR_CLR);
                }

                row.push(ch);
            }
            win.data.push(row);
        }
        win.outline_with(outline_ch());
    }

    /// Outline the map with walls.
    pub fn outline(&mut self) {
        for y in 0..=self.data.hgt+1 {
            for x in 0..=self.data.wid+1 {
                let tl = if x == 0 || x == self.data.wid+1 || y == 0 || y == self.data.hgt+1 {
                    Tile::wall()
                } else {
                    Tile::floor()
                };
                self.data.insert_tile(tl, Point::new(x as i32, y as i32))
            }
        }
    }

    /// Get the current object to use.
    fn get_obj(&self) -> &Ent {
        &self.objs[self.cur_idx][self.cur_rot]
    }

    /// Increase this rotation, ensuring it does not end up out of bounds.
    fn rot(&mut self, up: bool) {
        let len = self.objs[self.cur_idx].len();
        if up {
            self.cur_rot += 1;
            if self.cur_rot == len {
                self.cur_rot = 0;
            }
        } else {
            if self.cur_rot == 0 {
                self.cur_rot = len - 1;
            } else {
                self.cur_rot -= 1;
            }
        }

    }

    /// Do something with a key event.
    pub fn handle_key(&mut self, ev: event::KeyEvent) -> EditEvent {
        let mv = match ev.code {
            event::KeyCode::Left
            | event::KeyCode::Char('a')
            | event::KeyCode::Char('h') => Point::new(-1, 0),
            event::KeyCode::Right
            | event::KeyCode::Char('d')
            | event::KeyCode::Char('l') => Point::new(1, 0),
            event::KeyCode::Down
            | event::KeyCode::Char('s')
            | event::KeyCode::Char('j') => Point::new(0, -1),
            event::KeyCode::Up
            | event::KeyCode::Char('w')
            | event::KeyCode::Char('k') => Point::new(0, 1),
            event::KeyCode::Char('o') => {
                self.cur_rot = 0;
                self.just_deleted = false;
                return EditEvent::PickObj;
            }
            event::KeyCode::Char('y') => {
                self.rot(true);
                Point::ORIGIN
            }
            event::KeyCode::Char('i') => {
                self.rot(false);
                Point::ORIGIN
            }
            event::KeyCode::Char('m') => {
                if let Some(e) = self.data.get_ent_mut(self.cursor) {
                    e.movable = !e.movable;
                }
                Point::ORIGIN
            }
            event::KeyCode::Enter => {
                self.data.insert_entity(self.get_obj().clone(), self.cursor);
                self.just_deleted = false;
                Point::ORIGIN
            }
            event::KeyCode::Char(';') => {
                self.data.insert_tile(Tile::wall(), self.cursor);
                Point::ORIGIN
            }
            event::KeyCode::Backspace => {
                // If no entity, delete a wall if there is one (replace with floor).
                if self.data.del_ent(self.cursor).is_none() {
                    if let Some(t) = self.data.get_map_mut(self.cursor) && t.blocking {
                        self.data.insert_tile(Tile::floor(), self.cursor);
                    }
                }
                self.just_deleted = true;
                Point::ORIGIN
            }
            event::KeyCode::Esc => {
                return EditEvent::Menu;
            }
            _ => Point::ORIGIN
        };

        let new = self.cursor + mv;
        if mv == Point::ORIGIN || new.x >= self.data.wid as i32-1 || new.x <= 0 || new.y >= self.data.hgt as i32-1 || new.y <= 0 {
            return EditEvent::Null;
        }
        self.just_deleted = false;
        self.cursor = new;
        EditEvent::Null
    }
}

/// Something that may occur while editing a puzzle that must be handled externally.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub enum EditEvent {
    PickObj,
    Menu,
    Null,
}
