//! Loading and handling puzzles.

use super::*;

pub mod ts;

/// Initialise a puzzle, returning a clone of its map for use.
pub fn start_puzzle(pzl: &Puzzle) -> bn::Map<Ent> {
    unsafe {
        PLAYER = pzl.pl_pos;
    }
    pzl.data.clone()
}

/// Partially populated puzzle.
#[derive(Default)]
struct PuzzleBuilder {
    /// Map of the puzzle.
    data: Option<bn::Map<Ent>>,
    /// Location of the player in the map.
    pl_pos: Option<Point>,
    /// Puzzle name.
    name: Option<String>,
    /// Unique identifier (md5 hash of the puzzle data).
    id: Option<u128>,
}

impl PuzzleBuilder {
    /// Create an empty puzzle builder.
    fn new() -> Self {
        Self::default()
    }

    /// Check it contains all necessary data for a puzzle.
    fn check(&self) -> bool {
        self.data.is_some()
            && self.pl_pos.is_some()
            && self.name.is_some()
            && self.id.is_some()
    }
}

/// Contains all necessary information for a puzzle.
pub struct Puzzle {
    /// Map of the puzzle.
    pub data: bn::Map<Ent>,
    /// Location of the player in the map.
    pub pl_pos: Point,
    /// Puzzle name.
    pub name: String,
    /// Unique identifier (md5 hash of the puzzle data).
    pub id: u128,
}

impl Puzzle {
    /// Create an empty puzzle with a name.
    pub fn new(name: String) -> Self {
        Self {
            data: bn::Map::new(50, 50),
            pl_pos: Point::ORIGIN,
            name,
            id: 0,
        }
    }
}

impl TryFrom<PuzzleBuilder> for Puzzle {
    type Error = ();

    fn try_from(value: PuzzleBuilder) -> Result<Self, Self::Error> {
        if value.check() {
            Ok(Puzzle {
                data: value.data.unwrap(),
                pl_pos: value.pl_pos.unwrap(),
                name: value.name.unwrap(),
                id: value.id.unwrap(),
            })
        } else {
            Err(())
        }
    }
}

/// Turns a string into a map using a tile set.
fn create_map(data: &str, tile_set: &ts::TileSet, default_tile: &Tile) -> PuzzleBuilder {
    let mut map = bn::Map::new(69, 69);
    let mut builder = PuzzleBuilder::new();

    for (y, ln) in data.lines().rev().enumerate() {
        let mut clr = beam::Clr::Black;
        for (x, ch) in ln.chars().enumerate() {
            if x % 2 == 0 {
                clr = ch.try_into().unwrap();
                continue;
            }

            let pos = Point::new((x / 2) as i32, y as i32);

            if let Some(obj) = tile_set.map(ch.with(clr.into())) {
                match obj {
                    ts::BanditObj::Tile(t) => map.insert_tile(t.clone(), pos),
                    ts::BanditObj::En(en) => {
                        if en.is_player() {
                            builder.pl_pos.replace(pos);
                        }
                        map.insert_entity(en.clone(), pos);
                        map.insert_tile(default_tile.clone(), pos);
                    }
                }
            }
        }
    }

    builder.id.replace(u128::from_ne_bytes(*md5::compute(data.as_bytes())));
    builder.data.replace(map);
    builder
}

/// Uses the given tileset to turn a string into a puzzle. Unknown characters will be ignored.
pub fn load_pzl(
    data: &str,
    default_tile: &Tile,
    tile_set: &ts::TileSet,
    name: String,
) -> Result<Puzzle, ()> {
    let mut b = create_map(data, tile_set, default_tile);
    b.name = Some(name);

    Ok(Puzzle::try_from(b)?)
}

/// Takes a file and loads all puzzles from it, assuming the file is stored in the correct format.
pub fn load_pzls<P: AsRef<std::path::Path>>(
    fname: P,
    default_tile: &Tile,
    tile_set: &ts::TileSet,
) -> Result<Vec<Puzzle>, ()> {
    let mut pzls = Vec::new();
    let mut state = 0;
    let mut data = String::new();
    let mut builder = PuzzleBuilder::new();

    for line in read_lines(fname).unwrap() {
        let line = line.unwrap();
        match state {
            // Read name.
            0 => {
                builder.name = Some(line);

                state = 1;
            }
            // Add lines to data until an empty line is found, then load the puzzle.
            1 => {
                if line.is_empty() {
                    let pzl = load_pzl(
                        &data,
                        default_tile,
                        tile_set,
                        builder.name.unwrap(),
                    )?;
                    pzls.push(pzl);
                    data = String::new();
                    builder = PuzzleBuilder::new();
                    state = 0;
                } else {
                    data.push_str(&line);
                    data.push('\n');
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(pzls)
}

