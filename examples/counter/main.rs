use retgui::elements::{Button, Container, Element, Text};
use retgui::events::Event;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, retgui_main, rgb};

use retgui_builder::{ElementBuilder, TextBuilder, button, container, text, window};

fn create_button(
    app: &mut App<i64>,
    label: &str,
    accessibility_name: &str,
    base_color: Color,
    delta: i64,
    count_text: Text,
) -> Button {
    let button = button(app)
        .accessibility_name(app, accessibility_name)
        .border_radius_all(app, (8.0, 8.0))
        .padding(app, px(12), px(30), px(12), px(30))
        .justify_content(app, JustifyContent::Center)
        .background_color(app, base_color)
        .push(
            text(app, label)
                .font_size(app, 24.0)
                .color(app, Color::WHITE)
                .selectable(app, false),
            app,
        )
        .build();
    button.add_click_listener(app, move |event, app, count| {
        *count += delta;
        count_text.set_text(app, &format!("Count: {count}"));
        event.stop_propagation();
    });
    button
}

fn counter(app: &mut App<i64>) -> Container {
    let count_text = text(app, "Count: 0").build();
    container(app)
        .flex_direction(app, FlexDirection::Column)
        .justify_content(app, JustifyContent::Center)
        .align_items(app, AlignItems::Center)
        .width(app, pct(100))
        .height(app, pct(100))
        .row_gap(app, px(24))
        .font_size(app, 28.0)
        .color(app, rgb(63, 63, 70))
        .background_color(app, Color::WHITE)
        .push(count_text, app)
        .push(
            container(app)
                .column_gap(app, px(16))
                .push(
                    create_button(app, "−", "Decrement", rgb(244, 63, 94), -1, count_text),
                    app,
                )
                .push(
                    create_button(app, "+", "Increment", rgb(16, 185, 129), 1, count_text),
                    app,
                ),
            app,
        )
        .build()
}

pub fn main() {
    let mut app = App::new();
    window(&mut app, "Counter")
        .width(&mut app, pct(100))
        .height(&mut app, pct(100))
        .push(counter(&mut app), &mut app);
    retgui_main(app, 0_i64, RetGuiOptions::basic("Counter"));
}
