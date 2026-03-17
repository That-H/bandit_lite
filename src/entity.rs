use super::*;

const DIAG_ARROWS: [char; 8] = ['↖', '↑', '↗', '→', '↘', '↓', '↙', '←'];
/// Background colour of immovable objects.
pub const IMMOVABLE_CLR: style::Color = style::Color::DarkGrey;

/// A move that the player did.
#[derive(Clone, Copy, Debug)]
pub struct Move(pub Point);

impl Move {
    /// Create a move.
    pub const fn new(dir: Point) -> Self {
        Self(dir)
    }
}

/// A possible source of activation for an entity or tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActSource {
    /// An object such as a button.
    Obj,
    /// A laser beam.
    Laser,
    /// A player or an object moves on to this tile.
    WalkOn,
    /// A player walks off of this tile.
    WalkOff,
    /// A player or an object is still on this tile at the start of the frame.
    StayOn,
    /// Frame ends.
    FrameEnd,
    /// Frame begins.
    FrameStart,
}

/// A possible effect of being activated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
}

/// Handles an activation from a source.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
            match self.ef {
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
                        e.active = false;
                    })));
                    cmds.push(bn::Cmd::new_on(pos).modify_tile(Box::new(|t: &mut Tile| {
                        t.active = false;
                    })));
                }
                ActEffect::Null => (),
            }
        }
        cmds
    }
}

/// Encodes the behaviour of an entity.
#[derive(Clone, Debug)]
pub enum EntType {
    /// Is the player.
    Player,
    /// Fires a coloured laser beam in the given direction.
    Laser(beam::Beam),
    /// Uses its inputs to potentially create an output.
    Obj(port::PortGrp),
    /// Takes an input of the given colour to any port. This activates all surrounding
    /// tiles/entities.
    Goal(beam::Clr),
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
    /// True if this entity can be pushed.
    pub movable: bool,
    /// True if this entity is being activated this frame.
    pub active: bool,
    /// Activation handlers of this entity.
    pub handlers: Vec<ActHandler>,
}

impl Ent {
    /// Create an entity that is the player.
    pub fn player() -> Self {
        Self::default()
    }

    /// Create an entity that fires a laser.
    pub fn laser(dir: Point, clr: beam::Clr) -> Self {
        Self {
            ch: DIAG_ARROWS[beam::port_num(-dir)].with(clr.into()),
            tp: EntType::Laser(beam::Beam::new(clr, dir)),
            ..Default::default()
        }
    }

    /// Create a goal entity.
    pub fn goal(clr: beam::Clr) -> Self {
        Self {
            ch: 'O'.with(clr.into()),
            tp: EntType::Goal(clr),
            ..Default::default()
        }
    }

    /// Create an object that takes laser inputs.
    pub fn obj(ch: StyleCh, exprs: Vec<port::Expr>) -> Self {
        Self {
            ch,
            tp: EntType::Obj(
                port::PortGrp::from_iter(
                    exprs.into_iter()
                )
            ),
            ..Default::default()
        }
    }

    /// True if this entity is the player entity.
    pub fn is_player(&self) -> bool {
        match self.tp {
            EntType::Player => true,
            _ => false,
        }
    }

    /// True if the player can walk on the given optional tile (false if it is None).
    pub fn walkable(tl: &Option<&Tile>) -> bool {
        if let Some(t) = tl && !t.blocking {
            true
        } else {
            false
        }
    }

    /// Rotate this entity 90 degrees clockwise.
    pub fn rotate_90(&mut self) {
        match &mut self.tp {
            EntType::Laser(bm) => bm.dir.rotate_90_cw_ip(),
            EntType::Obj(pgrp) => pgrp.rotate_90(),
            _ => (),
        }
    }

    /// Get all beams that would be output by this entity.
    pub fn outputs(&self, inpts: &port::Clrs) -> port::Clrs {
        match &self.tp {
            EntType::Obj(ports) => {
                let outs = ports.determine(inpts);

                outs
            }
            _ => Default::default(),
        }
    }

    /// Get the way this should be represented in a file.
    pub fn file_repr(&self) -> String {
        let mut string = String::new();
        let mut ch1: char = beam::Clr::from(self.ch.style().foreground_color.unwrap()).into();
        if !self.movable { 
            ch1 = ch1.to_ascii_uppercase();
        }
        string.push(ch1);
        string.push(*self.ch.content());

        string
    }

    /// Do the effect of activating this entity, using the commands instance and the position of
    /// this entity.
    pub fn activate(&self, map: &bn::Map<Ent>, src: ActSource, pos: Point) -> Vec<bn::Cmd<Ent>> {
        let mut cmds = Vec::new();
        for handle in self.handlers.iter() {
            cmds.append(&mut handle.activate(map, src, pos));
        }
        cmds
    }
}

impl bn::Entity for Ent {
    type Tile = Tile;
    type Vfx = Vfx;

    fn repr(&self) -> <<Self as bandit::Entity>::Tile as bandit::Tile>::Repr {
        if self.movable {
            self.ch.clone()
        } else {
            self.ch.on(IMMOVABLE_CLR)
        }
    }

    fn update(&self, cmd: &mut bandit::Commands<'_, Self>, pos: Point)
        where
            Self: Sized {
        // Make sure everyone knows we updated.
        cmd.queue(bn::Cmd::new_here().modify_entity(Box::new(|e: &mut Ent| e.updated = true)));

        match &self.tp {
            EntType::Player => unsafe { 
                let nx = PLAYER + DIR;
                if Ent::walkable(&cmd.get_map(nx)) {
                    // Push the entity in our way if possible.
                    if let Some(e) = cmd.get_ent(nx) {
                        let nx2 = nx + DIR;
                        // Possible if the location we would push to contains no entity and is
                        // walkable.
                        if Ent::walkable(&cmd.get_map(nx2)) && cmd.get_ent(nx2).is_none() && e.movable {
                            cmd.queue(bn::Cmd::new_on(nx).move_to(nx2));
                            if let Some(t) = cmd.get_map(nx2) {
                                cmd.queue_many(t.activate(cmd, ActSource::WalkOn, nx2));
                            }
                        } else {
                            return;
                        }
                    }
                    if let Some(t) = cmd.get_map(nx) {
                        cmd.queue_many(t.activate(cmd, ActSource::WalkOn, nx));
                    }
                    cmd.queue(bn::Cmd::new_here().displace(DIR));
                    MOVES.write().unwrap().push(Move::new(DIR));
                    PLAYER = nx;
                }
            },
            EntType::Laser(bm) => {
                bm.propagate(cmd, pos);
            }
            // Anyone else has nothing to worry about.
            _ => (),
        }
    }

    fn priority(&self) -> u32 {
        if self.updated {
            0
        } else {
            match self.tp {
                EntType::Player => 10,
                EntType::Obj(_) => 0,
                EntType::Laser(_) => 3,
                EntType::Goal(_) => 2,
            }
        }
    }
}

impl Default for Ent {
    fn default() -> Self {
        Self {
            ch: '@'.white(),
            tp: EntType::Player,
            updated: false,
            movable: true,
            active: false,
            handlers: Vec::new(),
        }
    }
}

