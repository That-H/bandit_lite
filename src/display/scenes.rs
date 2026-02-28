//! Ui scenes.

use crate::loader::get_assets_path;

use super::*;
use windowed::ui;
use std::{fs, time};
use std::io::Read;
use std::collections::HashSet;

const MAIN_MENU_SIZE: (usize, usize) = (20, 4);
const MAIN_MENU_POS: Point = Point::new(centre(MAIN_MENU_SIZE.0), 12);
const PZL_SIZE: (usize, usize) = (24, 17);
const PZL_POS: Point = Point::new(centre(PZL_SIZE.0), 12);
const END_SIZE: (usize, usize) = (20, 5);
const END_POS: Point = Point::new(centre(END_SIZE.0), 12);
const SELECTOR: &str = ">";
const HOVER_CLR: style::Color = style::Color::Yellow;
const SELECTOR_CLR: style::Color = HOVER_CLR;
const DELAY: time::Duration = time::Duration::from_millis(35);

/// Exit code for playing the game.
pub const PLAY: u32 = 0;
/// Exit code for saving and quitting.
pub const SAVE_AND_QUIT: u32 = 1;
/// Exit code for returning to the main menu.
pub const MAIN_MENU: u32 = 2;
/// Exit code for immediately playing the next puzzle.
pub const NEXT: u32 = 3;

/// Turn a width into a centred x position on the terminal.
const fn centre(wid: usize) -> i32 {
    (TERMINAL_WID / 2 - wid as u16 / 2) as i32
}

/// Make the given file into a title and put it in the scene.
fn add_title<P: AsRef<std::path::Path>>(fname: P, scene: &mut ui::Scene, y: i32) {
    let mut f = fs::File::open(get_assets_path().join(fname)).unwrap();
    let mut text = String::new();
    let _ = f.read_to_string(&mut text);
    let wid = text.lines().next().unwrap().len();

    let title = ui::widgets::Title::new(Point::new(centre(wid), y), text, Some(DELAY));
    scene.add_element(Box::new(title), Point::new(500, 500));
}

/// Get the character used to outline scenes.
fn outline_ch() -> StyleCh {
    '#'.grey()
}

/// Standard button.
fn basic_button() -> ui::widgets::Button {
    ui::widgets::Button::empty_new()
        .set_selector(String::from(SELECTOR))
        .set_hover_clr(HOVER_CLR)
        .set_selector_clr(SELECTOR_CLR)
        .set_static_len(true)
}

/// Standard entry box (probably won't need this).
fn _basic_entry() -> ui::widgets::TextEntry {
    ui::widgets::TextEntry::new()
        .set_hover_clr(HOVER_CLR)
        .set_highlight_clr(style::Color::Cyan)
        .set_active_clr(HOVER_CLR)
}

/// Adds an outline to the scene.
fn add_outline(scene: &mut ui::Scene, wid: usize) {
    scene.add_element(Box::new(ui::widgets::Outline::new(outline_ch(), wid)), Point::new(999, 999));
}

/// Create a ui scene for the main menu.
pub fn main_menu() -> ui::Scene {
    let mut scene = ui::Scene::new(MAIN_MENU_POS, MAIN_MENU_SIZE.0, MAIN_MENU_SIZE.1);

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Play"))
                .set_events(
                    vec![
                        ui::Event::Broadcast(String::from("clr")),
                        ui::Event::ChangeScene(1),
                    ]
                )
                .set_screen_pos(Point::new(1, 1)),
        ),
        Point::new(1, 1),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Save and Quit"))
                .set_event(ui::Event::Exit(SAVE_AND_QUIT))
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.move_cursor(Point::new(1, 1));
    add_outline(&mut scene, MAIN_MENU_SIZE.0);
    add_title("title.txt", &mut scene, 1);
    scene
}

