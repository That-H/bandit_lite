//! Ui scenes.

use crate::loader::get_assets_path;

use super::*;
use windowed::ui;
use std::{fs, time};
use std::io::Read;
use std::collections::HashSet;

const MAIN_MENU_SIZE: (usize, usize) = (20, 5);
const MAIN_MENU_POS: Point = Point::new(centre(MAIN_MENU_SIZE.0), 12);
const PZL_SIZE: (usize, usize) = (24, 18);
const PZL_POS: Point = Point::new(centre(PZL_SIZE.0) - 15, 10);
const PREVIEW_POS: Point = Point::new(PZL_POS.x + PZL_SIZE.0 as i32 + 3, PZL_POS.y);
const PREVIEW_SIZE: (usize, usize) = PZL_SIZE;
const END_SIZE: (usize, usize) = (20, 6);
const END_POS: Point = Point::new(centre(END_SIZE.0), 14);
const PAUSE_SIZE: (usize, usize) = (20, 7);
const PAUSE_POS: Point = Point::new(centre(PAUSE_SIZE.0), 14);
const PACK_SIZE: (usize, usize) = (20, 11);
const PACK_POS: Point = Point::new(centre(PACK_SIZE.0), 14);
const ENTRY_SIZE: (usize, usize) = (20, 4);
const ENTRY_POS: Point = Point::new(centre(ENTRY_SIZE.0), 14);
const OPTS_SIZE: (usize, usize) = (10, 6);
const OPTS_POS: Point = Point::new(centre(OPTS_SIZE.0), 14);
const EDIT_MENU_SIZE: (usize, usize) = (12, 6);
const EDIT_MENU_POS: Point = Point::new(centre(EDIT_MENU_SIZE.0), 14);
const WARN_WID: usize = 20;
const WARN_POS: Point = Point::new(centre(WARN_WID), 14);
const WID_HGT_SIZE: (usize, usize) = (20, 7);
const WID_HGT_POS: Point = Point::new(centre(WID_HGT_SIZE.0), 14);
const SELECTOR: &str = ">";
const HOVER_CLR: style::Color = style::Color::Yellow;
const SELECTOR_CLR: style::Color = HOVER_CLR;
const DELAY: time::Duration = time::Duration::from_millis(35);
const SECTION_SIZES: [usize; SECTION_COUNT+1] = [
    0,
    3
];
const SECTION_COUNT: usize = 1;

/// Exit code for playing the game.
pub const PLAY: u32 = 0;
/// Exit code for saving and quitting.
pub const SAVE_AND_QUIT: u32 = 1;
/// Exit code for returning to the main menu.
pub const MAIN_MENU: u32 = 2;
/// Exit code for immediately playing the next puzzle.
pub const NEXT: u32 = 3;
/// Exit code for going to puzzle select screen (necessary so you see the title).
pub const PUZZLE_SEL: u32 = 4;
/// Exit code for editing a puzzle.
pub const EDITOR: u32 = 5;
/// Exit code for creating a new puzzle pack.
pub const NEW_PACK: u32 = 6;
/// Exit code for creating a new puzzle.
pub const NEW_PUZZLE: u32 = 7;
/// Exit code saying we didn't do anything.
pub const CANCEL: u32 = 8;
/// Exit code saying we did do something.
pub const CONFIRM: u32 = 9;
/// Exit code for renaming.
pub const RENAME: u32 = 10;
/// Exit code for modification.
pub const MODIFY: u32 = 11;
/// Exit code for deletion.
pub const DEL: u32 = 12;
/// Exit code for saving progress, but not quitting.
pub const SAVE: u32 = 13;
/// Exit code to reset the puzzle to the initial state.
pub const RESET: u32 = 14;

/// Turn a width into a centred x position on the terminal.
const fn centre(wid: usize) -> i32 {
    (TERMINAL_WID / 2 - wid as u16 / 2) as i32
}

mod linked_button;
mod counter;

mod utils;
use utils::*;

pub mod level_editor;

pub mod presets;

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
                .set_txt(String::from("Editor"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(EDITOR),
                ])
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Save and Quit"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(SAVE_AND_QUIT),
                ])
                .set_screen_pos(Point::new(1, 3)),
        ),
        Point::new(1, 3),
    );
    scene.move_cursor(Point::new(1, 1));
    add_outline(&mut scene, MAIN_MENU_SIZE.0);
    add_title("title.txt", &mut scene, 1);
    scene
}

