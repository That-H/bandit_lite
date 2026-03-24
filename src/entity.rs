use super::*;
use loader::puzzles;

const DIAG_ARROWS: [char; 8] = ['↖', '↑', '↗', '→', '↘', '↓', '↙', '←'];
/// Background colour of immovable objects.
pub const IMMOVABLE_CLR: style::Color = style::Color::DarkGrey;

mod activation;
pub use activation::*;

/// A move that the player did.
#[derive(Clone, Copy, Debug)]
pub struct Move(pub Point);

impl Move {
    /// Create a move.
    pub const fn new(dir: Point) -> Self {
        Self(dir)
    }
}

/// Encodes the behaviour of an entity.
#[derive(Clone, Debug)]
pub enum EntType {
    /// Is the player.
    Player,
    /// Fires a coloured laser beam.
    Laser,
    /// Uses its inputs to potentially create an output.
    Obj(port::PortGrp),
    /// Takes an input of the given colour to any port. This activates all surrounding
    /// tiles/entities.
    Goal(beam::Clr),
    /// Something else that is driven by its activation handlers.
    Other,
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
    /// True if the entity was being activated during the previous frame.
    pub prev_active: bool,
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
        let bm = beam::Beam::new(clr, dir);
        Self {
            ch: DIAG_ARROWS[beam::port_num(-dir)].with(clr.into()),
            tp: EntType::Laser,
            handlers: vec![
                ActHandler::new(ActSource::LaserTime, ActEffect::Laser(bm)),
            ],
            ..Default::default()
        }
    }

    /// Create an entity that requires activation to fire a laser.
    pub fn inact_laser(dir: Point, clr: beam::Clr) -> Self {
        let laser = ActEffect::Laser(beam::Beam::new(clr, dir));
        let handlers = vec![
            ActHandler::new(ActSource::FrameEnd, ActEffect::Reset),
            ActHandler::new(ActSource::FrameStart, Cond::EPrevActive.cond_ef(laser.clone())),
            ActHandler::new(ActSource::Obj, Cond::EActive.not().cond_ef(laser.clone().chain(ActEffect::MkActive))),
        ];

        Self {
            ch: DIAG_ARROWS[beam::port_num(-dir)].with(clr.darker()),
            tp: EntType::Other,
            handlers,
            ..Default::default()
        }
    }

    /// Create a goal entity.
    pub fn goal(clr: beam::Clr) -> Self {
        let handlers = vec![
            ActHandler::new(ActSource::Laser, Cond::EActive.not().cond_ef(ActEffect::Prop).chain(ActEffect::MkActive)),
            ActHandler::new(ActSource::FrameEnd, ActEffect::Reset),
        ];
        Self {
            ch: 'O'.with(clr.into()),
            tp: EntType::Goal(clr),
            handlers,
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
        let mut ch1: char = puzzles::map_clr(self.ch.style().foreground_color.unwrap()).unwrap();
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

    fn update(&self, cmd: &mut bandit::Commands<'_, Self>, _pos: Point)
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
                        cmd.queue_many(t.activate(cmd, ActSource::PlWalkOn, nx));
                    }
                    cmd.queue(bn::Cmd::new_here().displace(DIR));
                    MOVES.write().unwrap().push(Move::new(DIR));
                    PLAYER = nx;
                }
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
                EntType::Laser => 0,
                EntType::Goal(_) => 0,
                EntType::Other => 0,
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
            prev_active: false,
            handlers: Vec::new(),
        }
    }
}

