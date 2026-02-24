pub use bandit as bn;
pub use bn::{Point, windowed};

// Necessary as we need to use one of the methods without conflicting with our tile.
use bn::Tile as Ti;

use crossterm::{queue, style, cursor};
use style::Stylize;
use std::{fmt, io};
use io::Write;

pub const TERMINAL_WID: u16 = 121;
pub const TERMINAL_HGT: u16 = 30;
pub const GAME_WID: u16 = 21;
pub const GAME_HGT: u16 = 21;

pub static mut PLAYER: Point = Point::ORIGIN;
pub static mut DIR: Point = Point::ORIGIN;

pub type StyleCh = style::StyledContent<char>;

pub mod display;

pub mod beam;

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
    pub fn new(ch: StyleCh, blocking: bool, opaque: bool) -> Self {
        Self {
            ch,
            blocking,
            opaque
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

/// Encodes the behaviour of an entity.
#[derive(Clone, Debug, PartialEq)]
pub enum EntType {
    /// Is the player.
    Player,
    /// Fires a coloured laser beam in the given direction.
    Laser(beam::Beam),
}

/// A movable object in a game.
#[derive(Clone, Debug)]
pub struct Ent {
    /// The character used to represent this entity.
    pub ch: StyleCh,
    /// Way in which this entity behaves.
    pub tp: EntType,
    /// Whether or not this entity has been updated yet.
    pub updated: bool,
}

impl Ent {
    /// Create an entity that is the player.
    pub fn player() -> Self {
        Self {
            ch: '@'.white(),
            tp: EntType::Player,
            updated: false,
        }
    }

    /// Create an entity that fires a laser.
    pub fn laser(dir: Point, clr: beam::Clr) -> Self {
        Self {
            ch: '!'.white(),
            tp: EntType::Laser(beam::Beam::new(clr, dir)),
            updated: false,
        }
    }

    /// True if this entity is the player entity.
    pub fn is_player(&self) -> bool {
        self.tp == EntType::Player
    }

    /// True if the player can walk on the given optional tile (false if it is None).
    pub fn walkable(tl: &Option<&Tile>) -> bool {
        if let Some(t) = tl && !t.blocking {
            true
        } else {
            false
        }
    }
}

impl bn::Entity for Ent {
    type Tile = Tile;
    type Vfx = Vfx;

    fn repr(&self) -> <<Self as bandit::Entity>::Tile as bandit::Tile>::Repr {
        self.ch.clone()
    }

    fn update(&self, cmd: &mut bandit::Commands<'_, Self>, pos: Point)
        where
            Self: Sized {
        match &self.tp {
            EntType::Player => unsafe { 
                let nx = PLAYER + DIR;
                if Ent::walkable(&cmd.get_map(nx)) {
                    // Push the entity in our way if possible.
                    if cmd.get_ent(nx).is_some() {
                        let nx2 = nx + DIR;
                        // Possible if the location we would push to contains no entity and is
                        // walkable.
                        if Ent::walkable(&cmd.get_map(nx2)) && cmd.get_ent(nx2).is_none() {
                            cmd.queue(bn::Cmd::new_on(nx).move_to(nx2));
                        } else {
                            return;
                        }
                    }
                    cmd.queue(bn::Cmd::new_here().displace(DIR));
                    PLAYER = nx;
                }
            },
            EntType::Laser(bm) => {
                bm.propagate(cmd, pos);
            }
        }
        
        // Make sure everyone knows we updated.
        cmd.queue(bn::Cmd::new_here().modify_entity(Box::new(|e: &mut Ent| e.updated = true)));
    }

    fn priority(&self) -> u32 {
        if self.updated {
            0
        } else if self.is_player() {
            2
        } else {
            1
        }
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

