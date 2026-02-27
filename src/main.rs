#![allow(unused_must_use)]

use bandit_lite::*;
use crossterm::{execute, terminal, event, style, cursor};
use style::Stylize;
use std::{time, io};

const GAME_POS: Point = Point::new(
    (TERMINAL_WID / 2 - GAME_WID / 2) as i32,
    (TERMINAL_HGT / 2 - GAME_HGT / 2 - 1) as i32,
);

fn main() {
    let mut main_cont = windowed::Container::new();
    main_cont.add_win(windowed::Window::new(GAME_POS));

    // Raw mode required for windowed to work correctly.
    terminal::enable_raw_mode();
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        terminal::SetSize(TERMINAL_WID, TERMINAL_HGT),
        cursor::Hide,
    );

    // Objects we have.
    let objs = entity::loader::load_objs();

    // Map used for the game.
    let mut map = bn::Map::new(32, 32);
    map.insert_entity(Ent::player(), Point::ORIGIN);
    map.insert_entity(Ent::laser(Point::new(1, 0), beam::Clr::Blue), Point::new(1, 0));
    map.insert_entity(Ent::laser(Point::new(-1, 0), beam::Clr::Red), Point::new(2, 0));
    map.insert_entity(Ent::laser(Point::new(0, 1), beam::Clr::Red), Point::new(3, 0));
    map.insert_entity(Ent::laser(Point::new(0, -1), beam::Clr::Green), Point::new(-1, 0));
    map.insert_entity(objs[1].clone(), Point::new(-2, -2));
    map.insert_entity(objs[0].clone(), Point::new(-3, -3));

    for y in -5..=5 {
        for x in -5..=5 {
            let p = Point::new(x, y);

            let t = if y.abs() == 5 || x.abs() == 5 {
                Tile::new('#'.stylize(), true, true)
            } else {
                Tile::new(' '.stylize(), false, false,)
            };
            
            map.insert_tile(t, p);
        }
    }

    'main: loop {
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
                    event::KeyCode::Esc => break 'main,
                    _ => Point::ORIGIN,
                };

                unsafe { DIR = mv }
                break;
            }
        }

        map.update_vfx();

        while map.update() {}
        let mut to_reset = Vec::new();

        for (&p, _e) in map.get_entities() {
            to_reset.push(p);
        }

        for p in to_reset {
            map.get_ent_mut(p).unwrap().updated = false;
        }

        beam::INPTS.write().unwrap().clear();
    }
    
    // Put the terminal in a "normal" state in case the player actually wants to use it afterwards.
    terminal::disable_raw_mode();
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show,
    );
}

/// Clears all events currently in the queue.
fn clear_events() {
    while let Ok(b) = event::poll(time::Duration::from_secs(0))
        && b
    {
        event::read();
    }
}

