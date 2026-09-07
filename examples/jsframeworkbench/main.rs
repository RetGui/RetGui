use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

use retgui::elements::{Button, Container, Element, State as StateHandle, Text, Window};
use retgui::events::ClickEvent;
use retgui::palette::css::WHITE;
use retgui::style::{AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Unit};
use retgui::{App, Color, States, rgb};

const ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

const COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black", "orange",
];

const NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger", "pizza", "mouse",
    "keyboard",
];

#[derive(Clone)]
pub struct Data {
    id: usize,
    label: String,
}

impl Data {
    pub fn new(id: usize, label: String) -> Self {
        Self { id, label }
    }
}

pub struct State {
    store: Store,
    rows: Vec<Row>,
    selected_row: Option<usize>,
    element: Container,
}

#[derive(Clone)]
struct Row {
    element: Container,
    label: Text,
}

impl State {
    fn new(element: Container) -> Self {
        Self {
            store: Store::new(),
            rows: Vec::new(),
            selected_row: None,
            element,
        }
    }

    fn run(&mut self, app: &mut App, lots: bool) {
        self.clear(app);
        if lots {
            self.store.run_lots();
        } else {
            self.store.run();
        }
        self.attach_rows(app, 0);
    }

    fn append(&mut self, app: &mut App) {
        let old_len = self.rows.len();
        self.store.add();
        self.selected_row = None;
        self.attach_rows(app, old_len);
    }

    fn attach_rows(&mut self, app: &mut App, start: usize) {
        for data in &self.store.data[start..] {
            let row = Self::create_row(app, data);
            self.element.push(app, row.element);
            self.rows.push(row);
        }
    }

    fn swap_rows(&mut self, app: &mut App) {
        if self.store.data.len() >= 999 {
            self.store.swap_rows();
            self.rows.swap(1, 998);
            self.element
                .swap_child(
                    app,
                    self.rows[1].element.as_dyn_element(),
                    self.rows[998].element.as_dyn_element(),
                )
                .expect("failed to swap rows");
        }
    }

    fn clear(&mut self, app: &mut App) {
        self.store.clear();
        self.rows.clear();
        self.selected_row = None;
        self.element.delete_all_children(app);
    }

    fn create_row(app: &mut App, data: &Data) -> Row {
        let label = Text::new(app, &data.label);
        let id = Text::new(app, &data.id.to_string());
        id.set_width(app, Unit::Px(60.0));
        id.set_margin(app, Unit::Px(0.0), Unit::Px(12.0), Unit::Px(0.0), Unit::Px(0.0));
        let element = Container::new(app);
        element.set_display(app, Display::Flex);
        element.set_flex_direction(app, FlexDirection::Row);
        element.set_width(app, Unit::Auto);
        element.set_padding(app, Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0));
        element.set_border_color_all(app, Color::from_rgb8(230, 230, 230));
        element.push(app, id);
        element.push(app, label);
        Row { element, label }
    }

    fn update(&mut self, app: &mut App) {
        self.store.update();
        self.selected_row = None;
        for (index, data) in self.store.data.iter().enumerate().step_by(10) {
            self.rows[index].label.set_text(app, &data.label);
        }
    }
}

pub struct Store {
    data: Vec<Data>,
    next_id: usize,
    rng: ThreadRng,
    selected: Option<usize>,
}

