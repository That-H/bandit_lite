//! Loading objects stored in files.

use super::*;
use std::fs;
use std::io::BufRead;

pub mod puzzles;
pub mod saver;

const ASSETS_DIR: &str = "assets";
const OBJS: &str = "objs.txt";
const PZLS: &str = "pzls.pzls";

/// Load builtin puzzles.
pub fn load_standard_pzls(default_tile: &Tile, ts: &puzzles::ts::TileSet) -> Vec<puzzles::Puzzle> {
    let assets = get_assets_path();

    // Assume standard puzzles are formatted correctly.
    puzzles::load_pzls(assets.join(PZLS), default_tile, ts).unwrap()
}

/// Load some objects from the given file.
pub fn load_objs() -> puzzles::ts::TileSet {
    let mut chs = Vec::new();
    let mut exprs = vec![port::Expr::Null; 8];
    let mut ts = puzzles::ts::TileSet::new();
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
            ts.add_entity(obj.clone());
            for ch in chs {
                obj.rotate_90();
                obj.ch = ch;
                ts.add_entity(obj.clone());
            }
            chs = Vec::new();
            exprs = vec![port::Expr::Null; 8];
        }
    }

    ts
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

