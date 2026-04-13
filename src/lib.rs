#![allow(static_mut_refs)]

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
use entity::{ActHandler, ActSource, ActEffect, Cond};

pub mod port;

pub mod loader;

const BUTTON_CH: char = '◻';
const OPEN_DOOR: char = '⸬';
const CLOSED_DOOR: char = '█';
const LEVER_ON: char = 'Δ';
const LEVER_OFF: char = '𝈔';
const EXIT_CH: char = '!';

/// A state of a tile.
#[derive(Clone, Debug, PartialEq)]
pub struct TileState {
    /// Representation of this state.
    pub ch: StyleCh,
    /// Whether the tile in this state prevents movement through it.
    pub blocking: bool,
    /// Whether the tile in this state prevents lasers passing through it.
    pub opaque: bool,
    /// Whether it is a bad idea for the player to walk on to this tile.
    pub scary: bool,
}

impl TileState {
    /// Create a new tile state.
    pub const fn new(ch: StyleCh, blocking: bool, opaque: bool) -> Self {
        Self {
            ch,
            blocking,
            opaque,
            scary: false,
        }
    }

    /// Make the tile state scary.
    pub fn scary(self) -> Self {
        Self {
            scary: true,
            ..self
        }
    }

    /// Return a standard wall tile.
    fn wall() -> Self {
        Self {
            ch: '#'.white(),
            blocking: true,
            opaque: true,
            scary: false,
        }
    }

    /// Return a standard floor tile.
    fn floor() -> Self {
        Self {
            ch: '.'.white(),
            blocking: false,
            opaque: false,
            scary: false,
        }
    }
}

impl Default for TileState {
    fn default() -> Self {
        Self {
            blocking: false,
            ch: ' '.white(),
            opaque: false,
            scary: false,
        }
    }
}


/// A single tile that may or may not block movement.
#[derive(Clone, Debug, Default, PartialEq)]
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
        let ch1: char = loader::puzzles::map_clr(self.ch.style().foreground_color.unwrap()).unwrap();
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

    /// Create a water tile.
    pub fn water() -> Self {
        Self::from(
            TileState::new('~'.blue(), false, false)
                // Water is very scary.
                .scary()
        )
            .with_handlers(vec![
                ActHandler::new(ActSource::WalkOn, ActEffect::Murder.chain(ActEffect::Transform(Self::floor())))
            ])
    }

    /// Create a standard weighted button (can only be pressed by objects, not the player).
    pub fn wgtd_button() -> Self {
        Self::new(BUTTON_CH.dark_yellow(), false, false)
            .join(TileState::new(BUTTON_CH.white(), false, false))
            .with_handlers(
                vec![
                    ActHandler::new(ActSource::WalkOn, ActEffect::Prop),
                    ActHandler::new(ActSource::StayOn, ActEffect::Prop),
                ]
            )
    }

    /// Create a standard button.
    pub fn button() -> Self {
        Tile::new(BUTTON_CH.white(), false, false)
            .join(TileState::new(BUTTON_CH.white(), false, false))
            .with_handlers(
                vec![
                    ActHandler::new(ActSource::WalkOn, ActEffect::Prop),
                    ActHandler::new(ActSource::PlWalkOn, ActEffect::Prop),
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
                ActHandler::new(ActSource::FrameEnd, ActEffect::Reset),
            ]
        )
    }

    /// Create a standard one way door.
    pub fn single_door() -> Self {
        // Pretty much take the door and mess with it a bit.
        let mut dr = Self::door(true).with_handlers(vec![
            ActHandler::new(ActSource::PlWalkOn, ActEffect::MkActive)
        ]);

        dr.inact.ch = OPEN_DOOR.red();
        let Some(st) = dr.act.as_mut() else { unreachable!() };
        st.ch = CLOSED_DOOR.red();

        dr
    }

    /// Create a standard lever.
    pub fn lever(is_on: bool) -> Self {
        let on = TileState::new(LEVER_ON.white(), false, false);
        let off = TileState::new(LEVER_OFF.white(), false, false);

        let lever = if is_on {
            Tile::from(on).join(off)
        } else {
            Tile::from(off).join(on)
        };

        let mut cond = Cond::TActive;
        if is_on {
            cond = cond.not();
        }

        let handlers = vec![
            ActHandler::new(ActSource::WalkOn, ActEffect::Inv),
            ActHandler::new(ActSource::PlWalkOn, ActEffect::Inv),
            ActHandler::new(ActSource::WalkOn, cond.clone().cond_ef(ActEffect::Prop)),
            ActHandler::new(ActSource::PlWalkOn, cond.clone().cond_ef(ActEffect::Prop)),
            ActHandler::new(ActSource::FrameStart, cond.cond_ef(ActEffect::Prop)),
        ];

        lever.with_handlers(handlers)
    }

    /// Create a standard exit.
    pub fn exit() -> Self {
        Tile::new(EXIT_CH.green(), false, false)
            .with_handlers(vec![
                ActHandler::new(ActSource::PlWalkOn, ActEffect::Win)
            ])
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
        Self {
            inact: value,
            ..Default::default()
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
        self.ch
    }
}

impl ops::Deref for Tile {
    type Target = TileState;

    fn deref(&self) -> &Self::Target {
        self.cur_state()
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

    map.actuate_comms(comms.into_iter(), Point::new(999, 999));
    let mut comms = map.get_comms();

    // Inform all tiles with an entity upon them of their situation. They may have something to say
    // about it.
    for y in (0..map.hgt).rev() {
        for x in 0..=map.wid {
            let p = Point::new(x as i32, y as i32);
            if let Some(t) = map.get_map(p) && map.get_ent(p).is_some() {
                comms.queue_many(t.activate(map, ActSource::StayOn, p));
            }
        }
    }

    map.actuate_comms(comms.into_iter(), Point::new(999, 999));
    let mut comms = map.get_comms();

    // Tell all active lasers to do their thing.
    for y in (0..map.hgt).rev() {
        for x in 0..=map.wid {
            let p = Point::new(x as i32, y as i32);
            if let Some(e) = map.get_ent(p) {
                comms.queue_many(e.activate(map, ActSource::LaserTime, p));
            }
        }
    }

    map.actuate_comms(comms.into_iter(), Point::new(999, 999));
}

/// Make the move on the map.
pub fn mk_move(map: &mut bn::Map<Ent>) {
    map.update_vfx();

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
