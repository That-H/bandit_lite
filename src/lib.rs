pub use bandit as bn;
pub use bn::{Point, windowed};

// Necessary as we need to use one of the methods without conflicting with our tile.
use bn::Tile as Ti;

use crossterm::{queue, style, cursor};
use style::Stylize;
use std::{fmt, io};
use std::sync::RwLock;
use io::Write;

pub const TERMINAL_WID: u16 = 121;
pub const TERMINAL_HGT: u16 = 30;
pub const GAME_WID: u16 = 20;
pub const GAME_HGT: u16 = 20;

/// Position of the player.
pub static mut PLAYER: Point = Point::ORIGIN;
/// Direction the player should try to move in.
pub static mut DIR: Point = Point::ORIGIN;
/// Whether the puzzle is complete yet or not.
pub static mut SHOULD_WIN: bool = false;
/// List of moves that have been played.
pub static MOVES: RwLock<Vec<entity::Move>> = RwLock::new(Vec::new());

pub type StyleCh = style::StyledContent<char>;

pub mod display;

pub mod beam;

pub mod entity;
pub use entity::Ent;

pub mod port;

pub mod loader;

/// A single tile that may or may not block movement.
#[derive(Clone, Debug)]
pub struct Tile {
    /// The character used to represent this tile.
    pub ch: StyleCh,
    /// Whether or not this tile prevents movement.
    pub blocking: bool,
    /// Whether or not this tile stops laser beams.
    pub opaque: bool,
}

impl Tile {
    /// Construct a tile with the given fields.
    pub const fn new(ch: StyleCh, blocking: bool, opaque: bool) -> Self {
        Self {
            ch,
            blocking,
            opaque
        }
    }

    /// Return the representation of this tile as it would be in a file.
    pub fn file_repr(&self) -> String {
        let mut string = String::new();
        let ch1: char = beam::Clr::from(self.ch.style().foreground_color.unwrap()).into();
        string.push(ch1);
        string.push(*self.ch.content());

        string
    }

    /// Return a standard wall tile.
    pub fn wall() -> Self {
        Self {
            ch: '#'.white(),
            blocking: true,
            opaque: true,
        }
    }

    /// Return a standard floor tile.
    pub fn floor() -> Self {
        Self {
            ch: '.'.white(),
            blocking: false,
            opaque: false,
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            blocking: false,
            ch: ' '.stylize(),
            opaque: false,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl bn::Tile for Tile {
    type Repr = StyleCh;

    fn repr(&self) -> Self::Repr {
        self.ch.clone()
    }
}
/// A singular frame of animation in a visual effect.
#[derive(Clone, Debug)]
pub enum Frame {
    /// Change nothing.
    Transparent,
    /// Replace with the given styled character.
    Opaque(StyleCh),
    /// Set the colour that the character is on.
    ReplaceFloor(style::Color),
    /// Compute the new text in some other way.
    Other(Box<fn(&StyleCh) -> StyleCh>),
}

impl Frame {
    /// Turn the original text at this position into something new.
    pub fn map(&self, txt: &StyleCh) -> StyleCh {
        match self {
            Self::Transparent => *txt,
            Self::Opaque(ch) => *ch,
            Self::ReplaceFloor(clr) => txt.on(*clr),
            Self::Other(cl) => cl(txt),
        }
    }
}

/// A visual effect in the grid.
#[derive(Clone, Debug)]
pub struct Vfx {
    frames: Vec<Frame>,
    cur_idx: usize,
}

impl Vfx {
    /// Create a new instance with the given frames.
    pub fn new(frames: Vec<Frame>) -> Self {
        Self { frames, cur_idx: 0 }
    }

    /// Create a new instance with frames copies of the given character
    /// as opaque frames.
    pub fn new_opaque(ch: StyleCh, frames: usize) -> Self {
        Self {
            frames: vec![Frame::Opaque(ch); frames],
            cur_idx: 0,
        }
    }

    /// Create a new instance with 1 copy of the given character
    /// as opaque frames.
    pub fn new_opaque_single(ch: StyleCh) -> Self {
        Self::new_opaque(ch, 1)
    }

    /// Create a new instance with frames copies of the given character
    /// coloured using clr as opaque frames.
    pub fn opaque_with_clr(ch: char, clr: style::Color, frames: usize) -> Self {
        Self {
            frames: vec![Frame::Opaque(ch.with(clr)); frames],
            cur_idx: 0,
        }
    }
}

impl bn::Vfx for Vfx {
    type Txt = StyleCh;

    fn update(&mut self) -> bool {
        self.cur_idx += 1;
        self.cur_idx == self.frames.len()
    }

    fn modify_txt(&self, txt: &Self::Txt) -> Self::Txt {
        self.frames[self.cur_idx].map(txt)
    }
}