/// Create a ui scene for puzzle selection.
pub fn puzzle_select(
    pzls: &loader::puzzles::PuzzlePack,
    completion: &HashSet<u128>,
    sectioning: bool,
    editing: bool,
) -> ui::Scene {
    // Puzzle selection screen.
    let mut pzl_scene = ui::Scene::new(PZL_POS, PZL_SIZE.0, PZL_SIZE.1).with_scrolling(true);
    
    add_title("puzzle_title.txt", &mut pzl_scene, 1);

    // Add an indicator for current completion status.
    let pzl_count = pzls.pzls.len();
    let mut completed = 0;
    for pzl in pzls.pzls.iter() {
        if completion.contains(&pzl.id) {
            completed += 1;
        }
    }
    pzl_scene.add_element(
        Box::new(
            basic_button()
                .set_txt(format!("Completed: {}/{}", completed, pzl_count))
                .set_screen_pos(Point::new(1, 2))
        ),
        Point::new(-1, -1)
    );

    let mut percent = completed as f64 / pzl_count as f64 * 100.0;
    if percent.is_nan() {
        percent = 0.0;
    }
    pzl_scene.add_element(
        Box::new(
            basic_button()
                .set_txt(format!("Completion: {:.1}%", percent))
                .set_screen_pos(Point::new(1, 1))
        ),
        Point::new(-2, -1)
    );

    // Last section.
    let mut last_sect = -1;
    let mut section_size = 0;
    let mut pos = Point::new(1, 2);
    let mut screen_pos = pos + Point::new(1, 3);

    for (n, pzl) in pzls.pzls.iter().enumerate() {
        let this_sect = SECTION_SIZES.get((last_sect + 1) as usize).copied().unwrap_or(999);

        // New section.
        if sectioning && this_sect == section_size {
            last_sect += 1;
            section_size = 0;
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
        section_size += 1;

        let txt_clr = if completion.contains(&pzl.id) {
            style::Color::Rgb { r: 50, g: 255, b: 0 }
        } else {
            style::Color::White
        };

        let button = basic_button()
            .set_txt(format!("{}", pzl.name))
            .set_clr(txt_clr)
            .set_events(vec![
                ui::Event::Broadcast(String::from("clr")),
                ui::Event::Exit(n as u32 + 2000),
            ])
            .set_screen_pos(screen_pos);
        let mut pzl_win = windowed::Window::new(PREVIEW_POS);
        let top_left = Point::new(
            -(PREVIEW_SIZE.0 as i32 / 2 - pzl.data.wid as i32 / 2),
            PREVIEW_SIZE.1 as i32 / 2 + pzl.data.hgt as i32 / 2,
        );
        pzl.data.display_into(&mut pzl_win, top_left, PREVIEW_SIZE.0 as u32, PREVIEW_SIZE.1 as u32);
        pzl_win.outline_with(outline_ch());
        pzl_scene.add_element(
            Box::new(
                linked_button::LinkedButton::new(
                    button,
                    pzl_win,
                )
            ),
            pos,
        );
        pos.y += 1;
        screen_pos.y += 1;
    }

    if editing {
        let new = basic_button()
            .set_txt(String::from("    New Puzzle    "))
            .set_screen_pos(screen_pos)
            .set_events(vec![
                ui::Event::Broadcast(String::from("clr")),
                ui::Event::Exit(NEW_PUZZLE),
            ]);
        pzl_scene.add_element(
            Box::new(new),
            pos,
        );
        pos.y += 1;
        screen_pos.y += 1;
    }
    let menu = basic_button()
        .set_txt(String::from("Back"))
        .set_screen_pos(screen_pos + Point::new(0, 1))
        .set_events(
            vec![
                ui::Event::Broadcast(String::from("clr")),
                if editing { ui::Event::Exit(CANCEL) } else { ui::Event::ChangeScene(0) }
            ]
        );
    pzl_scene.add_element(
        Box::new(menu),
        pos,
    );
    add_outline(&mut pzl_scene, PZL_SIZE.0);

    pzl_scene.move_cursor(Point::new(1, 2));
    pzl_scene
}

/// End screen when a puzzle is completed.
pub fn end_screen(editor: bool) -> ui::Scene {
    let hgt = if editor { 
        3 
    } else {
        END_SIZE.1
    };
    let mut scene = ui::Scene::new(END_POS, END_SIZE.0, hgt);

    let txt = if editor {
        vec![
            ("Back to Editor", ui::Event::Exit(EDITOR))
        ]
    } else {
        vec![
            ("Next Puzzle", ui::Event::Exit(NEXT)),
            ("Puzzle Select", ui::Event::Exit(PUZZLE_SEL)),
            ("Main Menu", ui::Event::Exit(MAIN_MENU)),
            ("Save and Quit", ui::Event::Exit(SAVE_AND_QUIT)),
        ]
    };

    for (n, (msg, ev)) in txt.into_iter().enumerate() {
        scene.add_element(
            Box::new(
                basic_button()
                    .set_txt(String::from(msg))
                    .set_events(vec![
                        ui::Event::Broadcast(String::from("clr")),
                        ev
                    ])
                    .set_screen_pos(Point::new(1, n as i32 + 1)),
            ),
            Point::new(1, n as i32 + 1),
        );
    }

    add_outline(&mut scene, END_SIZE.0);
    add_title("complete.txt", &mut scene, 1);

    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Screen for when the game is paused.
pub fn pause_screen(editor: bool) -> ui::Scene {
    let hgt = if editor { 
        5
    } else {
        PAUSE_SIZE.1
    };
    let mut scene = ui::Scene::new(PAUSE_POS, PAUSE_SIZE.0, hgt);

    let txt = if editor {
        vec![
            ("Resume", ui::Event::Exit(PLAY)),
            ("Reset", ui::Event::Exit(RESET)),
            ("Back to Editor", ui::Event::Exit(EDITOR))
        ]
    } else {
        vec![
            ("Resume", ui::Event::Exit(PLAY)),
            ("Reset", ui::Event::Exit(RESET)),
            ("Puzzle Select", ui::Event::Exit(PUZZLE_SEL)),
            ("Main Menu", ui::Event::Exit(MAIN_MENU)),
            ("Save and Quit", ui::Event::Exit(SAVE_AND_QUIT)),
        ]
    };

    for (n, (msg, ev)) in txt.into_iter().enumerate() {
        scene.add_element(
            Box::new(
                basic_button()
                    .set_txt(String::from(msg))
                    .set_events(vec![
                        ui::Event::Broadcast(String::from("clr")),
                        ev
                    ])
                    .set_screen_pos(Point::new(1, n as i32 + 1)),
            ),
            Point::new(1, n as i32 + 1),
        );
    }

    add_outline(&mut scene, PAUSE_SIZE.0);
    add_title("paused.txt", &mut scene, 1);

    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Scene for selecting a puzzle pack or creating a new one.
pub fn pack_sel(packs: &[loader::puzzles::PuzzlePack]) -> ui::Scene {
    let mut scene = ui::Scene::new(PACK_POS, PACK_SIZE.0, PACK_SIZE.1);
    add_list(&mut scene, Point::new(1, 1), packs.iter().map(|p| p.name.clone()), false);

    let pos = Point::new(1, packs.len() as i32 + 1);
    let new = basic_button()
        .set_txt(String::from("    New Pack    "))
        .set_screen_pos(pos)
        .set_events(vec![
            ui::Event::Broadcast(String::from("clr")),
            ui::Event::Exit(NEW_PACK),
        ]);
    let menu = basic_button()
        .set_txt(String::from("Back"))
        .set_screen_pos(pos + Point::new(0, 2))
        .set_events(
            vec![
                ui::Event::Broadcast(String::from("clr")),
                ui::Event::Exit(CANCEL),
            ]
        );
    scene.add_element(
        Box::new(new),
        pos,
    );
    scene.add_element(
        Box::new(menu),
        pos + Point::new(0, 1),
    );

    add_outline(&mut scene, PACK_SIZE.0);
    add_title("my_puzzles.txt", &mut scene, 1);
    scene.move_cursor(Point::new(1, 1));

    scene.with_scrolling(true)
}

/// Scene for getting a new name for something.
pub fn name_entry() -> ui::Scene {
    let mut scene = ui::Scene::new(ENTRY_POS, ENTRY_SIZE.0, ENTRY_SIZE.1);

    scene.add_element(
        Box::new(
            basic_entry()
                .set_screen_pos(Point::new(1, 1))
                .set_len(16)
        ),
        Point::new(1, 1),
    );

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Confirm"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CONFIRM),
                ])
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Cancel"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CANCEL)
                ])
                .set_screen_pos(Point::new(10, 2)),
        ),
        Point::new(2, 2),
    );
    scene.move_cursor(Point::new(1, 1));

    add_outline(&mut scene, ENTRY_SIZE.0);
    add_title("enter_name.txt", &mut scene, 1);

    scene
}

