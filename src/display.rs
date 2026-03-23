//! Contains utilities for displaying parts of the game.
use super::*;

pub mod scenes;

/// Width of the puzzle title window.
pub const TITLE_WID: usize = 20;

/// Display the title of the puzzle.
pub fn display_title(pzl_name: &str, cont: &mut windowed::Container<StyleCh>) {
    let cur_win = &mut cont.windows[1];
    cur_win.data.clear();
    let mut ln = Vec::new();
    ln.push(' '.stylize());
    for ch in pzl_name.chars() {
        ln.push(ch.white());
    }
    for _ in ln.len()..TITLE_WID - 2 {
        ln.push(' '.stylize());
    }
    cur_win.data.push(vec![' '.stylize(); TITLE_WID - 2]);
    cur_win.data.push(ln);
    cur_win.data.push(vec![' '.stylize(); TITLE_WID - 2]);
    cur_win.outline_with('#'.grey());
}

/// Immediately displays the map into the container, then displays it on to the screen.
pub fn display_all(map: &bn::Map<Ent>, cont: &mut windowed::Container<StyleCh>, centre: Point, pzl_name: &str) {
    display_map(map, cont, centre);
    display_title(pzl_name, cont);
    cont.refresh();
    print_win(cont);
}

/// Displays the contents of a map into a window centred on the centre.
pub fn display_map(map: &bn::Map<Ent>, cont: &mut windowed::Container<StyleCh>, centre: Point) {
    let top_left =
        centre - Point::new(GAME_WID as i32 / 2, -(GAME_HGT as i32) / 2 - 1);
    let cur_win = &mut cont.windows[0];
    map.display_into(cur_win, top_left, GAME_WID as u32, GAME_HGT as u32);
    cur_win.outline_with('#'.grey());
}

/// Display a window container into the terminal window.
pub fn print_win(win_cont: &windowed::Container<style::StyledContent<char>>) {
    let mut handle = io::stdout();
    let buf = win_cont.get_buffer();

    for change in win_cont.changed() {
        let _ = queue!(handle, cursor::MoveTo(change.x as u16, change.y as u16), style::Print(buf[change]));
    }

    let _ = handle.flush();
}

/// Colours the text with the given colour and puts it into the window. Ensures at least len styled characters
/// are contained within the line.
pub fn add_line(
    clr: style::Color,
    txt: &str,
    win: &mut windowed::Window<style::StyledContent<char>>,
    len: usize
) {
    let mut line = vec![' '.stylize()];
    for ch in txt.chars() {
        line.push(ch.with(clr));
    }
    let line_len = line.len();
    if line_len < len {
        for _ in 0..len - line_len {
            line.push(' '.stylize());
        }
    }

    win.data.push(line);
}
