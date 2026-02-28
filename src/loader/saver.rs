//! Handles saving and loading puzzle saves.

use std::collections::HashSet;
use std::fs;
use std::io::{self, ErrorKind, Write};

const QUAL: &str = "";
const ORGANISATION: &str = "Uranium Productions";
const APP: &str = "Bandit Lite";
const PZLS_FILE: &str = "completed_pzls.txt";

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
        let mut id = 0;
        for (n, ch) in line.chars().enumerate() {
            id <<= 4;
            id += ch.to_digit(16).unwrap() as u128;
            if n % 32 == 0 && n != 0 {
                set.insert(id);
            }
        }
        set.insert(id);
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
        file.write_all(format!("{comp:x}").as_bytes())
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

// Get the path to the puzzle save file.
fn get_pzl_path() -> std::path::PathBuf {
    let pro_dirs = get_save_path();

    pro_dirs.join(PZLS_FILE)
}