/// Options after selecting something; rename, modify, delete, cancel.
pub fn sel_opts() -> ui::Scene {
    let mut scene = mk_scene(OPTS_POS, OPTS_SIZE);

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Rename"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(RENAME),
                ])
                .set_screen_pos(Point::new(1, 1)),
        ),
        Point::new(1, 1),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Modify"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(MODIFY),
                ])
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Delete"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(DEL),
                ])
                .set_screen_pos(Point::new(1, 3)),
        ),
        Point::new(1, 3),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Cancel"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CANCEL),
                ])
                .set_screen_pos(Point::new(1, 4)),
        ),
        Point::new(1, 4),
    );

    add_outline(&mut scene, OPTS_SIZE.0);
    add_title("choose.txt", &mut scene, 1);
    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Scene to make sure the user really wants to this.
pub fn confirm_scene(msg: String) -> ui::Scene {
    let hgt = msg.len() / (WARN_WID - 2) + 5;
    let mut scene = mk_scene(WARN_POS, (WARN_WID, hgt));

    let mut cur_str = String::new();
    let mut y = 1;

    for (n, ch) in msg.chars().enumerate() {
        cur_str.push(ch);
        if (n % (WARN_WID - 2) == 0 && n != 0) || n == msg.len() - 1 {
            scene.add_element(
                Box::new(
                    basic_button()
                        .set_txt(cur_str)
                        .set_clr(style::Color::DarkRed)
                        .set_screen_pos(Point::new(1, y))
                        .set_static_len(false)
                ),
                Point::new(1, y-69),
            );
            cur_str = String::new();
            y += 1;
        }
    }

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Confirm"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CONFIRM),
                ])
                .set_screen_pos(Point::new(1, y+1)),
        ),
        Point::new(1, 1),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Cancel"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CANCEL)
                ])
                .set_screen_pos(Point::new(10, y+1)),
        ),
        Point::new(2, 1),
    );

    add_outline(&mut scene, WARN_WID);
    add_title("warning.txt", &mut scene, 1);
    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Warns the user about something they did wrong and doesn't give them a choice.
