//! Uses ui scenes to perform common tasks.

use super::*;
use loader::puzzles;
use level_editor::EditEvent;
use crossterm::{execute, event, terminal};

/// Get the user to choose a puzzle pack. Returns None if they don't want to. Also allows them to
/// create new packs, rename existing packs, or delete them.
pub fn choose_pack(packs: &mut Vec<puzzles::PuzzlePack>, std_pzls: &puzzles::PuzzlePack) -> Option<usize> {
    let mut handle = io::stdout();

    // See which pack we're using.
    let mut pack_sel = ui::UiContainer::new();
    pack_sel.add_scene(scenes::pack_sel(&packs));

    // See what the user wants to do.
    let mut decision = ui::UiContainer::new();
    decision.add_scene(scenes::sel_opts());
    decision.add_scene(
        scenes::confirm_scene(
            String::from(
                "Are you sure? This cannot be undone!"
            )
        )
    );

    let pack_idx = loop {
        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
        match pack_sel.run() {
            scenes::NEW_PACK => {
                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                if let Some(name) = get_name() {
                    let pack = if loader::saver::is_secret(&name) {
                        let mut pck = std_pzls.clone();
                        pck.name = name;

                        pck
                    } else {
                        let pack = loader::puzzles::PuzzlePack::new(name);
                        let _ = loader::saver::write_pzls(&pack);
                        pack
                    };
                    packs.push(pack); 
                    pack_sel.scenes[0] = scenes::pack_sel(packs);
                }
                continue
            }
            scenes::CANCEL => {
                return None;
            }
            // User has made a selection.
            idx if idx >= 2000 => {
                let idx = idx as usize - 2000;
                decision.change_scene(0);
                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                match decision.run() {
                    RENAME => {
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        if let Some(name) = get_name() {
                            let old_name = packs[idx].name.clone();
                            packs[idx].name = name;
                            pack_sel.scenes[0] = scenes::pack_sel(packs);
                            let _ = loader::saver::delete_pack(old_name);
                            let _ = loader::saver::write_pzls(&packs[idx]);
                        }
                    }
                    MODIFY => break idx,
                    DEL => {
                        decision.change_scene(1);
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        match decision.run() {
                            CONFIRM => {
                                let _ = loader::saver::delete_pack(packs.remove(idx).name);
                                pack_sel.scenes[0] = scenes::pack_sel(packs);
                            },
                            CANCEL => (),
                            _ => unreachable!(),
                        }
                    }
                    CANCEL => (),
                    _ => unreachable!(),
                }
            }
            _ => (),
        }
    };

    let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
    Some(pack_idx)
}

/// Edit a puzzle using the LevelEditor.
pub fn edit_puzzle(
    cont: &mut windowed::Container<StyleCh>,
    objs: &Vec<Vec<Ent>>,
    pzl: &mut puzzles::Puzzle,
) -> EditExit {
    let mut handle = io::stdout();
    let mut ui = ui::UiContainer::new();
    let mut scene = ui::Scene::new(level_editor::DEFAULT_POS - Point::new(18, 1), 4, 7);

    for (n, obj) in objs.iter().enumerate() {
        let obj = &obj[0];
        let clr = obj.ch.style().foreground_color.unwrap();
        let pos = Point::new(1, n as i32 + 1);
        let button = basic_button()
            .set_txt(String::from(*obj.ch.content()))
            .set_clr(clr)
            .set_hover_clr(clr)
            .set_screen_pos(pos)
            .set_event(ui::Event::Exit(n as u32));
        scene.add_element(
            Box::new(button),
            pos,
        );
    }

    scene.move_cursor(Point::new(1, 1));
    add_outline(&mut scene, 5);

    ui.add_scene(scene.with_scrolling(true));

    let mut menu = ui::UiContainer::new();
    menu.add_scene(scenes::editor_menu());
    menu.add_scene(scenes::confirm_scene(String::from("Are you sure?      Unsaved changes   will be lost!")));

    let mut editor = level_editor::LevelEditor::new(objs, &mut pzl.data);
    // This is the default player position of puzzles, so if it is this, there must not be a
    // player yet.
    editor.pl_pos = if pzl.pl_pos == Point::new(-69, -420) { None } else { Some(pzl.pl_pos) };
    
    editor.outline();

    loop {
        editor.draw(&mut cont.windows[0], true);
        cont.refresh();
        display::print_win(&cont);
        while let event::Event::Key(ke) = event::read().expect("what") {
            if ke.is_press() {
                match editor.handle_key(ke) {
                    EditEvent::PickObj => {
                        editor.draw(&mut cont.windows[0], false);
                        cont.refresh();
                        display::print_win(&cont);
                        let new_idx = ui.run() as usize;
                        editor.cur_idx = new_idx;
                        break;
                    }
                    EditEvent::Menu => {
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        menu.change_scene(0);
                        match menu.run() {
                            CANCEL => (),
                            SAVE => { 
                                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                                if let Some(p) = editor.pl_pos {
                                    pzl.update();
                                    pzl.pl_pos = p;
                                    return EditExit::Save;
                                } else {
                                    warn("No Player!");
                                }
                            },
                            CONFIRM => return EditExit::Quit,
                            PLAY => {
                                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                                if let Some(p) = editor.pl_pos {
                                    pzl.update();
                                    pzl.pl_pos = p;
                                    return EditExit::Test;
                                } else {
                                    warn("No Player!");
                                }
                            }
                            _ => (),
                        }
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                    },
                    EditEvent::Null => (),
                }
                break;
            }
        }
    }
}

