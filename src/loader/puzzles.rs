//! Loading and handling puzzles.

use super::*;
use std::fmt;

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
#[derive(Clone)]
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
            data: bn::Map::new(7, 7),
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

/// A group of puzzles.
#[derive(Clone)]
pub struct PuzzlePack {
    /// Name of the pack.
    pub name: String,
    /// Actual puzzles contained.
    pub pzls: Vec<Puzzle>,
}

impl PuzzlePack {
    /// Create a new puzzle pack with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            pzls: Vec::new(),
        }
    }

    /// Add the puzzle to the pack.
    pub fn add_pzl(&mut self, pzl: Puzzle) {
        self.pzls.push(pzl);
    }
}

/// Turns a string into a map using a tile set.
fn create_map(data: &str, tile_set: &ts::TileSet, default_tile: &Tile) -> Result<PuzzleBuilder, PzlIOErr> {
    let mut map = bn::Map::new(69, 69);
    let mut builder = PuzzleBuilder::new();
    let mut max_x = 0;
    let mut max_y = 0;

    for (y, ln) in data.lines().rev().enumerate() {
        max_y += 1;
        let mut clr = beam::Clr::Black;
        let mut movable = true;
        for (x, ch) in ln.chars().enumerate() {
            max_x = std::cmp::max(max_x, x / 2 + 1);
            if x % 2 == 0 {
                clr = ch.to_ascii_lowercase().try_into()?;
                if ch.is_uppercase() {
                    movable = false;
                }
                continue;
            }

            let pos = Point::new((x / 2) as i32, y as i32);

            // Try to map this character to an object.
            if let Some(obj) = tile_set.map(ch.with(clr.into())) {
                match obj {
                    ts::BanditObj::Tile(t) => map.insert_tile(t.clone(), pos),
                    ts::BanditObj::En(en) => {
                        if en.is_player() {
                            builder.pl_pos.replace(pos);
                        }
                        let mut en = en.clone();
                        en.movable = movable;
                        movable = true;
                        map.insert_entity(en, pos);
                        map.insert_tile(default_tile.clone(), pos);
                    }
                }
            // If we can't, start screaming.
            } else {
                eprintln!("{ch}");
                return Err(PzlIOErr::InvalidFormat);
            }
        }
    }

    map.wid = max_x;
    map.hgt = max_y;
    builder.id.replace(u128::from_ne_bytes(*md5::compute(data.as_bytes())));
    builder.data.replace(map);
    Ok(builder)
}

/// Uses the given tileset to turn a string into a puzzle. Unknown characters will cause an error
/// to be returned.
pub fn load_pzl(
    data: &str,
    default_tile: &Tile,
    tile_set: &ts::TileSet,
    name: String,
) -> Result<Puzzle, PzlIOErr> {
    let mut b = create_map(data, tile_set, default_tile)?;
    b.name = Some(name);

    Ok(Puzzle::try_from(b)?)
}

/// Takes a file and loads all puzzles from it, assuming the file is stored in the correct format.
pub fn load_pzls<P: AsRef<std::path::Path>>(
    fname: P,
    default_tile: &Tile,
    tile_set: &ts::TileSet,
) -> Result<PuzzlePack, PzlIOErr> {
    // Get the name, and err if there isn't one.
    let Some(_name) = fname.as_ref().file_prefix() else { return Err(PzlIOErr::InvalidFile) };

    // Make sure the file has the correct extension.
    if let Some(ext) = fname.as_ref().extension() {
        if ext.to_str().unwrap() != "pzls" {
            return Err(PzlIOErr::InvalidFile);
        }
    } else {
        return Err(PzlIOErr::InvalidFile);
    }

    let mut pzls = PuzzlePack::new(String::new());
    let mut state = -1;
    let mut data = String::new();
    let mut builder = PuzzleBuilder::new();

    for line in read_lines(fname)? {
        let line = line?;
        match state {
            // Read pack name.
            -1 => {
                pzls.name = line.trim().to_string();
                state = 0;
            }
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
                        builder.name.ok_or(PzlIOErr::InvalidFormat)?,
                    )?;
                    pzls.add_pzl(pzl);
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

/// An error when writing/loading a puzzle.
#[derive(Clone, Copy, Debug, Default)]
pub enum PzlIOErr {
    InvalidFile,
    #[default]
    InvalidFormat,
    FileBusy,
    CantAccess,
}

impl From<()> for PzlIOErr {
    fn from(_value: ()) -> Self {
        Self::InvalidFormat
    }
}

impl From<io::Error> for PzlIOErr {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::InvalidFilename => Self::InvalidFile,
            io::ErrorKind::NotFound => Self::CantAccess,
            io::ErrorKind::PermissionDenied => Self::CantAccess,
            io::ErrorKind::InvalidData => Self::InvalidFormat,
            io::ErrorKind::IsADirectory => Self::InvalidFile,
            _ => Self::CantAccess,
        }
    }
}

impl fmt::Display for PzlIOErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let txt = match *self {
            Self::InvalidFile => "Invalid File Type",
            Self::InvalidFormat => "File is Corrupt",
            Self::FileBusy => "File is Busy",
            Self::CantAccess => "Can't Access File",
        };
        
        write!(f, "{txt}")
    }
}
