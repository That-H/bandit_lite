use super::*;

pub mod loader;

/// Encodes the behaviour of an entity.
#[derive(Clone, Debug)]
pub enum EntType {
    /// Is the player.
    Player,
    /// Fires a coloured laser beam in the given direction.
    Laser(beam::Beam),
    /// Uses its inputs to potentially create an output.
    Obj(port::PortGrp),
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

    /// Create an object that takes laser inputs.
    pub fn obj(ch: StyleCh, exprs: Vec<port::Expr>) -> Self {
        Self {
            ch,
            tp: EntType::Obj(
                port::PortGrp::from_iter(
                    exprs.into_iter()
                )
            ),
            updated: false,
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
        // Make sure everyone knows we updated.
        cmd.queue(bn::Cmd::new_here().modify_entity(Box::new(|e: &mut Ent| e.updated = true)));

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
            // Anyone else has nothing to worry about.
            _ => (),
        }
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

