use retgui::drivers::headless::run;
use retgui::elements::{Container, Element, State, Text, Window};
use retgui::events::{Event, PointerButton};
use retgui::geometry::Size;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RendererType, pct, px, rgb};

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
    let button = Container::new(app);
    button.set_border_width(app, px(1), px(2), px(3), px(4));
    button.set_border_color(app, border_color, border_color, border_color, border_color);
    button.set_border_radius(app, (10.0, 10.0), (10.0, 10.0), (10.0, 10.0), (10.0, 10.0));
    button.set_padding(app, px(15), px(30), px(15), px(30));
    button.set_justify_content(app, JustifyContent::Center);
    button.set_background_color(app, base_color);
    button.add_pointer_button_up_listener(app, move |event, app| {
        if event.button == Some(PointerButton::Left) {
            let count = state.update(app, |count| {
                *count += delta;
                *count
            });
            count_text.set_text(app, &format!("Count: {count}"));
            event.stop_propagation();
        }
    });
    button.push(app, label);
    button
}

#[cfg(test)]
mod test_utils;

#[test]
fn counter() {
    run(
        "counter_test",
        |app| {
            let count = app.insert_state(0_i64);
            let count_text = Text::new(app, "Count: 0");
            let subtract = create_button(app, "-", rgb(244, 67, 54), -1, count, count_text);
            let add_button = create_button(app, "+", rgb(76, 175, 80), 1, count, count_text);
            let buttons = Container::new(app);
            buttons.set_gap(app, px(20), px(20));
            buttons.push(app, subtract);
            buttons.push(app, add_button);
            let window = Window::new_with_renderer(app, "Counter", RendererType::VelloCPU);
            window.set_flex_direction(app, FlexDirection::Column);
            window.set_justify_content(app, JustifyContent::Center);
            window.set_align_items(app, AlignItems::Center);
            window.set_width(app, pct(100));
            window.set_height(app, pct(100));
            window.set_gap(app, px(20), px(20));
            window.push(app, count_text);
            window.push(app, buttons);
            (count, add_button, window)
        },
        |test, (count, add_button, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            for _ in 0..3 {
                test.click(&add_button);
            }

            assert_eq!(*test.app().state(count), 3);
            test_utils::check_snapshot(test_utils::screenshot_rgb(test, &window), "counter.png");
        },
    );
}
