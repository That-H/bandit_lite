//! Useful objects for activation.

use super::*;

/// A possible source of activation for an entity or tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActSource {
    /// An object such as a button.
    Obj,
    /// A laser beam.
    Laser,
    /// An object moves on to this tile.
    WalkOn,
    /// A player moves on to this tile.
    PlWalkOn,
    /// A player walks off of this tile.
    WalkOff,
    /// A player or an object is still on this tile at the start of the frame.
    StayOn,
    /// Frame ends.
    FrameEnd,
    /// Frame begins.
    FrameStart,
}

/// A condition that must be met for an effect to occur.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Cond {
    /// Tile at this position must be active.
    TActive,
    /// Entity at this position must be active.
    EActive,
    /// Entity at this position must have been active on the previous frame.
    EPrevActive,
    /// Condition must not be met.
    Not(Box<Self>),
    /// Both conditions must be met.
    Chain(Box<Self>, Box<Self>),
}

impl Cond {
    /// Join two conditions together in a Chain variant.
    pub fn chain(self, other: Self) -> Self {
        Self::Chain(Box::new(self), Box::new(other))
    }

    /// Invert a condition.
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Returns true if the condition is met. Requires the map and the position of the object
    /// that is checking the condition.
    pub fn check(&self, map: &bn::Map<Ent>, pos: Point) -> bool {
        match self {
            Self::TActive => {
                if let Some(t) = map.get_map(pos) && t.active {
                    true
                } else {
                    false
                }
            }
            Self::EActive => {
                if let Some(e) = map.get_ent(pos) && e.active {
                    true
                } else {
                    false
                }
            }
            Self::EPrevActive => {
                if let Some(e) = map.get_ent(pos) && e.prev_active {
                    true
                } else {
                    false
                }
            }
            Self::Not(cond) => {
                !cond.check(map, pos)
            }
            Self::Chain(c1, c2) => {
                c1.check(map, pos) && c2.check(map, pos)
            }
        }
    }

    /// Make a conditional effect out of this condition and the effect.
    pub fn cond_ef(self, ef: ActEffect) -> ActEffect {
        ActEffect::CondEf(self, Box::new(ef))
    }
}

/// A possible effect of being activated.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ActEffect {
    /// Propagate this activation to surrounding tiles/entities.
    Prop,
    /// Invert the current active state.
    Inv,
    /// Set the current active state to true.
    MkActive,
    /// Set the previous active state to the current one, then set the current one to false.
    Reset,
    /// Win the puzzle.
    Win,
    /// Does nothing.
    Null,
    /// Conditionally does something.
    CondEf(Cond, Box<Self>),
    /// Fire a laser beam(!).
    Laser(beam::Beam),
    /// Do both effects.
    Chain(Box<Self>, Box<Self>),
}

impl ActEffect {
    /// Put both effects in a Chain variant.
    pub fn chain(self, other: Self) -> Self {
        Self::Chain(Box::new(self), Box::new(other))
    }

    /// Return the commands needed to actuate this effect.#
    pub fn actuate(&self, map: &bn::Map<Ent>, pos: Point) -> Vec<bn::Cmd<Ent>> {
        let mut cmds = Vec::new();

        match self {
            ActEffect::Inv => {
                cmds.push(bn::Cmd::new_on(pos).modify_entity(Box::new(|e: &mut Ent| {
                    e.active = !e.active; 
                })));
                cmds.push(bn::Cmd::new_on(pos).modify_tile(Box::new(|t: &mut Tile| {
                    t.active = !t.active; 
                })));
            }
            ActEffect::MkActive => {
                cmds.push(bn::Cmd::new_on(pos).modify_entity(Box::new(|e: &mut Ent| {
                    e.active = true;
                })));
                cmds.push(bn::Cmd::new_on(pos).modify_tile(Box::new(|t: &mut Tile| {
                    t.active = true;
                })));
            }
            ActEffect::Prop => {
                for p in pos.get_all_adjacent_diagonal() {
                    if let Some(e) = map.get_ent(p) {
                        cmds.append(&mut e.activate(map, ActSource::Obj, p));
                    }
                    if let Some(t) = map.get_map(p) {
                        cmds.append(&mut t.activate(map, ActSource::Obj, p));
                    }
                }
            }
            ActEffect::Win => {
                unsafe {
                    SHOULD_WIN = true;
                }
            }
            ActEffect::Reset => {
                cmds.push(bn::Cmd::new_on(pos).modify_entity(Box::new(|e: &mut Ent| {
                    e.prev_active = e.active;
                    e.active = false;
                })));
                cmds.push(bn::Cmd::new_on(pos).modify_tile(Box::new(|t: &mut Tile| {
                    t.active = false;
                })));
            }
            ActEffect::Laser(bm) => {
                cmds.append(&mut bm.propagate(map, pos));
            }
            ActEffect::Chain(ef1, ef2) => {
                cmds.append(&mut ef1.actuate(map, pos));
                cmds.append(&mut ef2.actuate(map, pos));
            }
            ActEffect::CondEf(c, ef) => {
                if c.check(map, pos) {
                    cmds.append(&mut ef.actuate(map, pos));
                }
            }
            _ => (),
        }

        cmds
    }
}

/// Handles an activation from a source.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ActHandler {
    src: ActSource,
    ef: ActEffect,
}

impl ActHandler {
    /// Create a handler for the source that does the effect.
    pub fn new(src: ActSource, ef: ActEffect) -> Self {
        Self {
            src,
            ef,
        }
    }

    /// Handle an activation from the source. Requires the map the object is in and its position. 
    pub fn activate(&self, map: &bn::Map<Ent>, src: ActSource, pos: Point) -> Vec<bn::Cmd<Ent>> {
        let mut cmds = Vec::new();
        if src == self.src {
            cmds.append(&mut self.ef.actuate(map, pos));
        }
        cmds
    }
}

