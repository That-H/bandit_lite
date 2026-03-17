pub use bandit as bn;
pub use bn::{Point, windowed};

// Necessary as we need to use one of the methods without conflicting with our tile.
use bn::Tile as Ti;

use crossterm::{queue, style, cursor};
use style::Stylize;
use std::{fmt, io, ops};
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
use entity::{ActHandler, ActSource, ActEffect};

pub mod port;

pub mod loader;

const BUTTON_CH: char = '◻';
const OPEN_DOOR: char = '⸬';
const CLOSED_DOOR: char = '█';

/// A state of a tile.
#[derive(Clone, Debug, PartialEq)]
pub struct TileState {
    /// Representation of this state.
    pub ch: StyleCh,
    /// Whether the tile in this state prevents movement through it.
    pub blocking: bool,
    /// Whether the tile in this state prevents lasers passing through it.
    pub opaque: bool,
}

impl TileState {
    /// Create a new tile state.
    pub const fn new(ch: StyleCh, blocking: bool, opaque: bool) -> Self {
        Self {
            ch,
            blocking,
            opaque,
        }
    }

    /// Return a standard wall tile.
    fn wall() -> Self {
        Self {
            ch: '#'.white(),
            blocking: true,
            opaque: true,
        }
    }

    /// Return a standard floor tile.
    fn floor() -> Self {
        Self {
            ch: '.'.white(),
            blocking: false,
            opaque: false,
        }
    }
}

impl Default for TileState {
    fn default() -> Self {
        Self {
            blocking: false,
            ch: ' '.white(),
            opaque: false,
        }
    }
}


/// A single tile that may or may not block movement.
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    /// State when the tile is not being activated.
    inact: TileState,
    /// State when the tile is being activated.
    act: Option<TileState>,
    /// Whether the tile is being activated.
    active: bool,
    /// Activation handlers of this tile.
    handlers: Vec<ActHandler>,
}

impl Tile {
    /// Construct a single state tile with the given fields.
    pub fn new(ch: StyleCh, blocking: bool, opaque: bool) -> Self {
        Self {
            inact: TileState::new(ch, blocking, opaque),
            ..Default::default()
        }
    }

    /// Set the active state of this tile to the provided one.
    pub fn join(self, active: TileState) -> Self {
        Self {
            act: Some(active),
            ..self
        }
    }

    /// Add the given activation handlers to the tile.
    pub fn with_handlers(self, handlers: Vec<ActHandler>) -> Self {
        Self {
            handlers,
            ..self
        }
    }

    /// If this is a dual state tile, return a new tile with the inactive and active states
    /// reversed.
    pub fn flipped(&self) -> Option<Self> {
        Some(Self {
            inact: self.act.clone()?,
            act: Some(self.inact.clone()),
            active: false,
            handlers: self.handlers.clone(),
        })
    }

    /// Return the representation of this tile as it would be in a file.
    pub fn file_repr(&self) -> String {
        let mut string = String::new();
        let ch1: char = beam::Clr::from(self.ch.style().foreground_color.unwrap()).into();
        string.push(ch1);
        string.push(*self.ch.content());

        string
    }

    /// Get a reference to the current state.
    fn cur_state(&self) -> &TileState {
        if self.active && let Some(st) = &self.act {
            st 
        } else {
            &self.inact
        }
    }

    /// Create a standard floor.
    pub fn floor() -> Self {
        Self {
            inact: TileState::floor(),
            ..Default::default()
        }
    }

    /// Create a standard wall.
    pub fn wall() -> Self {
        Self {
            inact: TileState::wall(),
            ..Default::default()
        }
    }

    /// Create a standard button.
    pub fn button() -> Self {
        Tile::new(BUTTON_CH.white(), false, false)
            .join(TileState::new(BUTTON_CH.white(), false, false))
            .with_handlers(
                vec![
                    ActHandler::new(ActSource::WalkOn, ActEffect::Prop),
                    ActHandler::new(ActSource::StayOn, ActEffect::Prop),
                ]
            )
    }

    /// Create a standard door.
    pub fn door(open: bool) -> Self {
        let op = TileState::new(OPEN_DOOR.white(), false, false);
        let cls = TileState::new(CLOSED_DOOR.white(), true, true);

        let dr = if open {
            Tile::from(op).join(cls)
        } else {
            Tile::from(cls).join(op)
        };

        dr.with_handlers(
            vec![
                ActHandler::new(ActSource::Obj, ActEffect::MkActive),
                ActHandler::new(ActSource::FrameStart, ActEffect::Reset),
            ]
        )
    }

    /// Do the effect of activating this tile, using the map and the position of
    /// this entity.
    pub fn activate(&self, map: &bn::Map<Ent>, src: ActSource, pos: Point) -> Vec<bn::Cmd<Ent>> {
        let mut cmds = Vec::new();
        for handle in self.handlers.iter() {
            cmds.append(&mut handle.activate(map, src, pos));
        }
        cmds
    }
}

impl From<TileState> for Tile {
    fn from(value: TileState) -> Self {
        Self::new(value.ch, value.blocking, value.opaque)
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

impl ops::Deref for Tile {
    type Target = TileState;

    fn deref(&self) -> &Self::Target {
        self.cur_state()
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            active: false,
            inact: TileState::default(),
            act: None,
            handlers: Vec::new(),
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

/// Start the frame.
pub fn start_frame(map: &mut bn::Map<Ent>) {
    map.update_vfx();

    let mut comms = map.get_comms();

    // Start the frame.
    for y in (0..map.hgt).rev() {
        for x in 0..=map.wid {
            let p = Point::new(x as i32, y as i32);

            if let Some(e) = map.get_ent(p) {
                comms.queue_many(e.activate(map, ActSource::FrameStart, p));
            }
            if let Some(t) = map.get_map(p) {
                comms.queue_many(t.activate(map, ActSource::FrameStart, p));
            }
        }
    }

    // Inform all tiles with an entity upon them of their situation. They may have something to say
    // about it.
    for y in (0..map.hgt).rev() {
        for x in 0..=map.wid {
            let p = Point::new(x as i32, y as i32);
            if let Some(t) = map.get_map(p) {
                if map.get_ent(p).is_some() {
                    comms.queue_many(t.activate(map, ActSource::StayOn, p));
                }
            }
        }
    }

    map.actuate_comms(comms.into_iter(), Point::new(999, 999));
}

/// Make the move on the map.
pub fn mk_move(map: &mut bn::Map<Ent>) {
    while map.update() {}
}

/// Do things required at the end of a frame.
pub fn end_frame(map: &mut bn::Map<Ent>) {
    let mut to_reset = Vec::new();

    for (&p, _e) in map.get_entities() {
        to_reset.push(p);
    }

    for p in to_reset {
        map.get_ent_mut(p).unwrap().updated = false;
    }

    let mut comms = map.get_comms();

    // End the frame.
    for y in (0..map.hgt).rev() {
        for x in 0..=map.wid {
            let p = Point::new(x as i32, y as i32);

            if let Some(e) = map.get_ent(p) {
                comms.queue_many(e.activate(map, ActSource::FrameEnd, p));
            }
            if let Some(t) = map.get_map(p) {
                comms.queue_many(t.activate(map, ActSource::FrameEnd, p));
            }
        }
    }

    // Don't actually have a source, so we just say we have a ridiculous one. This is ok as long as
    // we never use Cmd::new_here().
    map.actuate_comms(comms.into_iter(), Point::new(999, 999));

    beam::INPTS.write().unwrap().clear();
}
