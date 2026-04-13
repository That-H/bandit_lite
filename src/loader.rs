//! Loading objects stored in files.

use super::*;
use std::fs;
use std::io::BufRead;
use std::ops;

pub mod puzzles;
pub mod saver;
use puzzles::ts::BanditObj;

const ASSETS_DIR: &str = "assets";
const OBJS: &str = "objs.txt";
const PZLS: &str = "pzls.pzls";

/// Load builtin puzzles.
pub fn load_standard_pzls(default_tile: &Tile, ts: &puzzles::ts::TileSet) -> puzzles::PuzzlePack {
    let assets = get_assets_path();

    // Assume standard puzzles are formatted correctly.
    puzzles::load_pzls(assets.join(PZLS), default_tile, ts).unwrap()
}

/// Load all user created puzzle packs.
pub fn load_custom_pzls(default_tile: &Tile, ts: &puzzles::ts::TileSet) -> Vec<puzzles::PuzzlePack> {
    let mut packs = Vec::new();
    let save_dir = saver::get_save_path();
    let custom_pzl_path = save_dir.join(loader::saver::PACK_SAVE_DIR);
    eprintln!("{custom_pzl_path:?}");

    let mut fnames = Vec::new();

    if let Ok(d) = fs::read_dir(custom_pzl_path) {
        for f in d {
            let f = f.unwrap();
            let path = f.path();

            if path.is_file() {
                fnames.push(path);
            }
        }
    }

    fnames.sort();

    // Assume they are formatted correctly.
    for fname in fnames {
        packs.push(puzzles::load_pzls(fname, default_tile, ts).unwrap());        
    }

    packs
}

/// Load some objects from the given file.
pub fn load_objs() -> ObjList {
    let mut chs = Vec::new();
    let mut exprs = vec![port::Expr::Null; 8];
    let mut ents = ObjList::new();
    let assets = get_assets_path();

    for line in read_lines(assets.join(OBJS)).unwrap() {
        let line = line.unwrap();

        // This line must be the start of a new definition.
        if chs.is_empty() {
            let mut temp_chs = Vec::new();
            'ch_finder: for (n, part) in line.split(" ").enumerate() {
                if n == 0 {
                    for s in part.split(",") {
                        let ch = s.chars().next().unwrap();
                        temp_chs.push(ch);
                    }
                } else {
                    let clr = part.parse().unwrap();
                    for &ch in temp_chs.iter() {
                        chs.push(ch.with(clr));
                    }
                    break 'ch_finder;
                }
            }
        // Otherwise we in an expression line.
        } else if !line.is_empty() {
            let port = line.chars().next().unwrap().to_digit(8).unwrap() as usize;
            exprs[port] = line[2..].parse().unwrap();
        // Empty line means we should turn what we saw into an object.
        } else {
            let mut obj = Ent::obj(chs.remove(0), exprs);
            let mut this = vec![obj.clone()];
            for ch in chs {
                obj.rotate_90();
                obj.ch = ch;
                this.push(obj.clone());
            }
            ents.add_entities(this);
            chs = Vec::new();
            exprs = vec![port::Expr::Null; 8];
        }
    }

    ents
}

/// A list of objects.
#[derive(Clone, Debug)]
pub struct ObjList(pub Vec<Vec<BanditObj>>);

impl ObjList {
    /// Create an empty object list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a single tile to the end of the list.
    pub fn add_tile(&mut self, tile: Tile) {
        self.0.push(vec![BanditObj::Tile(tile)]);
    }
    
    /// Add a single entity to the end of the list.
    pub fn add_entity(&mut self, ent: Ent) {
        self.0.push(vec![BanditObj::En(ent)]);
    }

    /// Add some entities to the list, assuming they are rotations of each other.
    pub fn add_entities<I: IntoIterator<Item=Ent>>(&mut self, ents: I) {
        self.0.push(ents.into_iter().map(BanditObj::from).collect())
    }
}

impl ops::Deref for ObjList {
    type Target = Vec<Vec<BanditObj>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ObjList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ObjList {
    fn default() -> Self {
        Self::new()
    }
}

/// Return a buffered reader over the lines of a file.
pub fn read_lines<P: AsRef<std::path::Path>>(
    path: P,
) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

/// Return the path to the assets directory of the project.
pub fn get_assets_path() -> std::path::PathBuf {
    let mut this_path = std::env::current_exe().expect("Failed to get path to project");
    for _ in 0..3 {
        this_path.pop();
    }
    this_path.push(ASSETS_DIR);

    this_path
}

