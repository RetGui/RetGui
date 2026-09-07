use retgui::elements::{Container, Element, State, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, BoxShadow, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, States, pct, px, retgui_main, rgb, rgba};

use util::setup_logging;

fn create_button(
    app: &mut App,
    label: &str,
    base_color: Color,
    delta: i64,
    state: State<i64>,
    count_text: Text,
) -> Container {
    let border_color = rgb(0, 0, 0);
    let label = Text::new(app, label);
    label.set_font_size(app, 24.0);
    label.set_color(app, Color::WHITE);
    label.set_selectable(app, false);
    let container = Container::new(app);
    container.set_box_shadows(
        app,
        vec![
            BoxShadow::new(false, 0.0, 5.0, 5.0, 0.0, rgba(0, 0, 0, 200)),
            BoxShadow::new(false, 0.0, 25.0, 35.0, 0.0, rgba(0, 0, 0, 150)),
            BoxShadow::new(true, 0.0, 4.0, 4.0, 0.0, rgba(255, 255, 255, 120)),
        ],
    );
    container.set_border_width_all(app, px(0));
    container.set_border_color_all(app, border_color);
    container.set_border_radius_all(app, (8.0, 8.0));
    container.set_padding(app, px(15), px(30), px(15), px(30));
    container.set_justify_content(app, JustifyContent::Center);
    container.set_background_color(app, base_color);
    container.add_click_listener(app, move |event, app, states| {
        let count = state.borrow_mut(states);
        *count += delta;
        count_text.set_text(app, &format!("Count: {count}"));
        event.stop_propagation();
    });
    container.push(app, label);
    container
}

pub fn counter(app: &mut App, states: &mut States) -> Container {
    let count = states.insert(0_i64);
    let count_text = Text::new(app, "Count: 0");
    let subtract = create_button(app, "-", rgb(244, 63, 94), -1, count, count_text);
    let add = create_button(app, "+", rgb(16, 185, 129), 1, count, count_text);
    let buttons = Container::new(app);
    buttons.set_column_gap(app, px(20));
    buttons.push(app, subtract);
    buttons.push(app, add);

    let container = Container::new(app);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_justify_content(app, JustifyContent::Center);
    container.set_align_items(app, AlignItems::Center);
    container.set_width(app, pct(100));
    container.set_height(app, pct(100));
    container.set_row_gap(app, px(20));
    container.push(app, count_text);
    container.push(app, buttons);
    container
}

pub fn main() {
    setup_logging();
    let mut app = App::new();
    let mut states = States::new();
    let content = counter(&mut app, &mut states);
    let window = Window::new(&mut app, "Counter");
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.push(&mut app, content);
    retgui_main(app, states, RetGuiOptions::basic("Counter"));
}