impl Store {
    pub fn swap_rows(&mut self) {
        if self.data.len() >= 999 {
            self.data.swap(1, 998);
        }
    }

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next_id: 1,
            rng: rng(),
            selected: None,
        }
    }

    pub fn build_data(&mut self, count: usize) {
        self.data.reserve(count);
        for _ in 0..count {
            self.data.push(Data::new(
                self.next_id,
                format!(
                    "{} {} {}",
                    ADJECTIVES.choose(&mut self.rng).unwrap(),
                    COLOURS.choose(&mut self.rng).unwrap(),
                    NOUNS.choose(&mut self.rng).unwrap()
                ),
            ));
            self.next_id += 1;
        }
    }

    pub fn run(&mut self) {
        self.build_data(1000);
        self.selected = None;
    }

    pub fn run_lots(&mut self) {
        self.build_data(10000);
        self.selected = None;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.selected = None;
    }

    pub fn select(&mut self, id: Option<usize>) {
        self.selected = id;
    }

    pub fn add(&mut self) {
        self.build_data(1000);
        self.selected = None;
    }

    pub fn delete(&mut self, id: usize) {
        self.data.retain(|f| f.id != id)
    }

    pub fn update(&mut self) {
        self.update_data();
        self.selected = None;
    }

    pub fn update_data(&mut self) {
        for data in self.data.iter_mut().step_by(10) {
            data.label += " !!!";
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    //util::setup_logging();

    let mut app = App::new();
    let mut states = States::new();
    let data_list = build_data_list(&mut app);
    let state = states.insert(State::new(data_list));

    let body = build_body(&mut app, &states, state);
    let window = Window::new(&mut app, "JsFrameworkBench");
    window.set_width(&mut app, Unit::Percentage(100.0));
    window.set_height(&mut app, Unit::Percentage(100.0));
    window.push(&mut app, body);

    use retgui::RetGuiOptions;

    retgui::retgui_main(app, states, RetGuiOptions::basic("jsframeworkbench"));
}

fn build_body(app: &mut App, states: &States, state: StateHandle<State>) -> Container {
    let buttons = build_buttons(app, state);

    let body = Container::new(app);
    body.set_overflow(app, Overflow::Visible, Overflow::Scroll);
    body.set_width(app, Unit::Percentage(100.0));
    body.set_height(app, Unit::Percentage(100.0));
    body.set_flex_direction(app, FlexDirection::Column);
    body.set_align_items(app, AlignItems::Start);
    body.set_padding_all(app, Unit::Px(15.0));

    let text = Text::new(app, r#"RetGui-"keyed""#);
    text.set_font_size(app, 32.0);
    text.set_color(app, Color::BLACK);

    let text_container = Container::new(app);
    text_container.set_display(app, Display::Flex);
    text_container.set_flex_direction(app, FlexDirection::Row);
    text_container.set_width(app, Unit::Percentage(50.0));
    text_container.set_justify_content(app, JustifyContent::Center);
    text_container.set_align_items(app, AlignItems::Center);
    text_container.push(app, text);

    let header = Container::new(app);
    header.set_background_color(app, rgb(238, 238, 238));
    header.set_display(app, Display::Flex);
    header.set_flex_direction(app, FlexDirection::Row);
    header.set_border_radius_all(app, (6.0, 6.0));
    header.set_padding(app, Unit::Px(10.0), Unit::Px(60.0), Unit::Px(10.0), Unit::Px(60.0));
    header.push(app, text_container);
    header.set_width(app, Unit::Percentage(100.0));
    header.push(app, buttons);

    let data_list = state.borrow(states).element;
    body.push(app, header);
    body.push(app, data_list);
    body
}

fn build_data_list(app: &mut App) -> Container {
    let container = Container::new(app);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_width(app, Unit::Percentage(100.0));
    container
}

fn build_buttons(app: &mut App, state: StateHandle<State>) -> Container {
    let buttons = Container::new(app);
    buttons.set_flex_direction(app, FlexDirection::Column);
    buttons.set_justify_content(app, JustifyContent::FlexEnd);
    buttons.set_align_items(app, AlignItems::Start);
    buttons.set_gap(app, Unit::Px(12.0), Unit::Px(12.0));
    buttons.set_wrap(app, FlexWrap::Wrap);
    buttons.set_max_height(app, Unit::Px(150.0));

    let btn_create_1k = build_button(app, "Create 1,000 rows", move |_event, app, states| {
        state.borrow_mut(states).run(app, false);
    });

    let btn_create_10k = build_button(app, "Create 10,000 rows", move |_event, app, states| {
        state.borrow_mut(states).run(app, true);
    });

    let btn_append_1k = build_button(app, "Append 1,000 rows", move |_event, app, states| {
        state.borrow_mut(states).append(app);
    });
    let btn_update_10th_row = build_button(app, "Update every 10th row", move |_event, app, states| {
        state.borrow_mut(states).update(app);
    });
    let btn_clear = build_button(app, "Clear", move |_event, app, states| {
        state.borrow_mut(states).clear(app);
    });
    let btn_swap = build_button(app, "Swap Rows", move |_event, app, states| {
        state.borrow_mut(states).swap_rows(app);
    });

    buttons.push(app, btn_create_1k);
    buttons.push(app, btn_create_10k);
    buttons.push(app, btn_append_1k);
    buttons.push(app, btn_update_10th_row);
    buttons.push(app, btn_clear);
    buttons.push(app, btn_swap);
    buttons
}

fn build_button<F>(app: &mut App, label: &str, callback: F) -> Button
where
    F: Fn(&mut ClickEvent, &mut App, &mut States) + 'static,
{
    let label = Text::new(app, label);
    label.set_selectable(app, false);
    label.set_color(app, Color::WHITE);
    let button = Button::new(app);
    button.set_background_color(app, Color::from_rgb8(211, 211, 211));
    button.set_border_color_all(app, Color::from_rgb8(111, 111, 111));
    button.set_flex_direction(app, FlexDirection::Row);
    button.set_justify_content(app, JustifyContent::Center);
    button.set_align_items(app, AlignItems::Center);
    button.set_gap(app, Unit::Px(12.0), Unit::Px(12.0));
    button.set_width(app, Unit::Px(250.0));
    button.set_height(app, Unit::Px(35.0));
    button.set_background_color(app, Color::from_rgb8(51, 122, 183));
    button.set_color(app, WHITE);
    button.set_border_radius_all(app, (4.0, 4.0));
    button.push(app, label);
    button.add_click_listener(app, callback);
    button
}