pub fn warn_scene(msg: String) -> ui::Scene {
    let hgt = msg.len() / (WARN_WID - 2) + 5;
    let mut scene = mk_scene(WARN_POS, (WARN_WID, hgt));

    let mut cur_str = String::new();
    let mut y = 1;

    for (n, ch) in msg.chars().enumerate() {
        cur_str.push(ch);
        if (n % (WARN_WID - 2) == 0 && n != 0) || n == msg.len() - 1 {
            scene.add_element(
                Box::new(
                    basic_button()
                        .set_txt(cur_str)
                        .set_clr(style::Color::DarkRed)
                        .set_screen_pos(Point::new(1, y))
                        .set_static_len(false)
                ),
                Point::new(1, y-69),
            );
            cur_str = String::new();
            y += 1;
        }
    }

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Cool story"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CONFIRM),
                ])
                .set_screen_pos(Point::new(1, y+1)),
        ),
        Point::new(1, 1),
    );

    add_outline(&mut scene, WARN_WID);
    add_title("warning.txt", &mut scene, 1);
    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Menu reached by pressing esc in the editor.
pub fn editor_menu() -> ui::Scene {
    let mut scene = mk_scene(EDIT_MENU_POS, EDIT_MENU_SIZE);

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Resume"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CANCEL),
                ])
                .set_screen_pos(Point::new(1, 1)),
        ),
        Point::new(1, 1),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Test"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(PLAY),
                ])
                .set_screen_pos(Point::new(1, 2)),
        ),
        Point::new(1, 2),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Save"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(SAVE),
                ])
                .set_screen_pos(Point::new(1, 3)),
        ),
        Point::new(1, 3),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Quit"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::ChangeScene(1),
                ])
                .set_screen_pos(Point::new(1, 4)),
        ),
        Point::new(1, 4),
    );

    add_outline(&mut scene, EDIT_MENU_SIZE.0);
    add_title("paused.txt", &mut scene, 1);
    scene.move_cursor(Point::new(1, 1));

    scene
}

/// Create a scene for getting a width and height of a puzzle.
pub fn size_scene(min: i32, max: i32, init: i32) -> ui::Scene {
    let mut scene = mk_scene(WID_HGT_POS, WID_HGT_SIZE);

    add_counter(&mut scene, 0, Point::new(1, 2), Point::new(1, 2), min, max, init);
    add_counter(&mut scene, 1, Point::new(4, 2), Point::new(2, 2), min, max, init);

    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Confirm"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CONFIRM),
                ])
                .set_screen_pos(Point::new(1, 5)),
        ),
        Point::new(1, 3),
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("Cancel"))
                .set_events(vec![
                    ui::Event::Broadcast(String::from("clr")),
                    ui::Event::Exit(CANCEL)
                ])
                .set_screen_pos(Point::new(10, 5)),
        ),
        Point::new(2, 3)
    );

    scene.move_cursor(Point::new(1, 2));
    add_outline(&mut scene, WID_HGT_SIZE.0);
    add_title("set_size.txt", &mut scene, 1);

    scene
}

