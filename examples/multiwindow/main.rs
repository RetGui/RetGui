use retgui::elements::{Container, Element, State, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use retgui::{App, Color, States, rgb};

#[derive(Default, Clone, Copy)]
pub struct Counter {
    count: i64,
}

impl Counter {
    fn change(&mut self, delta: i64) -> bool {
        self.count += delta;
        self.count >= 10
    }

    fn count(&self) -> i64 {
        self.count
    }
}

fn create_button(
    label: &str,
    base_color: Color,
    delta: i64,
    app: &mut App,
    state: State<Counter>,
    count_text: Text,
) -> Container {
    let label = Text::new(app, label);
    label.set_font_size(app, 24.0);
    label.set_color(app, Color::WHITE);
    label.set_selectable(app, false);
    let container = Container::new(app);
    container.set_border_width(app, Unit::Px(1.0), Unit::Px(2.0), Unit::Px(3.0), Unit::Px(4.0));
    container.set_border_color_all(app, rgb(0, 0, 0));
    container.set_border_radius_all(app, (10.0, 10.0));
    container.set_padding(app, Unit::Px(15.0), Unit::Px(30.0), Unit::Px(15.0), Unit::Px(30.0));
    container.set_display(app, Display::Flex);
    container.set_justify_content(app, JustifyContent::Center);
    container.set_align_items(app, AlignItems::Center);
    container.set_background_color(app, base_color);
    container.add_click_listener(app, move |event, app, states| {
        let state = state.borrow_mut(states);
        let create_window = state.change(delta);
        count_text.set_text(app, &format!("Count: {}", state.count()));
        if create_window {
            counter(app, states);
        }
        event.stop_propagation();
    });
    container.push(app, label);
    container
}

pub fn counter(app: &mut App, states: &mut States) -> Window {
    let count = states.insert(Counter::default());
    let count_text = Text::new(app, "Count: 0");
    let subtract = create_button("-", rgb(244, 67, 54), -1, app, count, count_text);
    let add = create_button("+", rgb(76, 175, 80), 1, app, count, count_text);
    let button = Container::new(app);
    button.set_display(app, Display::Flex);
    button.set_flex_direction(app, FlexDirection::Row);
    button.set_column_gap(app, Unit::Px(20.0));
    button.push(app, subtract);
    button.push(app, add);

    let window = Window::new(app, "MultiWindow");
    window.set_display(app, Display::Flex);
    window.set_flex_direction(app, FlexDirection::Column);
    window.set_justify_content(app, JustifyContent::Center);
    window.set_align_items(app, AlignItems::Center);
    window.set_width(app, Unit::Percentage(100.0));
    window.set_height(app, Unit::Percentage(100.0));
    window.set_row_gap(app, Unit::Px(20.0));
    window.push(app, count_text);
    window.set_font_size(app, 72.0);
    window.set_color(app, rgb(50, 50, 50));
    window.push(app, button);
    window
}

fn main() {
    let mut app = App::new();
    let mut states = States::new();
    let _counter1 = counter(&mut app, &mut states);
    use retgui::RetGuiOptions;

    util::setup_logging();
    retgui::retgui_main(app, states, RetGuiOptions::basic("Counter"));
}
