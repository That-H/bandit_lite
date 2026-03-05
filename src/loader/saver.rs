//! Handles saving and loading puzzle saves.

use std::collections::HashSet;
use std::fs;
use std::io::{self, ErrorKind, Write};

use crate::{Point, loader};
use loader::puzzles;

const QUAL: &str = "";
const ORGANISATION: &str = "Uranium Productions";
const APP: &str = "Bandit Lite";
const PZLS_FILE: &str = "completed_pzls.txt";
/// Directory in which custom puzzle packs are located.
pub const PACK_SAVE_DIR: &str = "packs";
const SUPER_SECRET_NAME: u128 = 0xd39b1b9ee3753f659fb0d1900b2136d1;

/// Load all solved puzzle hashes.
pub fn load_pzl_save() -> HashSet<u128> {
    let mut set = HashSet::new();
    let lines = match super::read_lines(get_pzl_path()) {
        Ok(lns) => lns,
        Err(e) => match e.kind() {
            // Must not have a save file yet, so no completion to read.
            ErrorKind::NotFound => return set,
            e => panic!("Error reading save file: {e:?}"),
        },
    };

    for line in lines.map_while(Result::ok) {
        for id in line.trim().split(" ") {
            set.insert(u128::from_str_radix(id, 16).unwrap());
        }
    }

    set
}

/// Write the current state of completion to the save file.
pub fn write_pzl_save(data: HashSet<u128>) {
    let mut p = get_pzl_path();
    p.pop();
    fs::create_dir_all(&p).expect("Can't create the directories");
    let mut file =
        io::BufWriter::new(fs::File::create(get_pzl_path()).expect("Unable to write save file"));

    for comp in data {
        file.write_all(format!("{comp:x} ").as_bytes())
            .expect("Unable to write save file");
    }

    file.flush().expect("Unable to flush save file");
}

/// Get the path to the save directory.
pub fn get_save_path() -> std::path::PathBuf {
    directories::ProjectDirs::from(QUAL, ORGANISATION, APP)
        .unwrap()
        .data_local_dir()
        .to_path_buf()
}

/// Write a puzzle pack to the appropriate location. Note: the wid and hgt of the maps contained in
/// the puzzles must be accurate!
/// Also writes to the standard puzzles if the pack has the secret name.
pub fn write_pzls(pack: &puzzles::PuzzlePack) -> Result<(), puzzles::PzlIOErr> {
    let path = if is_secret(&pack.name) {
        loader::get_assets_path().join(loader::PZLS)
    } else {
        get_save_path().join(PACK_SAVE_DIR).join(&format!("{}.pzls", pack.name))
    };

    let mut data = String::new();

    for pzl in pack.pzls.iter() {
        data.push_str(&pzl.name);
        data.push('\n');
        for y in (0..pzl.data.hgt).rev() {
            for x in 0..pzl.data.wid {
                let p = Point::new(x as i32, y as i32);
                if let Some(e) = pzl.data.get_ent(p) {
                    data.push_str(&e.file_repr());
                } else if let Some(t) = pzl.data.get_map(p) {
                    data.push_str(&t.file_repr());
                }
            }
            data.push('\n');
        }
        data.push('\n');
    }

    let _ = fs::create_dir_all(&path);
    
    // Hack to stop it from making smth.pzls into a directory.
    let _ = fs::remove_dir(&path);
    let mut file = fs::File::create(&path)?;
    file.write_all(data.as_bytes())?;

    Ok(())
}

/// Get the path to the puzzle save file.
fn get_pzl_path() -> std::path::PathBuf {
    let pro_dirs = get_save_path();

    pro_dirs.join(PZLS_FILE)
}

/// Check if a string is the super secret string.
pub fn is_secret(string: &str) -> bool {
    let hash = u128::from_be_bytes(*md5::compute(&string));

    hash == SUPER_SECRET_NAME
}

/// Erase the file that stores a puzzle pack. No checks for the secret pack as we will not delete
/// the standard puzzles.
pub fn delete_pack(name: String) -> Result<(), puzzles::PzlIOErr> {
    let path = get_save_path().join(PACK_SAVE_DIR).join(&format!("{}.pzls", name));

    fs::remove_file(path)?;
    Ok(())
}
