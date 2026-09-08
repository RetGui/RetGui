use retgui::elements::{Container, Element, Text, Window};
use retgui::events::{Event, PointerEnterEvent, PointerLeaveEvent};
use retgui::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Position, Unit};
use retgui::{App, Color, pct};

#[derive(Clone, Copy)]
struct EventLog {
    view: Container,
    entries: Container,
}

impl EventLog {
    fn push<S: 'static>(self, app: &mut App<S>, message: impl AsRef<str>) {
        let text = Text::new(app, message.as_ref());
        self.entries.push(app, text);
    }
}

fn title<S: 'static>(app: &mut App<S>, txt: &str) -> Text {
    let text = Text::new(app, txt);
    text.set_font_size(app, 24.0);
    text.set_padding(app, Unit::Px(0.0), Unit::Px(0.0), Unit::Px(25.0), Unit::Px(0.0));
    text
}

fn event_log<S: 'static>(app: &mut App<S>) -> EventLog {
    let entries = Container::new(app);
    entries.set_display(app, Display::Flex);
    entries.set_flex_direction(app, FlexDirection::Column);
    entries.set_overflow(app, Overflow::Visible, Overflow::Scroll);
    entries.set_width(app, Unit::Px(300.0));
    entries.set_height(app, Unit::Px(200.0));
    entries.set_max_width(app, Unit::Px(300.0));
    entries.set_max_height(app, Unit::Px(200.0));
    entries.set_border_width_all(app, Unit::Px(1.0));
    entries.set_margin(app, Unit::Px(25.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0));
    entries.set_border_color_all(app, Color::from_rgb8(99, 99, 99));

    let clear_log = Text::new(app, "Clear");
    clear_log.set_background_color(app, Color::from_rgb8(210, 210, 215));
    clear_log.set_border_width_all(app, Unit::Px(1.0));
    clear_log.set_border_radius_all(app, (6.0, 6.0));
    clear_log.set_padding(app, Unit::Px(10.0), Unit::Px(25.0), Unit::Px(10.0), Unit::Px(25.0));
    clear_log.set_width(app, Unit::Px(90.0));
    clear_log.add_click_listener(app, move |_event, app, _state| {
        entries.delete_all_children(app);
    });

    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, Unit::Px(20.0));
    container.push(app, entries);
    container.push(app, clear_log);

    EventLog {
        view: container,
        entries,
    }
}

fn pointer_capture_example<S: 'static>(app: &mut App<S>) -> Container {
    let container_padding = 20.0;

    let draggable_text = Text::new(app, "Draggable");
    let event_log = event_log(app);

    draggable_text.set_display(app, Display::Flex);
    draggable_text.set_width(app, Unit::Px(100.0));
    draggable_text.set_color(app, Color::WHITE);
    draggable_text.set_background_color(app, Color::from_rgba8(40, 40, 255, 100));
    draggable_text.add_pointer_button_down_listener(app, |event, app, _state| {
        event
            .target()
            .set_pointer_capture(app, event.pointer.pointer_id.unwrap());
    });
    draggable_text.add_pointer_moved_listener(app, move |event, app, _state| {
        let mouse_x = event.current.logical_position().x as f32;
        let half_width = draggable_text.computed_box_transformed(app).size.width / 2.0;
        if draggable_text.has_pointer_capture(app, event.pointer.pointer_id.unwrap()) {
            draggable_text.set_position(app, Position::Relative);
            draggable_text.set_inset(
                app,
                Unit::Px(0.0),
                Unit::Px(0.0),
                Unit::Px(0.0),
                Unit::Px(mouse_x - half_width - container_padding),
            );
        }
        event.prevent_default();
    });
    draggable_text.add_lost_pointer_capture_listener(app, move |_event, app, _state| {
        event_log.push(app, "Lost Pointer Capture");
    });
    draggable_text.add_got_pointer_capture_listener(app, move |_event, app, _state| {
        event_log.push(app, "Got Pointer Capture");
    });

    let heading = title(app, "Pointer Capture");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_padding_all(app, Unit::Px(container_padding));
    container.push(app, heading);
    container.push(app, draggable_text);
    container.push(app, event_log.view);
    container
}

fn pointer_enter_leave_example<S: 'static>(app: &mut App<S>) -> Container {
    let event_log = event_log(app);

    let pointer_enter_log = move |element_name: &'static str| {
        move |_event: &mut PointerEnterEvent, app: &mut App<S>, _state: &mut S| {
            event_log.push(app, format!("Pointer Enter: {element_name}"));
        }
    };
    let pointer_leave_log = move |element_name: &'static str| {
        move |_event: &mut PointerLeaveEvent, app: &mut App<S>, _state: &mut S| {
            event_log.push(app, format!("Pointer Leave: {element_name}"));
        }
    };

    let parent = Container::new(app);
    parent.set_display(app, Display::Flex);
    parent.set_flex_direction(app, FlexDirection::Row);
    parent.set_align_items(app, AlignItems::Center);
    parent.set_justify_content(app, JustifyContent::Center);
    parent.set_width(app, Unit::Px(250.0));
    parent.set_height(app, Unit::Px(250.0));
    parent.set_background_color(app, Color::from_rgba8(10, 10, 255, 150));
    parent.add_pointer_enter_listener(app, pointer_enter_log("Parent"));
    parent.add_pointer_leave_listener(app, pointer_leave_log("Parent"));

    let child_container = Container::new(app);
    child_container.set_width(app, Unit::Px(125.0));
    child_container.set_height(app, Unit::Px(125.0));
    child_container.set_background_color(app, Color::from_rgba8(255, 10, 10, 150));
    child_container.add_pointer_enter_listener(app, pointer_enter_log("Child"));
    child_container.add_pointer_leave_listener(app, pointer_leave_log("Child"));

    parent.push(app, child_container);
    let heading = title(app, "Pointer Enter + Leave");

    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_padding_all(app, Unit::Px(20.0));
    container.push(app, heading);
    container.push(app, parent);
    container.push(app, event_log.view);
    container
}

pub fn pointer_events<S: 'static>(app: &mut App<S>) -> Container {
    let capture = pointer_capture_example(app);
    let enter_leave = pointer_enter_leave_example(app);
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_overflow_y(app, Overflow::Scroll);
    container.set_max_height(app, Unit::Percentage(100.0));
    container.set_width(app, pct(100));
    container.set_height(app, pct(100));
    container.set_row_gap(app, Unit::Px(50.0));
    container.push(app, capture);
    container.push(app, enter_leave);
    container
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    let mut app = App::new();
    let content = pointer_events(&mut app);
    let window = Window::new(&mut app, "Pointer Events");
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.push(&mut app, content);

    use retgui::RetGuiOptions;

    //util::setup_logging();
    retgui::retgui_main(app, (), RetGuiOptions::basic("Pointer Events"));
}
