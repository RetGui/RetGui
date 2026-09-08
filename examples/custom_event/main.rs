use retgui::elements::{Container, Element, Text, Window};
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

struct Greeting {
    name: String,
}

pub fn custom_event<S: 'static>(app: &mut App<S>) -> Container {
    let message = Text::new(app, "No event received yet");

    let receiver = Container::new(app);
    receiver.add_custom_event_listener(app, move |event, app, _state| {
        if let Some(greeting) = event.data::<Greeting>() {
            message.set_text(app, &format!("Hello, {}!", greeting.name));
        }
    });
    receiver.push(app, message);

    let button_label = Text::new(app, "Send custom event");
    button_label.set_color(app, Color::WHITE);
    let button = Container::new(app);
    button.set_padding(app, px(12), px(20), px(12), px(20));
    button.set_background_color(app, rgb(59, 130, 246));
    button.add_click_listener(app, move |_event, app, _state| {
        receiver.emit_custom_event(
            app,
            Greeting {
                name: "Mary".to_string(),
            },
        );
    });
    button.push(app, button_label);

    let container = Container::new(app);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_justify_content(app, JustifyContent::Center);
    container.set_align_items(app, AlignItems::Center);
    container.set_width(app, pct(100));
    container.set_height(app, pct(100));
    container.set_row_gap(app, px(20));
    container.push(app, receiver);
    container.push(app, button);
    container
}

pub fn main() {
    setup_logging();
    let mut app = App::new();
    let content = custom_event(&mut app);
    let window = Window::new(&mut app, "Custom Event");
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.push(&mut app, content);
    retgui_main(app, (), RetGuiOptions::basic("Custom Event"));
}