/// A way in which the edit_puzzle function can exit.
#[derive(Clone, Debug)]
pub enum EditExit {
    Save,
    Quit,
    Test,
}

/// Choose a puzzle from the pack, with options to create new puzzles, rename them, or delete them.
/// Will return None if the user changes their mind.
pub fn choose_puzzle(
    pack: &mut puzzles::PuzzlePack,
    completion: &HashSet<u128>,
    sectioning: bool,
    editing: bool
) -> Option<usize> {
    let mut handle = io::stdout();

    // See which puzzle we're using.
    let mut puzzle_sel = ui::UiContainer::new();
    puzzle_sel.add_scene(scenes::puzzle_select(&pack, &completion, sectioning, editing));

    // See what the user wants to do with this.
    let mut decision = ui::UiContainer::new();
    decision.add_scene(scenes::sel_opts());
    decision.add_scene(
        scenes::confirm_scene(
            String::from(
                "Are you sure? This cannot be undone!"
            )
        )
    );

    let pzl_idx = loop {
        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
        match puzzle_sel.run() {
            scenes::NEW_PUZZLE if editing => {
                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                loop {
                    if let Some(name) = get_name() {
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        if let Some((wid, hgt)) = get_size() {
                            let mut pzl = puzzles::Puzzle::new(name);
                            pzl.data.wid = wid as usize;
                            pzl.data.hgt = hgt as usize;
                            pack.pzls.push(pzl); 
                            puzzle_sel.scenes[0] = scenes::puzzle_select(&pack, &completion, sectioning, editing);
                            break;
                        } else {
                            let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                            continue;
                        }
                    } else {
                        break;
                    }
                }
                continue
            }
            scenes::CANCEL => {
                return None;
            }
            // User has made a selection.
            idx if idx >= 2000 => {
                let idx = idx as usize - 2000;
                if !editing {
                    break idx;
                }
                decision.change_scene(0);
                let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                match decision.run() {
                    RENAME => {
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        if let Some(name) = get_name() {
                            pack.pzls[idx].name = name;
                            puzzle_sel.scenes[0] = scenes::puzzle_select(&pack, &completion, sectioning, editing);
                        }
                    }
                    MODIFY => break idx,
                    DEL => {
                        decision.change_scene(1);
                        let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
                        match decision.run() {
                            CONFIRM => {
                                pack.pzls.remove(idx);
                                puzzle_sel.scenes[0] = scenes::puzzle_select(&pack, &completion, sectioning, editing);
                            },
                            CANCEL => (),
                            _ => unreachable!(),
                        }
                        let _ = loader::saver::write_pzls(pack);
                    }
                    CANCEL => (),
                    _ => unreachable!(),
                }
            }
            _ => (),
        }
    };

    let _ = execute!(handle, terminal::Clear(terminal::ClearType::All));
    Some(pzl_idx)
}

/// Get a name for something. Returns None if the user decides they don't want to give us a name.
pub fn get_name() -> Option<String> {
    let mut cont = ui::UiContainer::new();
    cont.add_scene(scenes::name_entry());

    match cont.run() {
        scenes::CONFIRM => {
            let name = cont.cur_scene().get_element(Point::new(1, 1)).unwrap().get_text();
            Some(name)
        }
        _ => None,
    }
}

/// Get a width and height for something. Returns None if the user decides they don't want to give us a name.
pub fn get_size() -> Option<(i32, i32)> {
    let mut cont = ui::UiContainer::new();
    cont.add_scene(scenes::size_scene(6, 16, 7));

    match cont.run() {
        scenes::CONFIRM => {
            let wid = cont.cur_scene().get_element(Point::new(70, 71)).unwrap().get_text();
            let hgt = cont.cur_scene().get_element(Point::new(73, 71)).unwrap().get_text();
            Some((wid.parse().unwrap(), hgt.parse().unwrap()))
        }
        _ => None,
    }
}

/// Tell the user off about something.
pub fn warn(msg: &str) {
    let mut cont = ui::UiContainer::new();
    cont.add_scene(scenes::warn_scene(String::from(msg)));

    cont.run();
}

