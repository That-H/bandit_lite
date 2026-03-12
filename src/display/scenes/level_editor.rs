//! User friendly widget for creating puzzles.

use crate::entity::IMMOVABLE_CLR;
use loader::puzzles::ts::BanditObj;
use loader::ObjList;

use super::*;
use crossterm::event;

pub const DEFAULT_POS: Point = Point::new(TERMINAL_WID as i32 / 2, 5);
const CURSOR_CLR: style::Color = style::Color::DarkCyan;

/// Allows the user to create a puzzle and save it locally.
pub struct LevelEditor<'a> {
    data: &'a mut bn::Map<Ent>,
    objs: &'a ObjList,
    cursor: Point,
    /// Current object index into objs.
    pub cur_idx: usize,
    cur_rot: usize,
    just_deleted: bool,
    /// Current player position.
    pub pl_pos: Option<Point>,
}

impl<'a> LevelEditor<'a> {
    /// Create a new level editor.
    pub fn new(objs: &'a ObjList, data: &'a mut bn::Map<Ent>) -> Self {
        let editor = Self {
            objs,
            cursor: Point::new(1, 1),
            data,
            cur_idx: 0,
            cur_rot: 0,
            just_deleted: false,
            pl_pos: None,
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
                    let content = if self.just_deleted || (self.pl_pos.is_some() && self.cur_idx == 0) { 
                        let style_ch = if let Some(e) = self.data.get_ent(self.cursor) {
                            e.ch
                        } else {
                            self.data.get_map(self.cursor).unwrap().ch
                        };
                        *style_ch.content()
                    } else {
                        *self.get_cur_ch().content()
                    };
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
        for y in 0..self.data.hgt {
            for x in 0..self.data.wid {
                let p = Point::new(x as i32, y as i32);
                let tl = if x == 0 || x == self.data.wid-1 || y == 0 || y == self.data.hgt-1 {
                    Tile::wall()
                } else {
                    // Don't overwrite my walls with floors!
                    if let Some(t) = self.data.get_map(p) && t.blocking {
                        continue;
                    }
                    Tile::floor()
                };
                self.data.insert_tile(tl, p)
            }
        }
    }

    /// Get the current object to use.
    fn get_obj(&self) -> &BanditObj {
        &self.objs[self.cur_idx][self.cur_rot]
    }

    /// Get the representation of the current object.
    fn get_cur_ch(&self) -> &StyleCh {
        match self.get_obj() {
            BanditObj::En(e) => &e.ch,
            BanditObj::Tile(t) => &t.ch,
        }
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

    /// Place the current object at the cursor.
    fn place(&mut self) {
        match self.get_obj() {
            BanditObj::En(e) => {
                self.data.insert_entity(e.clone(), self.cursor);
                self.data.insert_tile(Tile::floor(), self.cursor);
            },
            BanditObj::Tile(t) => {
                self.data.insert_tile(t.clone(), self.cursor);
                self.data.del_ent(self.cursor);
            },
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
                if self.cur_idx == 0 && self.pl_pos.is_some() {
                    return EditEvent::Null;
                }
                self.place();
                self.just_deleted = false;
                Point::ORIGIN
            }
            event::KeyCode::Backspace => {
                // If no entity, delete a wall if there is one (replace with floor).
                if let Some(e) = self.data.del_ent(self.cursor) {
                    if e.is_player() {
                        self.pl_pos = None;
                    }
                } else {
                    self.data.insert_tile(Tile::floor(), self.cursor);
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
