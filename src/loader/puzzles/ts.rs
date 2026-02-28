//! Converting characters into bandit compatible objects.

use std::collections::HashMap;
use super::*;

/// An object in a bandit map.
pub enum BanditObj {
    Tile(Tile),
    En(Ent),
}

impl BanditObj {
    /// Assume this is an En variant and return a reference to the en value within. If it is
    /// actually a tile variant, panics.
    pub fn assume_en(&self) -> &Ent {
        match self {
            Self::Tile(_) => panic!("Assumed a Tile variant was an En variant"),
            Self::En(en) => en,
        }
    }
}

/// Mapping of characters to tiles or entities.
pub struct TileSet(pub HashMap<(char, style::Color), BanditObj>);

impl TileSet {
    /// Create an empty tile set.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert an object into the tile set.
    pub fn insert(&mut self, obj: BanditObj) {
        match obj {
            BanditObj::En(en) => self.add_entity(en),
            BanditObj::Tile(t) => self.add_tile(t),
        }
    }

    /// Add the tile into the tile set.
    pub fn add_tile(&mut self, tile: Tile) {
        self.0.insert(Self::key(tile.ch), BanditObj::Tile(tile));
    }

    /// Add the entity into the tile set.
    pub fn add_entity(&mut self, en: Ent) {
        self.0.insert(Self::key(en.ch), BanditObj::En(en));
    }

    /// Get a reference to what the given character should be, if anything.
    pub fn map(&self, ch: StyleCh) -> Option<&BanditObj> {
        self.0.get(&Self::key(ch))
    }

    /// Turn a StyleCh into a valid key.
    fn key(ch: StyleCh) -> (char, style::Color) {
        (*ch.content(), ch.style().foreground_color.unwrap_or(style::Color::White))
    }
}
