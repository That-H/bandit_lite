use super::*;

const DIAG_ARROWS: [char; 8] = ['↖', '↑', '↗', '→', '↘', '↓', '↙', '←'];
/// Background colour of immovable objects.
pub const IMMOVABLE_CLR: style::Color = style::Color::DarkGrey;

/// A move that the player did.
#[derive(Clone, Copy, Debug)]
pub struct Move {
    /// Direction of the move.
    pub dir: Point,
    /// Whether it pushed anything or not.
    pub push: bool
}

impl Move {
    /// Create a move.
    pub const fn new(dir: Point, push: bool) -> Self {
        Self {
            dir, 
            push,
        }
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
    /// Takes an input of the given colour to any port. When this occurs for all of this type of
    /// object present, the player completes the puzzle.
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
}

impl Ent {
    /// Create an entity that is the player.
    pub fn player() -> Self {
        Self {
            ch: '@'.white(),
            tp: EntType::Player,
            updated: false,
            movable: true,
        }
    }

    /// Create an entity that fires a laser.
    pub fn laser(dir: Point, clr: beam::Clr) -> Self {
        Self {
            ch: DIAG_ARROWS[beam::port_num(-dir)].with(clr.into()),
            tp: EntType::Laser(beam::Beam::new(clr, dir)),
            updated: false,
            movable: true,
        }
    }

    /// Create a goal entity.
    pub fn goal(clr: beam::Clr) -> Self {
        Self {
            ch: 'O'.with(clr.into()),
            tp: EntType::Goal(clr),
            updated: false,
            movable: true,
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
            movable: true,
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
                // We undo.
                if DIR == Point::ORIGIN {
                    if let Some(mv) = MOVES.write().unwrap().pop() {
                        let nx = pos - mv.dir;
                        cmd.queue(bn::Cmd::new_here().move_to(nx));
                        PLAYER = nx;
                        if mv.push {
                            cmd.queue(bn::Cmd::new_on(pos + mv.dir).move_to(pos));
                        }
                        return;
                    } else {
                        // Nothing to undo.
                        return;
                    }
                }
                let nx = PLAYER + DIR;
                let mut pushed = false;
                if Ent::walkable(&cmd.get_map(nx)) {
                    // Push the entity in our way if possible.
                    if let Some(e) = cmd.get_ent(nx) {
                        let nx2 = nx + DIR;
                        // Possible if the location we would push to contains no entity and is
                        // walkable.
                        if Ent::walkable(&cmd.get_map(nx2)) && cmd.get_ent(nx2).is_none() && e.movable {
                            cmd.queue(bn::Cmd::new_on(nx).move_to(nx2));
                            pushed = true;
                        } else {
                            return;
                        }
                    }
                    cmd.queue(bn::Cmd::new_here().displace(DIR));
                    MOVES.write().unwrap().push(Move::new(DIR, pushed));
                    PLAYER = nx;
                }
            },
            EntType::Laser(bm) => {
                bm.propagate(cmd, pos);
            }
            EntType::Goal(_) => {
                unsafe { 
                    SHOULD_WIN = false;
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
                EntType::Laser(_) => 3,
                EntType::Goal(_) => 2,
            }
        }
    }
}

