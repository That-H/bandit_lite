#![allow(unused_must_use)]

use bandit_lite::{display::display_all, *};
use crossterm::{execute, terminal, event, style, cursor};
use style::Stylize;
use std::{time, io};
use display::scenes;

const GAME_POS: Point = Point::new(
    (TERMINAL_WID / 2 - GAME_WID / 2) as i32,
    (TERMINAL_HGT / 2 - GAME_HGT / 2 - 1) as i32,
);

fn main() {
    let mut handle = io::stdout();

    // Raw mode required for windowed to work correctly.
    terminal::enable_raw_mode();
    execute!(
        handle,
        terminal::Clear(terminal::ClearType::All),
        terminal::SetSize(TERMINAL_WID, TERMINAL_HGT),
        cursor::Hide,
    );

    // Objects we have.
    let mut objs = loader::load_objs();
    let default_tile = Tile::new('.'.white(), false, false);
    objs.add_tile(default_tile.clone());
    objs.add_tile(Tile::new('#'.white(), true, true));
    objs.add_entity(Ent::player());
    
    // Add goals and lasers to the tile set.
    for i in 0..8 {
        let clr = beam::Clr::from(i);
        objs.add_entity(Ent::goal(clr));
        for p in 0..8 {
            objs.add_entity(Ent::laser(beam::PORT_DIRS[p], clr));
        }
    }

    // Load puzzles in.
    let pzls = loader::load_standard_pzls(&default_tile, &objs);
    // Index of current puzzle in the puzzle list.
    let mut pzl_idx = 0;
    // Load completion state.
    let mut completion = loader::saver::load_pzl_save();

    // Initial scene when the full loop begins.
    let mut init_scene = 0;

    'full: loop {
        let mut main_cont = windowed::Container::new();
        let mut ui_cont = windowed::ui::UiContainer::new();
        ui_cont.add_scene(scenes::main_menu());
        ui_cont.add_scene(scenes::puzzle_select(&pzls, &completion));
        ui_cont.add_scene(scenes::end_screen());
        ui_cont.add_scene(scenes::pause_screen());
        ui_cont.change_scene(init_scene);

        main_cont.add_win(windowed::Window::new(GAME_POS));
        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
        match ui_cont.run() {
            scenes::PLAY => (),
            scenes::SAVE_AND_QUIT => break,
            p if p >= 2000 => pzl_idx = p as usize - 2000,
            a => panic!("Unrecognised exit code {a}"),
        }
        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
        // Map used for the game.
        let mut map = loader::puzzles::start_puzzle(&pzls[pzl_idx]);

        'game: loop {
            clear_events();

            // Display the game window.
            display::display_all(&map, &mut main_cont, unsafe { PLAYER });

            while let event::Event::Key(ke) = event::read().expect("what") {
                if ke.is_press() {
                    let mv = match ke.code {
                        event::KeyCode::Left
                        | event::KeyCode::Char('a')
                        | event::KeyCode::Char('h') => Point::new(-1, 0),
                        event::KeyCode::Right
                        | event::KeyCode::Char('d')
                        | event::KeyCode::Char('l') => Point::new(1, 0),
                        event::KeyCode::Down
                        | event::KeyCode::Char('s')
                        | event::KeyCode::Char('j') => Point::new(0, -1),
                        event::KeyCode::Up
                        | event::KeyCode::Char('w')
                        | event::KeyCode::Char('k') => Point::new(0, 1),
                        // Pause the game.
                        event::KeyCode::Esc => {
                            ui_cont.change_scene(3);
                            let mut restart = false;
                            let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));

                            match ui_cont.run() {
                                scenes::PLAY => {
                                    display_all(&map, &mut main_cont, unsafe { PLAYER });
                                },
                                scenes::PUZZLE_SEL => {
                                    init_scene = 1;
                                    continue 'full;
                                }
                                scenes::SAVE_AND_QUIT => {
                                    break 'full;
                                }
                                p if p >= 2000 => { 
                                    pzl_idx = p as usize - 2000;
                                    restart = true;
                                }
                                a => panic!("Unrecognised exit code {a}"),
                            }
                            if restart {
                                if let Some(pzl) = pzls.get(pzl_idx) {
                                    map = loader::puzzles::start_puzzle(pzl);
                                    let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                                    continue 'game;
                                } else {
                                    init_scene = 0;
                                    continue 'full;
                                }
                            } else { 
                                continue;
                            }
                        },
                        _ => Point::ORIGIN,
                    };

                    unsafe { DIR = mv }
                    break;
                }
            }

            map.update_vfx();

            while map.update() {}
            unsafe {
                // Only true at this point when the puzzle is won, so record this.
                if SHOULD_WIN {
                    completion.insert(pzls[pzl_idx].id);
                    ui_cont.change_scene(2);
                    let mut restart = false;
                    let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));

                    match ui_cont.run() {
                        scenes::SAVE_AND_QUIT => break 'full,
                        scenes::NEXT => { 
                            pzl_idx += 1;
                            restart = true;
                        },
                        scenes::PUZZLE_SEL => {
                            init_scene = 1;
                            continue 'full;
                        }
                        p if p >= 2000 => { 
                            pzl_idx = p as usize - 2000;
                            restart = true;
                        }
                        a => panic!("Unrecognised exit code {a}"),
                    }

                    if restart {
                        if let Some(pzl) = pzls.get(pzl_idx) {
                            map = loader::puzzles::start_puzzle(pzl);
                            let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                            continue 'game;
                        } else {
                            init_scene = 0;
                            continue 'full;
                        }
                    }
                }
                SHOULD_WIN = true;
            }
            let mut to_reset = Vec::new();

            for (&p, _e) in map.get_entities() {
                to_reset.push(p);
            }

            for p in to_reset {
                map.get_ent_mut(p).unwrap().updated = false;
            }

            beam::INPTS.write().unwrap().clear();
        }
    }
    
    // Put the terminal in a "normal" state in case the player actually wants to use it afterwards.
    terminal::disable_raw_mode();
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
    );

    // Save the completion to the file.
    loader::saver::write_pzl_save(completion);
}

/// Clears all events currently in the queue.
fn clear_events() {
    while let Ok(b) = event::poll(time::Duration::from_secs(0))
        && b
    {
        event::read();
    }
}