/// Create a ui scene for puzzle selection.
pub fn puzzle_select(pzls: &[loader::puzzles::Puzzle], completion: &HashSet<u128>) -> ui::Scene {
    // Puzzle selection screen.
    let mut pzl_scene = ui::Scene::new(PZL_POS, PZL_SIZE.0, PZL_SIZE.1).with_scrolling(true);
    
    add_title("puzzle_title.txt", &mut pzl_scene, 1);

    // Add an indicator for current completion status.
    let pzl_count = pzls.len();
    let completed = completion.len();
    pzl_scene.add_element(
        Box::new(
            basic_button()
                .set_txt(format!("Completed: {}/{}", completed, pzl_count))
                .set_screen_pos(Point::new(1, 2))
        ),
        Point::new(-1, -1)
    );
    pzl_scene.add_element(
        Box::new(
            basic_button()
                .set_txt(format!("Completion: {:.1}%", completed as f64 / pzl_count as f64 * 100.0))
                .set_screen_pos(Point::new(1, 1))
        ),
        Point::new(-2, -1)
    );

    // Last section.
    let mut last_sect = -1;

    for (n, pzl) in pzls.iter().enumerate() {
        let mut pos = Point::new(1, n as i32 + 2);
        let mut screen_pos = pos + Point::new(0, last_sect + 4);

        // New difficulty block found
        if n % 8 == 0 {
            last_sect += 1;
            let clr = match last_sect {
                0 => style::Color::Green,
                1 => style::Color::Yellow,
                2 => style::Color::Red,
                3 => style::Color::DarkRed,
                4 => style::Color::DarkMagenta,
                d => panic!("Unexpected section '{d}'"),
            };
            pzl_scene.add_element(
                Box::new(
                    basic_button()
                        .set_txt(format!("Section {}", last_sect))
                        .set_clr(clr)
                        .set_screen_pos(screen_pos),
                ),
                pos + Point::new(500, 5),
            );
            screen_pos.y += 1;
        }

        let txt_clr = if completion.contains(&pzl.id) {
            style::Color::Rgb { r: 50, g: 255, b: 0 }
        } else {
            style::Color::White
        };

        pzl_scene.add_element(
            Box::new(
                basic_button()
                    .set_txt(format!("{}", pzl.name))
                    .set_clr(txt_clr)
                    .set_event(ui::Event::Exit(n as u32 + 2000))
                    .set_screen_pos(screen_pos),
            ),
            pos,
        );
        pos = pos + Point::new(0, 1);

        // Add a main menu button.
        if n == pzls.len() - 1 {
            pzl_scene.add_element(
                Box::new(
                    basic_button()
                        .set_txt(String::from("Main Menu"))
                        .set_events(vec![
                            ui::Event::Broadcast(String::from("clr")),
                            ui::Event::ChangeScene(0),
                        ])
                        .set_screen_pos(screen_pos + Point::new(0, 2)),
                ),
                pos 
            );
            break
        }
    }
    add_outline(&mut pzl_scene, PZL_SIZE.0);

    pzl_scene.move_cursor(Point::new(1, 2));
    pzl_scene
}

/// End screen when a puzzle is completed.
pub fn end_screen() -> ui::Scene {
    let mut scene = ui::Scene::new(END_POS, END_SIZE.0, END_SIZE.1);

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Next Puzzle"))
                .set_event(ui::Event::Exit(SAVE_AND_QUIT))
                .set_screen_pos(Point::new(1, 1)),
        ),
        Point::new(1, 1),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Puzzle Select"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::ChangeScene(1),
                ])
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Main Menu"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::ChangeScene(0),
                ])
                .set_screen_pos(Point::new(1, 3)),
        ),
        Point::new(1, 3),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Save and Quit"))
                .set_event(ui::Event::Exit(SAVE_AND_QUIT))
                .set_screen_pos(Point::new(1, 4)),
        ),
        Point::new(1, 4),
    );

    add_outline(&mut scene, END_SIZE.0);

    scene.move_cursor(Point::new(1, 2));

    scene
}
