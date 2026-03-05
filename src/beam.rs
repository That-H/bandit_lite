//! Contains objects for calculating and displaying laser beams.

use super::*;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Current inputs to each object in the map. Needs to be cleared after each update.
pub static INPTS: LazyLock<RwLock<HashMap<Point, port::Clrs>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Maps port numbers to directions out of the object.
pub const PORT_DIRS: [Point; 8] = [
    Point::new(-1, 1),
    Point::new(0, 1),
    Point::new(1, 1),
    Point::new(1, 0),
    Point::new(1, -1),
    Point::new(0, -1),
    Point::new(-1, -1),
    Point::new(-1, 0),
];

/// Characters used for non diagonal lasers.
const DIR_CHARS: [char; 4] = [
    '│',
    '─',
    '│',
    '─',
];

/// Characters for diagonal lasers.
const DIAG_CHARS: [char; 2] = [
    '╲',
    '╱',
];

/// Convert a direction into an object into its port number.
pub fn port_num(in_dir: Point) -> usize {
    match in_dir {
        Point { x: 1, y: -1 } => 0,
        Point { x: 0, y: -1 } => 1,
        Point { x: -1, y: -1 } => 2,
        Point { x: -1, y: 0 } => 3,
        Point { x: -1, y: 1 } => 4,
        Point { x: 0, y: 1 } => 5,
        Point { x: 1, y: 1 } => 6,
        Point { x: 1, y: 0 } => 7,
        _ => panic!("Invalid direction {in_dir}"),
    }
}

/// A laser colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Clr {
    #[default]
    Black=0b000,
    Red=0b100,
    Green=0b010,
    Blue=0b001,
    Yellow=0b110,
    Magenta=0b101,
    Cyan=0b011,
    White=0b111,
}

impl Clr {
    /// Returns the colour you would get by mixing the given two together.
    pub fn mix(self, other: Self) -> Self {
        Self::from(self as u8 | other as u8)
    }
}

impl From<u8> for Clr {
    fn from(val: u8) -> Self {
        match val {
            0b000 => Self::Black,
            0b100 => Self::Red,
            0b010 => Self::Green,
            0b001 => Self::Blue,
            0b110 => Self::Yellow,
            0b101 => Self::Magenta,
            0b011 => Self::Cyan,
            0b111 => Self::White,
            a => panic!("invalid colour {a}"),
        }
    }
}

impl From<style::Color> for Clr {
    fn from(value: style::Color) -> Self {
        match value {
            style::Color::Black => Self::Black,
            style::Color::Red => Self::Red,
            style::Color::Green => Self::Green,
            style::Color::Blue => Self::Blue,
            style::Color::Yellow => Self::Yellow,
            style::Color::Magenta => Self::Magenta,
            style::Color::Cyan => Self::Cyan,
            style::Color::White => Self::White,
            _ => Self::Black,
        }
    }
}

impl From<Clr> for style::Color {
    fn from(value: Clr) -> Self {
        match value {
            Clr::Black => Self::Black,
            Clr::Red => Self::Red,
            Clr::Green => Self::Green,
            Clr::Blue => Self::Blue,
            Clr::Yellow => Self::Yellow,
            Clr::Magenta => Self::Magenta,
            Clr::Cyan => Self::Cyan,
            Clr::White => Self::White,
        }
    }
}

impl TryFrom<char> for Clr {
    type Error = ();

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            // Null.
            'n' => Self::Black,
            'r' => Self::Red,
            'g' => Self::Green,
            'b' => Self::Blue,
            'y' => Self::Yellow,
            'm' => Self::Magenta,
            'c' => Self::Cyan,
            'w' => Self::White,
            _ => return Err(()),
        })
    }
}

impl From<Clr> for char {
    fn from(value: Clr) -> Self {
        match value {
            Clr::Black => 'n', 
            Clr::Red => 'r', 
            Clr::Green => 'g', 
            Clr::Blue => 'b', 
            Clr::Yellow => 'y', 
            Clr::Magenta => 'm', 
            Clr::Cyan => 'c', 
            Clr::White => 'w', 
        }
        
    }
}

/// A laser beam.
#[derive(Clone, Debug, PartialEq)]
pub struct Beam {
    /// Colour of the laser.
    pub clr: Clr,
    /// Direction of the laser.
    pub dir: Point,
}

impl Beam {
    /// Create a new laser beam.
    pub fn new(clr: Clr, dir: Point) -> Self {
        Self {
            clr,
            dir,
        }
    }

    /// True if this laser is diagonal, otherwise false.
    pub fn is_diag(&self) -> bool {
        self.dir.x != 0 && self.dir.y != 0
    }

    /// Returns the index into DIAG_CHARS for this beam's dir. Meaningless if this is not a
    /// diagonal beam.
    pub fn diag_idx(&self) -> usize {
        ((self.dir.x * self.dir.y + 1) / 2) as usize
    }

    /// Create the visual effects required to show this laser given a commands instance to do it
    /// with.
    pub fn propagate(&self, cmd: &mut bn::Commands<Ent>, pos: Point) {
        self.prop_internal(cmd, pos, &mut *INPTS.write().unwrap());
    }

    /// Internal propagation function.
    fn prop_internal(
        &self,
        cmd: &mut bn::Commands<Ent>,
        pos: Point,
        inpts: &mut HashMap<Point, port::Clrs>,
    ) {
        let mut cur = pos;
        let port = port_num(self.dir);

        loop {
            cur = cur + self.dir;
            if let Some(t) = cmd.get_map(cur) && !t.opaque {
                // An entity might change the beam, so handle this.
                if let Some(e) = cmd.get_ent(cur) {
                    let q = if let entity::EntType::Goal(clr) = e.tp && clr == self.clr {
                        true
                    } else {
                        false
                    };
                    inpts.entry(cur).or_default()[port] = self.clr;
                    for (o_port, &clr) in e.outputs(inpts.get(&cur).unwrap()).iter().enumerate() {
                        // Propagate beams that have a colour.
                        if clr != Clr::Black {
                            let bm = Self::new(clr, PORT_DIRS[o_port]);
                            bm.prop_internal(cmd, cur, inpts);
                        }
                    }
                    if q {
                        cmd.queue(bn::Cmd::new_on(cur).modify_entity(Box::new(
                            |e: &mut Ent| e.updated = true
                        )));
                    }
                    break;
                } else {
                    let ch = if self.is_diag() {
                        DIAG_CHARS[self.diag_idx()]
                    } else {
                        DIR_CHARS[self.dir.dir()]
                    };
                    cmd.queue(
                        bn::Cmd::new_on(cur)
                            .create_effect(
                                Vfx::new_opaque_single(ch.with(self.clr.into()))
                            )
                    );
                }
            } else {
                break;
            }
        }
    }
}

