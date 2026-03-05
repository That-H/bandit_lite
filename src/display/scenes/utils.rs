//! Some internal utilities for making scenes.

use super::*;
use counter::Counter;

/// Make a scene out of the provided position and size.
pub fn mk_scene(pos: Point, size: (usize, usize)) -> ui::Scene {
    ui::Scene::new(pos, size.0, size.1)
}

/// Add a list of buttons that exit with increasing codes (starting at 2000) to the scene. Adds
/// sections for standard puzzle selection if sectioning is true.
pub fn add_list<I: IntoIterator<Item=String>>(
    scene: &mut ui::Scene,
    start: Point,
    names: I,
    menu_button: bool,
) {
    // Last section.
    let Point { x, y } = start;
    let mut pos = Point::new(x, y);

    for (n, name) in names.into_iter().enumerate() {
        let button = basic_button()
            .set_txt(format!("{}", name))
            .set_events(vec![
                ui::Event::Broadcast(String::from("clr")),
                ui::Event::Exit(n as u32 + 2000),
            ])
            .set_screen_pos(pos);
        scene.add_element(
            Box::new(
                button,
            ),
            pos,
        );
        pos = pos + Point::new(0, 1);
    }

    if menu_button {
        // Add a main menu button.
        scene.add_element(
            Box::new(
                basic_button()
                    .set_txt(String::from("Main Menu"))
                    .set_events(vec![
                        ui::Event::Broadcast(String::from("clr")),
                        ui::Event::ChangeScene(0),
                    ])
                    .set_screen_pos(pos + Point::new(0, 2)),
            ),
            pos 
        );
    }
}

/// Make the given file into a title and put it in the scene.
pub fn add_title<P: AsRef<std::path::Path>>(fname: P, scene: &mut ui::Scene, y: i32) {
    let mut f = fs::File::open(get_assets_path().join(fname)).unwrap();
    let mut text = String::new();
    let _ = f.read_to_string(&mut text);
    let wid = text.lines().next().unwrap().len();

    let title = ui::widgets::Title::new(Point::new(centre(wid), y), text, Some(DELAY));
    scene.add_element(Box::new(title), Point::new(500, 500+y));
}

/// Get the character used to outline scenes.
pub fn outline_ch() -> StyleCh {
    '#'.grey()
}

/// Standard button.
pub fn basic_button() -> ui::widgets::Button {
    ui::widgets::Button::empty_new()
        .set_selector(String::from(SELECTOR))
        .set_hover_clr(HOVER_CLR)
        .set_selector_clr(SELECTOR_CLR)
        .set_static_len(true)
}

/// Standard entry box (probably won't need this).
pub fn basic_entry() -> ui::widgets::TextEntry {
    ui::widgets::TextEntry::new()
        .set_hover_clr(HOVER_CLR)
        .set_highlight_clr(style::Color::Cyan)
        .set_active_clr(HOVER_CLR)
}

/// Adds an outline to the scene.
pub fn add_outline(scene: &mut ui::Scene, wid: usize) {
    scene.add_element(Box::new(ui::widgets::Outline::new(outline_ch(), wid)), Point::new(999, 999));
}

/// Adds a counter with incr/decr buttons.
pub fn add_counter(scene: &mut ui::Scene, id: usize, centre: Point, ui_centre: Point, min: i32, max: i32, init: i32) {
    let mut counter = Counter::new(id, centre).with_min(min).with_max(max);
    counter.set_value(init);

    scene.add_element(
        Box::new(counter),
        Point::new(69, 69) + centre,
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("^"))
                .set_event(ui::Event::Broadcast(format!("{id}:1")))
                .set_screen_pos(centre + Point::new(0, -1)),
        ),
        ui_centre - Point::new(0, 1)
    );
    scene.add_element(
        Box::new(
            basic_button()
                .set_txt(String::from("v"))
                .set_event(ui::Event::Broadcast(format!("{id}:-1")))
                .set_screen_pos(centre + Point::new(0, 1)),
        ),
        ui_centre
    );
}
