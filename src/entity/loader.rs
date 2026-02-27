//! Loading objects stored in files.

use super::*;
use std::fs;
use std::io::BufRead;

const ASSETS_DIR: &str = "assets";
const OBJS: &str = "objs.txt";

/// Load some objects from the given file.
pub fn load_objs() -> Vec<entity::Ent> {
    let mut ch = None;
    let mut exprs = vec![port::Expr::Null; 8];
    let mut ents = Vec::new();
    let assets = get_assets_path();

    for line in read_lines(assets.join(OBJS)).unwrap() {
        let line = line.unwrap();

        // This line must be the start of a new definition.
        if ch.is_none() {
            let mut temp_ch = None;
            for part in line.split(" ") {
                if temp_ch.is_none() {
                    temp_ch = Some(part.chars().next().expect("unexpected empty line"));
                } else {
                    ch = Some(temp_ch.unwrap().with(part.parse().unwrap()));
                }
            }
        // Otherwise we in an expression line.
        } else if !line.is_empty() {
            let port = line.chars().next().unwrap().to_digit(8).unwrap() as usize;
            exprs[port] = line[2..].parse().unwrap();
        // Empty line means we should turn what we saw into an object.
        } else {
            ents.push(Ent::obj(ch.take().unwrap(), exprs));
            exprs = vec![port::Expr::Null; 8];
        }
    }

    ents
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

