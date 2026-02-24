//! Contains objects for calculating and displaying laser beams.

use super::*;

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

/// A laser colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Clr {
    #[default]
    Red=0b100,
    Green=0b010,
    Blue=0b001,
    Yellow=0b110,
    Purple=0b101,
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
            0b100 => Self::Red,
            0b010 => Self::Green,
            0b001 => Self::Blue,
            0b110 => Self::Yellow,
            0b101 => Self::Purple,
            0b011 => Self::Cyan,
            0b111 => Self::White,
            a => panic!("invalid colour {a}"),
        }
    }
}

impl From<Clr> for style::Color {
    fn from(value: Clr) -> Self {
        match value {
            Clr::Red => Self::Red,
            Clr::Green => Self::Green,
            Clr::Blue => Self::Blue,
            Clr::Yellow => Self::Yellow,
            Clr::Purple => Self::Magenta,
            Clr::Cyan => Self::Cyan,
            Clr::White => Self::White,
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
        let mut cur = pos;
        loop {
            cur = cur + self.dir;
            if let Some(t) = cmd.get_map(cur) && !t.opaque && cmd.get_ent(cur).is_none() {
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
            } else {
                break;
            }
        }
    }
}

